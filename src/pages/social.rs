use yew::prelude::*;
use yew_router::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use crate::storage;
use crate::models::{Friend, FriendStats};
use qrcode_generator;

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

fn setup_data_handler(conn: &DataConnection, friends: UseStateHandle<Vec<Friend>>) {
    let on_data = Closure::wrap(Box::new(move |data: JsValue| {
        if let Some(msg) = data.as_string() {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&msg) {
                if val.get("type").and_then(|t| t.as_str()) == Some("stats_exchange") {
                    let nickname = val.get("nickname").and_then(|n| n.as_str()).unwrap_or("Friend").to_string();
                    let peer_id = val.get("peer_id").and_then(|p| p.as_str()).unwrap_or("").to_string();
                    if let Some(stats) = val.get("stats") {
                        if let Ok(stats) = serde_json::from_value::<FriendStats>(stats.clone()) {
                            let mut current = (*friends).clone();
                            if let Some(f) = current.iter_mut().find(|f| f.id == peer_id) {
                                if f.name == "New Friend" {
                                    f.name = nickname;
                                }
                                f.last_stats = Some(stats);
                            }
                            storage::save_friends(&current);
                            friends.set(current);
                        }
                    }
                }
            }
        }
    }) as Box<dyn FnMut(JsValue)>);
    conn.on_conn("data", on_data.as_ref().unchecked_ref());
    on_data.forget();
}

fn send_my_stats(conn: &DataConnection, my_stats: &FriendStats, nickname: &str, peer_id: &str) {
    let share_data = serde_json::json!({
        "type": "stats_exchange",
        "nickname": nickname,
        "peer_id": peer_id,
        "stats": my_stats
    }).to_string();
    conn.send(&share_data);
}

