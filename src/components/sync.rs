use yew::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{RtcPeerConnection, RtcDataChannel, RtcSdpType, RtcSessionDescriptionInit};
use crate::storage;

#[derive(Clone, PartialEq)]
enum SyncMode {
    Idle,
    Sender,
    Receiver,
}

#[function_component(SyncPanel)]
pub fn sync_panel() -> Html {
    let mode = use_state(|| SyncMode::Idle);
    let offer_sdp = use_state(|| None::<String>);
    let answer_sdp = use_state(|| None::<String>);
    let connection_status = use_state(|| "Disconnected".to_string());
    
    // We'll use refs for the persistent WebRTC objects
    let peer_connection = use_mut_ref(|| None::<RtcPeerConnection>);
    let data_channel = use_mut_ref(|| None::<RtcDataChannel>);

    let start_sender = {
        let mode = mode.clone();
        let offer_sdp = offer_sdp.clone();
        let pc_ref = peer_connection.clone();
        let dc_ref = data_channel.clone();
        let status = connection_status.clone();
        
        Callback::from(move |_| {
            mode.set(SyncMode::Sender);
            status.set("Creating offer...".to_string());
            
            let config = web_sys::RtcConfiguration::new();
            let ice_servers = js_sys::Array::new();
            let server = web_sys::RtcIceServer::new();
            server.set_urls(&wasm_bindgen::JsValue::from_str("stun:stun.l.google.com:19302"));
            ice_servers.push(&server);
            config.set_ice_servers(&ice_servers);
            
            let pc = RtcPeerConnection::new_with_configuration(&config).unwrap();
            
            // Create data channel
            let dc = pc.create_data_channel("sync");
            let status_c = status.clone();
            let dc_ref_c = dc_ref.clone();
            let onopen = Closure::wrap(Box::new(move |_e: web_sys::Event| {
                status_c.set("Connected! Sending data...".to_string());
                // Send data when channel opens
                let data = storage::export_all_data();
                if let Some(dc) = dc_ref_c.borrow().as_ref() {
                    let _ = dc.send_with_str(&data);
                }
                status_c.set("Data sent successfully!".to_string());
            }) as Box<dyn FnMut(web_sys::Event)>);
            dc.set_onopen(Some(onopen.as_ref().unchecked_ref()));
            onopen.forget();
            
            *dc_ref.borrow_mut() = Some(dc);
            
            let offer_c = offer_sdp.clone();
            let pc_c = pc.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let offer = wasm_bindgen_futures::JsFuture::from(pc_c.create_offer()).await.unwrap();
                let offer_obj = offer.unchecked_into::<RtcSessionDescriptionInit>();
                let sdp = offer_obj.get_sdp().unwrap_or_default();
                let _ = wasm_bindgen_futures::JsFuture::from(pc_c.set_local_description(&offer_obj)).await;
                offer_c.set(Some(sdp));
            });
            
            *pc_ref.borrow_mut() = Some(pc);
        })
    };

    let start_receiver = {
        let mode = mode.clone();
        Callback::from(move |_| {
            mode.set(SyncMode::Receiver);
        })
    };

    let on_paste_offer = {
        let pc_ref = peer_connection.clone();
        let answer_sdp = answer_sdp.clone();
        let status = connection_status.clone();
        
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlTextAreaElement = e.target_unchecked_into();
            let sdp = input.value();
            if sdp.is_empty() { return; }
            
            let config = web_sys::RtcConfiguration::new();
            let ice_servers = js_sys::Array::new();
            let server = web_sys::RtcIceServer::new();
            server.set_urls(&wasm_bindgen::JsValue::from_str("stun:stun.l.google.com:19302"));
            ice_servers.push(&server);
            config.set_ice_servers(&ice_servers);
            
            let pc = RtcPeerConnection::new_with_configuration(&config).unwrap();
            status.set("Processing offer...".to_string());
            
            // Set up data channel handler
            let status_c = status.clone();
            let ondatachannel = Closure::wrap(Box::new(move |e: web_sys::RtcDataChannelEvent| {
                let dc = e.channel();
                let status_cc = status_c.clone();
                let onmessage = Closure::wrap(Box::new(move |e: web_sys::MessageEvent| {
                    if let Some(text) = e.data().as_string() {
                        match storage::import_all_data(&text) {
                            Ok(_) => status_cc.set("Data received and imported!".to_string()),
                            Err(e) => status_cc.set(format!("Import error: {}", e)),
                        }
                    }
                }) as Box<dyn FnMut(web_sys::MessageEvent)>);
                dc.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
                onmessage.forget();
            }) as Box<dyn FnMut(web_sys::RtcDataChannelEvent)>);
            pc.set_ondatachannel(Some(ondatachannel.as_ref().unchecked_ref()));
            ondatachannel.forget();

            let pc_c = pc.clone();
            let answer_c = answer_sdp.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let remote_desc = RtcSessionDescriptionInit::new(RtcSdpType::Offer);
                remote_desc.set_sdp(&sdp);
                let _ = wasm_bindgen_futures::JsFuture::from(pc_c.set_remote_description(&remote_desc)).await;
                
                let answer = wasm_bindgen_futures::JsFuture::from(pc_c.create_answer()).await.unwrap();
                let answer_obj = answer.unchecked_into::<RtcSessionDescriptionInit>();
                let answer_sdp_text = answer_obj.get_sdp().unwrap_or_default();
                let _ = wasm_bindgen_futures::JsFuture::from(pc_c.set_local_description(&answer_obj)).await;
                answer_c.set(Some(answer_sdp_text));
            });
            
            *pc_ref.borrow_mut() = Some(pc);
        })
    };

    let on_paste_answer = {
        let pc_ref = peer_connection.clone();
        let status = connection_status.clone();
        
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlTextAreaElement = e.target_unchecked_into();
            let sdp = input.value();
            if sdp.is_empty() { return; }
            
            if let Some(pc) = pc_ref.borrow().as_ref() {
                let pc = pc.clone();
                status.set("Connecting...".to_string());
                wasm_bindgen_futures::spawn_local(async move {
                    let remote_desc = RtcSessionDescriptionInit::new(RtcSdpType::Answer);
                    remote_desc.set_sdp(&sdp);
                    let _ = wasm_bindgen_futures::JsFuture::from(pc.set_remote_description(&remote_desc)).await;
                });
            }
        })
    };

    let render_qr = |data: &str| {
        let result: Result<String, _> = qrcode_generator::to_svg_to_string(data, qrcode_generator::QrCodeEcc::Low, 400, None::<&str>);
        match result {
            Ok(svg) => {
                let base64 = gloo::utils::window().btoa(&svg).unwrap_or_default();
                html! {
                    <div class="bg-white p-2 rounded-lg flex justify-center">
                        <img src={format!("data:image/svg+xml;base64,{}", base64)} class="w-64 h-64" />
                    </div>
                }
            }
            Err(_) => html! { <div class="text-red-500">{"Failed to generate QR"}</div> }
        }
    };

    html! {
        <div class="bg-gray-800 rounded-lg p-4 space-y-4">
            <h3 class="font-semibold text-lg">{"Direct Sync (P2P)"}</h3>
            <p class="text-sm text-gray-400">
                {"Transfer data directly between devices. Private and no server needed."}
            </p>

            { match *mode {
                SyncMode::Idle => html! {
                    <div class="grid grid-cols-2 gap-4">
                        <button onclick={start_sender} class="py-3 bg-blue-600 rounded-lg font-medium hover:bg-blue-700 flex flex-col items-center">
                            <span class="text-xl">{"📤"}</span>
                            <span>{"Send Data"}</span>
                        </button>
                        <button onclick={start_receiver} class="py-3 bg-green-600 rounded-lg font-medium hover:bg-green-700 flex flex-col items-center">
                            <span class="text-xl">{"📥"}</span>
                            <span>{"Receive"}</span>
                        </button>
                    </div>
                },
                SyncMode::Sender => html! {
                    <div class="space-y-3">
                        <div class="flex justify-between items-center">
                            <span class="text-sm font-medium">{"Step 1: Scan this Offer on other device"}</span>
                            <button onclick={let m = mode.clone(); Callback::from(move |_| m.set(SyncMode::Idle))} class="text-xs text-gray-400 underline">{"Cancel"}</button>
                        </div>
                        { if let Some(sdp) = &*offer_sdp {
                            html! {
                                <>
                                    {render_qr(sdp)}
                                    <details>
                                        <summary class="text-xs text-gray-500 cursor-pointer">{"Or copy text"}</summary>
                                        <textarea readonly=true class="w-full h-24 bg-gray-900 text-xs p-2 rounded border border-gray-700 mt-2" value={sdp.clone()} />
                                    </details>
                                    <div class="text-sm font-medium pt-2">{"Step 2: Paste Answer from other device"}</div>
                                    <textarea oninput={on_paste_answer} placeholder="Paste Answer SDP here..." class="w-full h-24 bg-gray-900 text-xs p-2 rounded border border-gray-700 focus:border-blue-500" />
                                </>
                            }
                        } else {
                            html! { <div class="animate-pulse text-sm text-blue-400">{"Generating offer..."}</div> }
                        }}
                        <div class="text-xs font-mono text-center text-blue-300">{&*connection_status}</div>
                    </div>
                },
                SyncMode::Receiver => html! {
                    <div class="space-y-3">
                        <div class="flex justify-between items-center">
                            <span class="text-sm font-medium">{"Step 1: Paste Offer from other device"}</span>
                            <button onclick={let m = mode.clone(); Callback::from(move |_| m.set(SyncMode::Idle))} class="text-xs text-gray-400 underline">{"Cancel"}</button>
                        </div>
                        <textarea oninput={on_paste_offer} placeholder="Paste Offer SDP here..." class="w-full h-24 bg-gray-900 text-xs p-2 rounded border border-gray-700 focus:border-green-500" />
                        
                        { if let Some(sdp) = &*answer_sdp {
                            html! {
                                <>
                                    <div class="text-sm font-medium pt-2">{"Step 2: Scan this Answer back on sender"}</div>
                                    {render_qr(sdp)}
                                    <details>
                                        <summary class="text-xs text-gray-500 cursor-pointer">{"Or copy text"}</summary>
                                        <textarea readonly=true class="w-full h-24 bg-gray-900 text-xs p-2 rounded border border-gray-700 mt-2" value={sdp.clone()} />
                                    </details>
                                </>
                            }
                        } else { html! {} }}
                        <div class="text-xs font-mono text-center text-green-300">{&*connection_status}</div>
                    </div>
                }
            }}
        </div>
    }
}
