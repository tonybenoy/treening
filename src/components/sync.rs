use yew::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use crate::storage;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = Peer)]
    #[derive(Clone)]
    type Peer;

    #[wasm_bindgen(constructor, js_name = Peer)]
    fn new(id: Option<&str>) -> Peer;

    #[wasm_bindgen(method, js_name = on)]
    fn on(this: &Peer, event: &str, callback: &JsValue);

    #[wasm_bindgen(method, js_name = connect)]
    fn connect(this: &Peer, id: &str) -> DataConnection;

    #[wasm_bindgen(js_name = DataConnection)]
    #[derive(Clone)]
    type DataConnection;

    #[wasm_bindgen(method, js_name = on)]
    fn on_conn(this: &DataConnection, event: &str, callback: &JsValue);

    #[wasm_bindgen(method, js_name = send)]
    fn send(this: &DataConnection, data: &str);

    #[wasm_bindgen(method, getter, js_name = id)]
    fn id(this: &Peer) -> String;
}

#[derive(Clone, PartialEq)]
enum SyncMode {
    Idle,
    Sender,
    Receiver,
}

#[function_component(SyncPanel)]
pub fn sync_panel() -> Html {
    let mode = use_state(|| SyncMode::Idle);
    let peer_id = use_state(|| String::new());
    let target_id = use_state(|| String::new());
    let connection_status = use_state(|| "Ready".to_string());
    
    let peer_ref = use_mut_ref(|| None::<Peer>);

    let start_sender = {
        let mode = mode.clone();
        let status = connection_status.clone();
        let peer_id_state = peer_id.clone();
        let peer_ref = peer_ref.clone();
        
        Callback::from(move |_| {
            mode.set(SyncMode::Sender);
            status.set("Connecting to relay...".to_string());
            
            let peer = Peer::new(None);
            let status_c = status.clone();
            let peer_id_c = peer_id_state.clone();
            let peer_ref_c = peer_ref.clone();

            // When peer is open
            let on_open = Closure::wrap(Box::new(move |id: String| {
                peer_id_c.set(id);
                status_c.set("Waiting for receiver...".to_string());
            }) as Box<dyn FnMut(String)>);
            peer.on("open", on_open.as_ref().unchecked_ref());
            on_open.forget();

            // When someone connects to us
            let status_cc = status.clone();
            let on_connection = Closure::wrap(Box::new(move |conn: DataConnection| {
                status_cc.set("Connected! Sending data...".to_string());
                let data = storage::export_all_data();
                
                let status_ccc = status_cc.clone();
                let conn_c = conn.clone();
                let on_open_conn = Closure::wrap(Box::new(move || {
                    conn_c.send(&data);
                    status_ccc.set("Data sent successfully!".to_string());
                }) as Box<dyn FnMut()>);
                conn.on_conn("open", on_open_conn.as_ref().unchecked_ref());
                on_open_conn.forget();
            }) as Box<dyn FnMut(DataConnection)>);
            peer.on("connection", on_connection.as_ref().unchecked_ref());
            on_connection.forget();

            *peer_ref_c.borrow_mut() = Some(peer);
        })
    };

    let start_receiver = {
        let mode = mode.clone();
        Callback::from(move |_| {
            mode.set(SyncMode::Receiver);
        })
    };

    let connect_to_peer = {
        let status = connection_status.clone();
        let target_id = target_id.clone();
        let peer_ref = peer_ref.clone();
        
        Callback::from(move |_| {
            let id = (*target_id).clone();
            if id.is_empty() { return; }
            
            status.set("Connecting...".to_string());
            
            // Re-use or create peer
            if peer_ref.borrow().is_none() {
                *peer_ref.borrow_mut() = Some(Peer::new(None));
            }
            
            let peer = peer_ref.borrow().as_ref().unwrap().clone();
            let status_c = status.clone();
            let id_c = id.clone();
            
            let peer_for_open = peer.clone();
            let status_for_open = status_c.clone();
            let id_for_open = id.clone();
            let on_open = Closure::wrap(Box::new(move |_id: String| {
                let conn = peer_for_open.connect(&id_for_open);
                let status_cc = status_for_open.clone();
                
                let status_data = status_cc.clone();
                let on_data = Closure::wrap(Box::new(move |data: String| {
                    match storage::merge_all_data(&data) {
                        Ok(_) => status_data.set("Data received and merged!".to_string()),
                        Err(e) => status_data.set(format!("Error: {}", e)),
                    }
                }) as Box<dyn FnMut(String)>);
                conn.on_conn("data", on_data.as_ref().unchecked_ref());
                on_data.forget();
                
                let status_ccc = status_cc.clone();
                let on_open_conn = Closure::wrap(Box::new(move || {
                    status_ccc.set("Connected! Receiving...".to_string());
                }) as Box<dyn FnMut()>);
                conn.on_conn("open", on_open_conn.as_ref().unchecked_ref());
                on_open_conn.forget();
            }) as Box<dyn FnMut(String)>);
            
            // We need to handle the case where peer is already open
            let id_str = peer.id();
            if !id_str.is_empty() {
                let conn = peer.connect(&id);
                let status_cc = status_c.clone();
                
                let status_data = status_cc.clone();
                let on_data = Closure::wrap(Box::new(move |data: String| {
                    match storage::merge_all_data(&data) {
                        Ok(_) => status_data.set("Data received and merged!".to_string()),
                        Err(e) => status_data.set(format!("Error: {}", e)),
                    }
                }) as Box<dyn FnMut(String)>);
                conn.on_conn("data", on_data.as_ref().unchecked_ref());
                on_data.forget();
                
                let status_ccc = status_cc.clone();
                let on_open_conn = Closure::wrap(Box::new(move || {
                    status_ccc.set("Connected! Receiving...".to_string());
                }) as Box<dyn FnMut()>);
                conn.on_conn("open", on_open_conn.as_ref().unchecked_ref());
                on_open_conn.forget();
            } else {
                peer.on("open", on_open.as_ref().unchecked_ref());
                on_open.forget();
            }
        })
    };

    let render_qr = |data: &str| {
        let result: Result<String, _> = qrcode_generator::to_svg_to_string(data, qrcode_generator::QrCodeEcc::Low, 400, None::<&str>);
        match result {
            Ok(svg) => {
                let base64 = gloo::utils::window().btoa(&svg).unwrap_or_default();
                html! {
                    <div class="bg-white p-4 rounded-xl flex justify-center shadow-lg">
                        <img src={format!("data:image/svg+xml;base64,{}", base64)} class="w-64 h-64" />
                    </div>
                }
            }
            Err(_) => html! { <div class="text-red-500">{"Failed to generate QR"}</div> }
        }
    };

    html! {
        <div class="bg-gray-800 rounded-lg p-4 space-y-4">
            <h3 class="font-semibold text-lg flex items-center gap-2">
                <span>{"📲"}</span> {"Sync Devices"}
            </h3>
            
            { match *mode {
                SyncMode::Idle => html! {
                    <div class="space-y-4">
                        <p class="text-sm text-gray-400">
                            {"Transfer and merge data between your devices. No data is stored on any server."}
                        </p>
                        <div class="grid grid-cols-2 gap-4">
                            <button onclick={start_sender} class="py-4 bg-blue-600 rounded-xl font-bold hover:bg-blue-700 transition flex flex-col items-center gap-2 shadow-lg shadow-blue-900/20">
                                <span class="text-2xl">{"📤"}</span>
                                <span>{"Send"}</span>
                            </button>
                            <button onclick={start_receiver} class="py-4 bg-gray-700 rounded-xl font-bold hover:bg-gray-600 transition flex flex-col items-center gap-2">
                                <span class="text-2xl">{"📥"}</span>
                                <span>{"Receive"}</span>
                            </button>
                        </div>
                    </div>
                },
                        <div class="space-y-4 text-center">
                            <div class="flex justify-between items-center mb-2">
                                <span class="text-sm font-medium text-blue-400 uppercase tracking-wider">{"Sending Mode"}</span>
                                <button onclick={let m = mode.clone(); Callback::from(move |_| m.set(SyncMode::Idle))} class="text-gray-500 hover:text-white">{"✕"}</button>
                            </div>

                            <div class="bg-blue-900/20 p-3 rounded-lg text-xs text-blue-300 text-left space-y-2">
                                <p class="font-bold uppercase tracking-widest text-[10px]">{"Instructions:"}</p>
                                <p>{"1. Open Treening on the other device."}</p>
                                <p>{"2. Go to Sync and tap \"Receive\"."}</p>
                                <p>{"3. Scan this QR code or enter the Meeting ID below."}</p>
                            </div>
                            
                            { if !peer_id.is_empty() {
                            html! {
                                <>
                                    {render_qr(&peer_id)}
                                    <div class="space-y-1">
                                        <div class="text-xs text-gray-500 uppercase">{"Meeting ID"}</div>
                                        <div class="text-2xl font-mono font-bold text-white tracking-widest">{&*peer_id}</div>
                                    </div>
                                    <p class="text-sm text-gray-400">{"Scan this QR or enter the ID on your other device."}</p>
                                </>
                            }
                        } else {
                            html! { <div class="py-12 animate-pulse text-blue-400">{"Generating ID..."}</div> }
                        }}
                        
                        <div class="text-xs font-medium py-2 px-3 bg-blue-900/30 rounded-full inline-block text-blue-300">
                            {&*connection_status}
                        </div>
                    </div>
                },
                SyncMode::Receiver => html! {
                    <div class="space-y-4">
                        <div class="flex justify-between items-center mb-2">
                            <span class="text-sm font-medium text-green-400 uppercase tracking-wider">{"Receiving Mode"}</span>
                            <button onclick={let m = mode.clone(); Callback::from(move |_| m.set(SyncMode::Idle))} class="text-gray-500 hover:text-white">{"✕"}</button>
                        </div>

                        <div class="bg-green-900/20 p-3 rounded-lg text-xs text-green-300 text-left space-y-2 mb-4">
                            <p class="font-bold uppercase tracking-widest text-[10px]">{"Instructions:"}</p>
                            <p>{"1. Tap \"Send\" on the device that has your data."}</p>
                            <p>{"2. Enter the Meeting ID shown on that device below."}</p>
                            <p>{"3. Once connected, your data will merge automatically."}</p>
                        </div>

                        <div class="space-y-3">
                            <label class="block">
                                <span class="text-xs text-gray-500 uppercase mb-1 block">{"Enter Meeting ID"}</span>
                                <input 
                                    type="text" 
                                    placeholder="e.g. apple-banana-cherry"
                                    class="w-full bg-gray-900 border border-gray-700 rounded-xl py-3 px-4 text-center text-xl font-mono focus:border-green-500 outline-none transition"
                                    oninput={let target = target_id.clone(); Callback::from(move |e: InputEvent| {
                                        let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                        target.set(input.value());
                                    })}
                                />
                            </label>
                            
                            <button 
                                onclick={connect_to_peer}
                                disabled={target_id.is_empty()}
                                class="w-full py-4 bg-green-600 disabled:opacity-50 disabled:bg-gray-700 rounded-xl font-bold hover:bg-green-700 transition shadow-lg shadow-green-900/20"
                            >
                                {"Connect & Sync"}
                            </button>
                        </div>

                        <div class="text-center">
                            <div class="text-xs font-medium py-2 px-3 bg-green-900/30 rounded-full inline-block text-green-300">
                                {&*connection_status}
                            </div>
                        </div>
                        
                        <p class="text-xs text-gray-500 text-center italic">
                            {"Tip: Once connected, your data will be merged automatically."}
                        </p>
                    </div>
                }
            }}
        </div>
    }
}
