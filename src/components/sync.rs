use yew::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use crate::storage;
use crate::models::TrustedDevice;

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

    #[wasm_bindgen(method, js_name = destroy)]
    fn destroy(this: &Peer);

    #[wasm_bindgen(js_name = DataConnection)]
    #[derive(Clone)]
    type DataConnection;

    #[wasm_bindgen(method, js_name = on)]
    fn on_conn(this: &DataConnection, event: &str, callback: &JsValue);

    #[wasm_bindgen(method, js_name = send)]
    fn send(this: &DataConnection, data: &str);

    #[wasm_bindgen(method, getter, js_name = id)]
    fn id(this: &Peer) -> String;

    #[wasm_bindgen(method, getter, js_name = peer)]
    fn remote_peer(this: &DataConnection) -> String;
}

fn now_iso() -> String {
    let date = js_sys::Date::new_0();
    date.to_iso_string().as_string().unwrap_or_default()
}

fn handle_sync_data(conn: &DataConnection, devices: UseStateHandle<Vec<TrustedDevice>>, status: UseStateHandle<String>) {
    let devices = devices.clone();
    let status = status.clone();
    let on_data = Closure::wrap(Box::new(move |data: JsValue| {
        if let Some(msg) = data.as_string() {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&msg) {
                if val.get("type").and_then(|t| t.as_str()) == Some("device_sync") {
                    if let Some(payload) = val.get("data").and_then(|d| d.as_str()) {
                        match storage::merge_all_data(payload) {
                            Ok(_) => {
                                let remote_id = val.get("peer_id").and_then(|p| p.as_str()).unwrap_or("").to_string();
                                let remote_name = val.get("name").and_then(|n| n.as_str()).unwrap_or("Device").to_string();
                                let now = now_iso();

                                // Update last_synced for this device, or auto-add it
                                let mut devs = (*devices).clone();
                                if let Some(d) = devs.iter_mut().find(|d| d.peer_id == remote_id) {
                                    d.last_synced = Some(now);
                                    if d.name == "New Device" && remote_name != "Athlete" {
                                        d.name = remote_name;
                                    }
                                } else if !remote_id.is_empty() {
                                    devs.push(TrustedDevice {
                                        peer_id: remote_id,
                                        name: remote_name,
                                        last_synced: Some(now),
                                    });
                                }
                                storage::save_trusted_devices(&devs);
                                devices.set(devs);
                                status.set("Synced!".to_string());
                            }
                            Err(e) => {
                                status.set(format!("Sync error: {}", e));
                            }
                        }
                    }
                }
            }
        }
    }) as Box<dyn FnMut(JsValue)>);
    conn.on_conn("data", on_data.as_ref().unchecked_ref());
    on_data.forget();
}

fn send_sync_data(conn: &DataConnection) {
    let config = storage::load_user_config();
    let payload = serde_json::json!({
        "type": "device_sync",
        "peer_id": config.peer_id,
        "name": config.nickname,
        "data": storage::export_all_data()
    })
    .to_string();
    conn.send(&payload);
}