#[function_component(SocialPage)]
pub fn social_page() -> Html {
    let user_config = use_state(storage::load_user_config);
    let navigator = use_navigator().unwrap();

    if !user_config.social_enabled {
        navigator.replace(&crate::Route::Home);
        return html! {};
    }

    let friends = use_state(storage::load_friends);
    let show_add_friend = use_state(|| false);
    let friend_id_input = use_state(String::new);
    let status = use_state(|| "Connecting...".to_string());
    let editing_friend = use_state(|| None::<String>);
    let edit_name_input = use_state(String::new);

    let peer_ref = use_mut_ref(|| None::<Peer>);

    let my_stats = {
        let workouts = storage::load_workouts();
        let now = chrono::Local::now();
        let week_ago = now - chrono::Duration::days(7);

        let this_week: Vec<_> = workouts.iter()
            .filter(|w| {
                if let Ok(dt) = chrono::NaiveDate::parse_from_str(&w.date, "%Y-%m-%d") {
                    dt >= week_ago.date_naive()
                } else { false }
            }).collect();

        let total_volume: f64 = this_week.iter()
            .map(|w| w.total_volume())
            .sum();

        let latest_weight = storage::load_body_metrics().first().and_then(|m| m.weight);

        FriendStats {
            workouts_this_week: this_week.len() as u32,
            total_volume_kg: total_volume,
            last_active: now.format("%Y-%m-%d").to_string(),
            body_weight: latest_weight,
        }
    };

    // Initialize Peer and connect to friends
    {
        let status = status.clone();
        let my_stats = my_stats.clone();
        let user_config = user_config.clone();
        let peer_ref = peer_ref.clone();
        let friends = friends.clone();

        use_effect_with((), move |_| {
            // Handle Auto-Join from URL
            let window = gloo::utils::window();
            if let Ok(search) = window.location().search() {
                let params = web_sys::UrlSearchParams::new_with_str(&search).unwrap();
                if let Some(friend_id) = params.get("join") {
                    let mut current = storage::load_friends();
                    if !current.iter().any(|f| f.id == friend_id) {
                        current.push(Friend {
                            id: friend_id.clone(),
                            name: "New Friend".to_string(),
                            last_stats: None,
                        });
                        storage::save_friends(&current);
                        friends.set(current);
                    }
                }
            }

            let peer = Peer::new(Some(&user_config.peer_id));

            // On peer open: set status and connect to all friends
            let status_c = status.clone();
            let friends_for_connect = friends.clone();
            let my_stats_for_connect = my_stats.clone();
            let user_config_for_connect = user_config.clone();
            let peer_clone = peer.clone();

            let on_open = Closure::wrap(Box::new(move |_id: String| {
                status_c.set("Online".to_string());

                // Actively connect to each friend
                let current_friends = (*friends_for_connect).clone();
                for f in &current_friends {
                    let conn = peer_clone.connect(&f.id);
                    let stats = my_stats_for_connect.clone();
                    let nickname = user_config_for_connect.nickname.clone();
                    let pid = user_config_for_connect.peer_id.clone();
                    let conn_c = conn.clone();
                    let friends_c = friends_for_connect.clone();

                    // When connection opens, send our stats
                    let on_open_conn = Closure::wrap(Box::new(move || {
                        send_my_stats(&conn_c, &stats, &nickname, &pid);
                    }) as Box<dyn FnMut()>);
                    conn.on_conn("open", on_open_conn.as_ref().unchecked_ref());
                    on_open_conn.forget();

                    // Listen for their stats back
                    setup_data_handler(&conn, friends_c);
                }
            }) as Box<dyn FnMut(String)>);
            peer.on("open", on_open.as_ref().unchecked_ref());
            on_open.forget();

            // Handle incoming connections (friends connecting to us)
            let my_stats_incoming = my_stats.clone();
            let user_config_incoming = user_config.clone();
            let friends_incoming = friends.clone();

            let on_connection = Closure::wrap(Box::new(move |conn: DataConnection| {
                let stats = my_stats_incoming.clone();
                let nickname = user_config_incoming.nickname.clone();
                let pid = user_config_incoming.peer_id.clone();
                let conn_c = conn.clone();

                // When connection opens, send our stats
                let on_open_conn = Closure::wrap(Box::new(move || {
                    send_my_stats(&conn_c, &stats, &nickname, &pid);
                }) as Box<dyn FnMut()>);
                conn.on_conn("open", on_open_conn.as_ref().unchecked_ref());
                on_open_conn.forget();

                // Listen for their stats
                setup_data_handler(&conn, friends_incoming.clone());
            }) as Box<dyn FnMut(DataConnection)>);
            peer.on("connection", on_connection.as_ref().unchecked_ref());
            on_connection.forget();

            // Handle errors
            let status_err = status.clone();
            let on_error = Closure::wrap(Box::new(move |_err: JsValue| {
                status_err.set("Connection error".to_string());
            }) as Box<dyn FnMut(JsValue)>);
            peer.on("error", on_error.as_ref().unchecked_ref());
            on_error.forget();

            *peer_ref.borrow_mut() = Some(peer);

            move || {
                if let Some(p) = peer_ref.borrow_mut().take() {
                    p.destroy();
                }
            }
        });
    }

    let on_add_friend = {
        let friends_state = friends.clone();
        let friend_id_input = friend_id_input.clone();
        let show_add = show_add_friend.clone();
        let peer_ref = peer_ref.clone();
        let my_stats = my_stats.clone();
        let user_config = user_config.clone();
        Callback::from(move |_| {
            let id = (*friend_id_input).trim().to_string();
            if id.is_empty() || id == user_config.peer_id { return; }

            let mut current = (*friends_state).clone();
            if !current.iter().any(|f| f.id == id) {
                current.push(Friend {
                    id: id.clone(),
                    name: "New Friend".to_string(),
                    last_stats: None,
                });
                storage::save_friends(&current);
                friends_state.set(current);

                // Immediately try to connect to the new friend
                if let Some(peer) = peer_ref.borrow().as_ref() {
                    let conn = peer.connect(&id);
                    let stats = my_stats.clone();
                    let nickname = user_config.nickname.clone();
                    let pid = user_config.peer_id.clone();
                    let conn_c = conn.clone();
                    let friends_c = friends_state.clone();

                    let on_open_conn = Closure::wrap(Box::new(move || {
                        send_my_stats(&conn_c, &stats, &nickname, &pid);
                    }) as Box<dyn FnMut()>);
                    conn.on_conn("open", on_open_conn.as_ref().unchecked_ref());
                    on_open_conn.forget();

                    setup_data_handler(&conn, friends_c);
                }
            }
            show_add.set(false);
            friend_id_input.set(String::new());
        })
    };

    let on_remove_friend = {
        let friends_state = friends.clone();
        Callback::from(move |id: String| {
            let mut current = (*friends_state).clone();
            current.retain(|f| f.id != id);
            storage::save_friends(&current);
            friends_state.set(current);
        })
    };

    let on_save_edit = {
        let friends_state = friends.clone();
        let editing = editing_friend.clone();
        let edit_input = edit_name_input.clone();
        Callback::from(move |_: ()| {
            if let Some(id) = &*editing {
                let new_name = (*edit_input).trim().to_string();
                if !new_name.is_empty() {
                    let mut current = (*friends_state).clone();
                    if let Some(f) = current.iter_mut().find(|f| &f.id == id) {
                        f.name = new_name;
                    }
                    storage::save_friends(&current);
                    friends_state.set(current);
                }
                editing.set(None);
            }
        })
    };

    let render_qr = |data: &str| {
        let result: Result<String, _> = qrcode_generator::to_svg_to_string(data, qrcode_generator::QrCodeEcc::Low, 400, None::<&str>);
        match result {
            Ok(svg) => {
                let base64 = gloo::utils::window().btoa(&svg).unwrap_or_default();
                html! {
                    <div class="bg-white p-2 rounded-lg inline-block shadow-lg">
                        <img src={format!("data:image/svg+xml;base64,{}", base64)} class="w-32 h-32" />
                    </div>
                }
            }
            Err(_) => html! { <div>{"QR Error"}</div> }
        }
    };

    html! {
        <div class="px-4 py-4 pb-24 space-y-6">
            <div class="flex justify-between items-center">
                <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">{"Community"}</h1>
                <div class={classes!(
                    "text-[10px]", "uppercase", "font-bold", "px-2", "py-1", "rounded-full", "transition-colors",
                    if *status == "Online" {
                        "bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-400"
                    } else if status.contains("error") {
                        "bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-400"
                    } else {
                        "bg-yellow-100 dark:bg-yellow-900/30 text-yellow-700 dark:text-yellow-400"
                    }
                )}>
                    {&*status}
                </div>
            </div>

            // My Profile / Stats
            <div class="bg-blue-600 rounded-2xl p-4 text-white shadow-xl shadow-blue-900/20">
                <div class="flex justify-between items-start mb-4">
                    <div>
                        <div class="text-xs opacity-70 uppercase font-bold tracking-wider">{"My Stats (This Week)"}</div>
                        <input
                            class="text-xl font-bold bg-transparent border-none outline-none focus:ring-0 p-0 w-full text-white placeholder-white/50"
                            value={user_config.nickname.clone()}
                            onchange={let uc = user_config.clone(); Callback::from(move |e: Event| {
                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                let mut config = (*uc).clone();
                                config.nickname = input.value();
                                storage::save_user_config(&config);
                                uc.set(config);
                            })}
                        />
                    </div>
                    {render_qr(&format!("https://tonybenoy.github.io/treening/#/social?join={}", user_config.peer_id))}
                </div>

                <div class="grid grid-cols-2 gap-4">
                    <div class="bg-white/10 rounded-xl p-3">
                        <div class="text-[10px] opacity-70 uppercase font-bold">{"Workouts"}</div>
                        <div class="text-xl font-bold">{my_stats.workouts_this_week}</div>
                    </div>
                    <div class="bg-white/10 rounded-xl p-3">
                        <div class="text-[10px] opacity-70 uppercase font-bold">{"Vol (kg)"}</div>
                        <div class="text-xl font-bold">{format!("{:.0}", my_stats.total_volume_kg)}</div>
                    </div>
                </div>
                <div class="mt-3 flex items-center justify-between">
                    <div class="text-[10px] opacity-50 font-mono">
                        {"Friend Code: "}{&user_config.peer_id}
                    </div>
                    <button
                        onclick={let id = user_config.peer_id.clone(); Callback::from(move |_| {
                            let window = gloo::utils::window();
                            let _ = window.navigator().clipboard().write_text(&id);
                        })}
                        class="text-[10px] bg-white/10 px-2 py-1 rounded hover:bg-white/20 transition"
                    >
                        {"Copy Code"}
                    </button>
                </div>
            </div>

            // Ranking / Friends List
            <div class="space-y-4">
                <div class="flex justify-between items-center px-1">
                    <h2 class="text-lg font-bold text-gray-900 dark:text-gray-100">{"Friend Ranking"}</h2>
                    <button
                        onclick={let s = show_add_friend.clone(); Callback::from(move |_| s.set(!*s))}
                        class="text-sm px-3 py-1 bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300 rounded-full hover:bg-gray-200 dark:hover:bg-gray-700 border border-gray-200 dark:border-transparent transition-colors"
                    >
                        {if *show_add_friend { "Cancel" } else { "+ Add Friend" }}
                    </button>
                </div>

                { if *show_add_friend {
                    html! {
                        <div class="bg-gray-100 dark:bg-gray-800 p-4 rounded-xl space-y-3 border border-blue-500/30 transition-colors">
                            <input
                                type="text"
                                placeholder="Enter Friend Code"
                                class="w-full bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-700 rounded-lg p-2 text-sm font-mono outline-none focus:border-blue-500 text-gray-900 dark:text-white"
                                oninput={let input = friend_id_input.clone(); Callback::from(move |e: InputEvent| {
                                    let i: web_sys::HtmlInputElement = e.target_unchecked_into();
                                    input.set(i.value());
                                })}
                            />
                            <button
                                onclick={on_add_friend}
                                class="w-full py-2 bg-blue-600 text-white rounded-lg font-bold text-sm hover:bg-blue-700 transition-colors"
                            >{"Add Friend"}</button>
                        </div>
                    }
                } else { html! {} }}

                <div class="space-y-2">
                    { if friends.is_empty() {
                        html! {
                            <div class="text-center py-12 bg-gray-50 dark:bg-gray-800/20 rounded-2xl border border-dashed border-gray-200 dark:border-gray-700 transition-colors">
                                <p class="text-sm text-gray-500 mb-1">{"No friends added yet."}</p>
                                <p class="text-xs text-gray-600 dark:text-gray-500">{"Share your Friend Code or scan a friend's QR to get started."}</p>
                            </div>
                        }
                    } else {
                        html! {
                            { for friends.iter().map(|f| {
                                let fid2 = f.id.clone();
                                let fid3 = f.id.clone();
                                let is_editing = *editing_friend == Some(f.id.clone());
                                let on_remove = on_remove_friend.clone();
                                let editing = editing_friend.clone();
                                let edit_input = edit_name_input.clone();
                                let on_save = on_save_edit.clone();
                                let fname = f.name.clone();
                                let initial = f.name.chars().next().unwrap_or('?').to_uppercase().to_string();

                                html! {
                                    <div class="bg-gray-100 dark:bg-gray-800 p-3 rounded-xl border border-gray-200 dark:border-transparent transition-colors">
                                        <div class="flex justify-between items-center">
                                            <div class="flex items-center gap-3">
                                                <div class="w-10 h-10 bg-blue-100 dark:bg-gray-700 text-blue-600 dark:text-gray-100 rounded-full flex items-center justify-center font-bold flex-shrink-0">
                                                    {initial}
                                                </div>
                                                <div class="min-w-0">
                                                    { if is_editing {
                                                        html! {
                                                            <input
                                                                type="text"
                                                                class="bg-white dark:bg-gray-900 border border-gray-300 dark:border-gray-600 rounded px-2 py-0.5 text-sm font-bold text-gray-800 dark:text-gray-100 outline-none focus:border-blue-500 w-full"
                                                                value={(*edit_input).clone()}
                                                                oninput={let ei = edit_input.clone(); Callback::from(move |e: InputEvent| {
                                                                    let i: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                                    ei.set(i.value());
                                                                })}
                                                                onkeypress={let s = on_save.clone(); Callback::from(move |e: KeyboardEvent| {
                                                                    if e.key() == "Enter" { s.emit(()); }
                                                                })}
                                                            />
                                                        }
                                                    } else {
                                                        html! {
                                                            <div class="font-bold text-gray-800 dark:text-gray-100 truncate">{&f.name}</div>
                                                        }
                                                    }}
                                                    <div class="text-[10px] text-gray-500 dark:text-gray-500 font-mono truncate">{&f.id}</div>
                                                </div>
                                            </div>
                                            <div class="text-right flex-shrink-0">
                                                { if let Some(stats) = &f.last_stats {
                                                    let vol_display = if let Some(bw) = stats.body_weight {
                                                        if bw > 0.0 {
                                                            format!("{:.1}x BW", stats.total_volume_kg / bw)
                                                        } else {
                                                            format!("{:.0} kg", stats.total_volume_kg)
                                                        }
                                                    } else {
                                                        format!("{:.0} kg", stats.total_volume_kg)
                                                    };
                                                    html! {
                                                        <>
                                                            <div class="text-xs font-bold text-blue-600 dark:text-blue-400">{vol_display}</div>
                                                            <div class="text-[10px] text-gray-500">{stats.workouts_this_week}{" workouts"}</div>
                                                        </>
                                                    }
                                                } else {
                                                    html! {
                                                        <div class="text-xs font-bold text-gray-400 dark:text-gray-500">{"Waiting..."}</div>
                                                    }
                                                }}
                                            </div>
                                        </div>
                                        // Edit/Remove buttons
                                        <div class="flex justify-end gap-2 mt-2 pt-2 border-t border-gray-200 dark:border-gray-700">
                                            { if is_editing {
                                                html! {
                                                    <>
                                                        <button
                                                            onclick={let s = on_save.clone(); Callback::from(move |_| s.emit(()))}
                                                            class="text-[11px] text-blue-600 dark:text-blue-400 font-bold hover:underline"
                                                        >{"Save"}</button>
                                                        <button
                                                            onclick={let ed = editing.clone(); Callback::from(move |_| ed.set(None))}
                                                            class="text-[11px] text-gray-500 font-bold hover:underline"
                                                        >{"Cancel"}</button>
                                                    </>
                                                }
                                            } else {
                                                html! {
                                                    <>
                                                        <button
                                                            onclick={let ed = editing.clone(); let ei = edit_input.clone(); Callback::from(move |_| {
                                                                ei.set(fname.clone());
                                                                ed.set(Some(fid2.clone()));
                                                            })}
                                                            class="text-[11px] text-gray-500 dark:text-gray-400 font-bold hover:underline"
                                                        >{"Edit"}</button>
                                                        <button
                                                            onclick={Callback::from(move |_| on_remove.emit(fid3.clone()))}
                                                            class="text-[11px] text-red-500 dark:text-red-400 font-bold hover:underline"
                                                        >{"Remove"}</button>
                                                    </>
                                                }
                                            }}
                                        </div>
                                    </div>
                                }
                            })}
                        }
                    }}
                </div>
            </div>

            // Explanation
            <div class="bg-gray-50 dark:bg-gray-900/50 p-4 rounded-xl border border-gray-200 dark:border-gray-800 transition-colors">
                <h3 class="text-xs font-bold uppercase tracking-widest text-gray-500 mb-2">{"How it works"}</h3>
                <p class="text-[11px] text-gray-500 leading-relaxed">
                    {"Social features are Peer-to-Peer. Your weekly stats are exchanged directly with friends when you both have the app open. No data is stored on any server."}
                </p>
            </div>
        </div>
    }
}