#[function_component(SyncPanel)]
pub fn sync_panel() -> Html {
    let user_config = storage::load_user_config();
    let my_peer_id = user_config.peer_id.clone();

    let devices = use_state(storage::load_trusted_devices);
    let status = use_state(|| "Connecting...".to_string());
    let add_id_input = use_state(String::new);
    let editing_idx = use_state(|| None::<usize>);
    let edit_name_input = use_state(String::new);

    let peer_ref = use_mut_ref(|| None::<Peer>);

    // Initialize peer and auto-sync on mount
    {
        let devices = devices.clone();
        let status = status.clone();
        let peer_ref = peer_ref.clone();
        let my_id = my_peer_id.clone();

        use_effect_with((), move |_| {
            // Handle ?pair= URL parameter for auto-adding a device
            let window = gloo::utils::window();
            if let Ok(search) = window.location().search() {
                let params = web_sys::UrlSearchParams::new_with_str(&search).unwrap();
                if let Some(pair_id) = params.get("pair") {
                    let mut devs = (*devices).clone();
                    if !pair_id.is_empty() && !devs.iter().any(|d| d.peer_id == pair_id) {
                        devs.push(TrustedDevice {
                            peer_id: pair_id,
                            name: "New Device".to_string(),
                            last_synced: None,
                        });
                        storage::save_trusted_devices(&devs);
                        devices.set(devs);
                    }
                }
            }

            let peer = Peer::new(Some(&my_id));
            let peer_for_open = peer.clone();
            let devices_for_open = devices.clone();
            let status_for_open = status.clone();
            let peer_ref_c = peer_ref.clone();

            // On peer open: connect to all trusted devices
            let on_open = Closure::wrap(Box::new(move |_id: String| {
                status_for_open.set("Online".to_string());

                let devs = (*devices_for_open).clone();
                for d in &devs {
                    let conn = peer_for_open.connect(&d.peer_id);
                    let devices_c = devices_for_open.clone();
                    let status_c = status_for_open.clone();
                    let conn_c = conn.clone();

                    let on_open_conn = Closure::wrap(Box::new(move || {
                        send_sync_data(&conn_c);
                    }) as Box<dyn FnMut()>);
                    conn.on_conn("open", on_open_conn.as_ref().unchecked_ref());
                    on_open_conn.forget();

                    handle_sync_data(&conn, devices_c, status_c);
                }
            }) as Box<dyn FnMut(String)>);
            peer.on("open", on_open.as_ref().unchecked_ref());
            on_open.forget();

            // On peer error
            let status_err = status.clone();
            let on_error = Closure::wrap(Box::new(move |_err: JsValue| {
                status_err.set("Connection error".to_string());
            }) as Box<dyn FnMut(JsValue)>);
            peer.on("error", on_error.as_ref().unchecked_ref());
            on_error.forget();

            // Handle incoming connections (other device connecting to us)
            let devices_incoming = devices.clone();
            let status_incoming = status.clone();
            let on_connection = Closure::wrap(Box::new(move |conn: DataConnection| {
                let conn_c = conn.clone();
                let on_open_conn = Closure::wrap(Box::new(move || {
                    send_sync_data(&conn_c);
                }) as Box<dyn FnMut()>);
                conn.on_conn("open", on_open_conn.as_ref().unchecked_ref());
                on_open_conn.forget();

                handle_sync_data(&conn, devices_incoming.clone(), status_incoming.clone());
            }) as Box<dyn FnMut(DataConnection)>);
            peer.on("connection", on_connection.as_ref().unchecked_ref());
            on_connection.forget();

            *peer_ref_c.borrow_mut() = Some(peer);

            // Cleanup
            let peer_ref_cleanup = peer_ref.clone();
            move || {
                if let Some(p) = peer_ref_cleanup.borrow_mut().take() {
                    p.destroy();
                }
            }
        });
    }

    // Add device callback
    let on_add_device = {
        let add_id_input = add_id_input.clone();
        let devices = devices.clone();
        let status = status.clone();
        let peer_ref = peer_ref.clone();

        Callback::from(move |_| {
            let new_id = (*add_id_input).clone().trim().to_string();
            if new_id.is_empty() {
                return;
            }

            // Don't add duplicates
            let mut devs = (*devices).clone();
            if devs.iter().any(|d| d.peer_id == new_id) {
                status.set("Device already added".to_string());
                return;
            }

            devs.push(TrustedDevice {
                peer_id: new_id.clone(),
                name: "New Device".to_string(),
                last_synced: None,
            });
            storage::save_trusted_devices(&devs);
            devices.set(devs);
            add_id_input.set(String::new());

            // Immediately try to connect
            if let Some(peer) = peer_ref.borrow().as_ref() {
                let conn = peer.connect(&new_id);
                let devices_c = devices.clone();
                let status_c = status.clone();
                let conn_c = conn.clone();

                let on_open_conn = Closure::wrap(Box::new(move || {
                    send_sync_data(&conn_c);
                }) as Box<dyn FnMut()>);
                conn.on_conn("open", on_open_conn.as_ref().unchecked_ref());
                on_open_conn.forget();

                handle_sync_data(&conn, devices_c, status_c);
            }

            status.set("Connecting to new device...".to_string());
        })
    };

    // Remove device
    let on_remove = {
        let devices = devices.clone();
        Callback::from(move |idx: usize| {
            let mut devs = (*devices).clone();
            if idx < devs.len() {
                devs.remove(idx);
                storage::save_trusted_devices(&devs);
                devices.set(devs);
            }
        })
    };

    // Manual sync all
    let on_sync_all = {
        let peer_ref = peer_ref.clone();
        let devices = devices.clone();
        let status = status.clone();

        Callback::from(move |_| {
            if let Some(peer) = peer_ref.borrow().as_ref() {
                let devs = (*devices).clone();
                for d in &devs {
                    let conn = peer.connect(&d.peer_id);
                    let devices_c = devices.clone();
                    let status_c = status.clone();
                    let conn_c = conn.clone();

                    let on_open_conn = Closure::wrap(Box::new(move || {
                        send_sync_data(&conn_c);
                    }) as Box<dyn FnMut()>);
                    conn.on_conn("open", on_open_conn.as_ref().unchecked_ref());
                    on_open_conn.forget();

                    handle_sync_data(&conn, devices_c, status_c);
                }
                status.set("Syncing...".to_string());
            }
        })
    };

    // Copy peer ID
    let on_copy_id = {
        let id = my_peer_id.clone();
        Callback::from(move |_| {
            let window = gloo::utils::window();
            let _ = window.navigator().clipboard().write_text(&id);
        })
    };

    // Share pairing link
    let on_share_link = {
        let id = my_peer_id.clone();
        Callback::from(move |_| {
            let url = format!("https://treen.ing/#/settings?pair={}", id);
            let window = gloo::utils::window();
            let navigator = window.navigator();
            let share_fn = js_sys::Reflect::get(&navigator, &"share".into()).ok();
            let can_share = share_fn.as_ref().map(|v| v.is_function()).unwrap_or(false);
            if can_share {
                let data = js_sys::Object::new();
                let _ = js_sys::Reflect::set(&data, &"title".into(), &"Pair with my Treening app".into());
                let _ = js_sys::Reflect::set(&data, &"text".into(), &format!("Pair your Treening app with mine to auto-sync!\n{}", url).into());
                let _ = js_sys::Reflect::set(&data, &"url".into(), &url.into());
                let share = share_fn.unwrap();
                let _ = js_sys::Function::from(share).call1(&navigator, &data);
            } else {
                let _ = navigator.clipboard().write_text(&url);
            }
        })
    };

    // Start editing device name
    let on_edit_start = {
        let editing_idx = editing_idx.clone();
        let edit_name_input = edit_name_input.clone();
        let devices = devices.clone();
        Callback::from(move |(idx,): (usize,)| {
            let devs = (*devices).clone();
            if let Some(d) = devs.get(idx) {
                edit_name_input.set(d.name.clone());
                editing_idx.set(Some(idx));
            }
        })
    };

    // Save edited name
    let on_edit_save = {
        let editing_idx = editing_idx.clone();
        let edit_name_input = edit_name_input.clone();
        let devices = devices.clone();
        Callback::from(move |_| {
            if let Some(idx) = *editing_idx {
                let mut devs = (*devices).clone();
                if let Some(d) = devs.get_mut(idx) {
                    d.name = (*edit_name_input).clone();
                    storage::save_trusted_devices(&devs);
                    devices.set(devs);
                }
                editing_idx.set(None);
            }
        })
    };

    let status_color = match (*status).as_str() {
        "Online" => "text-green-600 dark:text-green-400 bg-green-100 dark:bg-green-900/30",
        "Synced!" => "text-green-600 dark:text-green-400 bg-green-100 dark:bg-green-900/30",
        "Connecting..." | "Syncing..." | "Connecting to new device..." => "text-blue-600 dark:text-blue-400 bg-blue-100 dark:bg-blue-900/30",
        _ => "text-red-600 dark:text-red-400 bg-red-100 dark:bg-red-900/30",
    };

    html! {
        <div class="bg-gray-100 dark:bg-gray-800 rounded-lg p-4 space-y-4 border border-gray-200 dark:border-transparent transition-colors shadow-sm">
            <div class="flex items-center justify-between">
                <h3 class="font-bold text-lg flex items-center gap-2 text-gray-900 dark:text-gray-100">
                    <span>{"📲"}</span> {"Trusted Devices"}
                </h3>
                <span class={classes!("text-[10px]", "font-bold", "py-1", "px-2", "rounded-full", "uppercase", "tracking-wider", status_color)}>
                    {&*status}
                </span>
            </div>

            // Your Device ID
            <div class="bg-white dark:bg-gray-900 rounded-xl p-3 space-y-1 border border-gray-200 dark:border-gray-700">
                <div class="text-[10px] text-gray-500 uppercase font-bold tracking-wider">{"Your Device ID"}</div>
                <div class="flex items-center gap-2">
                    <code class="text-sm font-mono font-bold text-gray-900 dark:text-white flex-1 break-all">{&my_peer_id}</code>
                    <button
                        onclick={on_copy_id}
                        class="p-2 bg-gray-200 dark:bg-gray-700 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-600 transition text-gray-700 dark:text-gray-200 shrink-0"
                        title="Copy ID"
                    >
                        {"📋"}
                    </button>
                    <button
                        onclick={on_share_link}
                        class="px-3 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition text-xs font-bold shrink-0"
                        title="Share pairing link"
                    >
                        {"Share Link"}
                    </button>
                </div>
                <p class="text-[10px] text-gray-400">{"Share this link with your other device to pair instantly."}</p>
            </div>

            // Add device form
            <div class="space-y-2">
                <div class="text-[10px] text-gray-500 uppercase font-bold tracking-wider">{"Add Device"}</div>
                <div class="flex gap-2">
                    <input
                        type="text"
                        placeholder="Enter device ID (e.g. tr-abc12345)"
                        value={(*add_id_input).clone()}
                        class="flex-1 bg-white dark:bg-gray-900 border border-gray-300 dark:border-gray-700 rounded-xl py-2 px-3 text-sm font-mono focus:border-blue-500 dark:focus:border-blue-500 outline-none transition-colors text-gray-900 dark:text-white"
                        oninput={let input = add_id_input.clone(); Callback::from(move |e: InputEvent| {
                            let el: web_sys::HtmlInputElement = e.target_unchecked_into();
                            input.set(el.value());
                        })}
                        onkeypress={let cb = on_add_device.clone(); Callback::from(move |e: KeyboardEvent| {
                            if e.key() == "Enter" {
                                cb.emit(());
                            }
                        })}
                    />
                    <button
                        onclick={on_add_device.reform(|_| ())}
                        disabled={add_id_input.is_empty()}
                        class="px-4 py-2 bg-blue-600 disabled:opacity-50 disabled:bg-gray-300 dark:disabled:bg-gray-700 text-white rounded-xl font-bold hover:bg-blue-700 transition text-sm"
                    >
                        {"Add"}
                    </button>
                </div>
            </div>

            // Trusted devices list
            { if devices.is_empty() {
                html! {
                    <div class="text-center py-6 text-gray-400 dark:text-gray-500 text-sm">
                        <p>{"No trusted devices yet."}</p>
                        <p class="text-xs mt-1">{"Add a device ID above to start syncing."}</p>
                    </div>
                }
            } else {
                html! {
                    <div class="space-y-2">
                        <div class="flex items-center justify-between">
                            <div class="text-[10px] text-gray-500 uppercase font-bold tracking-wider">
                                {format!("Paired Devices ({})", devices.len())}
                            </div>
                            <button
                                onclick={on_sync_all}
                                class="text-xs text-blue-600 dark:text-blue-400 hover:underline font-bold"
                            >
                                {"Sync All"}
                            </button>
                        </div>
                        { for (*devices).iter().enumerate().map(|(idx, device)| {
                            let is_editing = *editing_idx == Some(idx);
                            let on_remove = on_remove.clone();
                            let on_edit_start = on_edit_start.clone();
                            let on_edit_save = on_edit_save.clone();
                            let edit_name_input = edit_name_input.clone();
                            let editing_idx = editing_idx.clone();

                            html! {
                                <div class="bg-white dark:bg-gray-900 rounded-xl p-3 border border-gray-200 dark:border-gray-700 flex items-center gap-3">
                                    <div class="flex-1 min-w-0">
                                        { if is_editing {
                                            html! {
                                                <div class="flex gap-2">
                                                    <input
                                                        type="text"
                                                        value={(*edit_name_input).clone()}
                                                        class="flex-1 bg-gray-100 dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg py-1 px-2 text-sm outline-none text-gray-900 dark:text-white"
                                                        oninput={let input = edit_name_input.clone(); Callback::from(move |e: InputEvent| {
                                                            let el: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                            input.set(el.value());
                                                        })}
                                                        onkeypress={let cb = on_edit_save.clone(); Callback::from(move |e: KeyboardEvent| {
                                                            if e.key() == "Enter" {
                                                                cb.emit(());
                                                            }
                                                        })}
                                                    />
                                                    <button
                                                        onclick={on_edit_save.reform(|_| ())}
                                                        class="text-xs text-green-600 dark:text-green-400 font-bold"
                                                    >
                                                        {"Save"}
                                                    </button>
                                                    <button
                                                        onclick={let ei = editing_idx.clone(); Callback::from(move |_| ei.set(None))}
                                                        class="text-xs text-gray-400"
                                                    >
                                                        {"Cancel"}
                                                    </button>
                                                </div>
                                            }
                                        } else {
                                            html! {
                                                <>
                                                    <div class="flex items-center gap-2">
                                                        <span class="font-bold text-sm text-gray-900 dark:text-white truncate">{&device.name}</span>
                                                        <button
                                                            onclick={let cb = on_edit_start.clone(); Callback::from(move |_| cb.emit((idx,)))}
                                                            class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 text-xs transition-colors"
                                                            title="Edit name"
                                                        >
                                                            {"✏️"}
                                                        </button>
                                                    </div>
                                                    <div class="text-[10px] text-gray-400 font-mono truncate">{&device.peer_id}</div>
                                                    { if let Some(ref ts) = device.last_synced {
                                                        html! { <div class="text-[10px] text-green-500">{ format!("Last synced: {}", &ts[..10]) }</div> }
                                                    } else {
                                                        html! { <div class="text-[10px] text-gray-400">{"Never synced"}</div> }
                                                    }}
                                                </>
                                            }
                                        }}
                                    </div>
                                    { if !is_editing {
                                        html! {
                                            <button
                                                onclick={Callback::from(move |_| on_remove.emit(idx))}
                                                class="p-2 text-red-400 hover:text-red-600 dark:hover:text-red-300 transition-colors shrink-0"
                                                title="Remove device"
                                            >
                                                {"✕"}
                                            </button>
                                        }
                                    } else {
                                        html! {}
                                    }}
                                </div>
                            }
                        })}
                    </div>
                }
            }}

            <p class="text-xs text-gray-400 dark:text-gray-500 text-center">
                {"Devices sync automatically when both are open. No data stored on servers."}
            </p>
        </div>
    }
}
