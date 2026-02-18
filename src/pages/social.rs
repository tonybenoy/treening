use yew::prelude::*;
use yew_router::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use crate::storage;
use crate::models::{Friend, FriendStats};
use qrcode_generator;

// Re-using the PeerJS definitions from sync.rs or ideally moving them to a shared place
// For now, we'll redefine the minimum needed for Social
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

#[function_component(SocialPage)]
pub fn social_page() -> Html {
    let user_config = use_state(|| storage::load_user_config());
    let navigator = use_navigator().unwrap();

    // Redirect if social is disabled
    if !user_config.social_enabled {
        navigator.replace(&crate::Route::Home);
        return html! {};
    }

    let friends = use_state(|| storage::load_friends());
    let show_add_friend = use_state(|| false);
    let friend_id_input = use_state(|| String::new());
    let status = use_state(|| "Offline".to_string());
    
    let peer_ref = use_mut_ref(|| None::<Peer>);

    // Calculate our own stats for sharing
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
            .map(|w| w.exercises.iter()
                .map(|e| e.sets.iter().map(|s| s.weight * s.reps as f64).sum::<f64>())
                .sum::<f64>()
            ).sum();

        FriendStats {
            workouts_this_week: this_week.len() as u32,
            total_volume_kg: total_volume,
            last_active: now.format("%Y-%m-%d").to_string(),
        }
    };

    // Initialize Peer for social presence
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
            let status_c = status.clone();
            let my_stats_c = my_stats.clone();
            let user_config_c = user_config.clone();

            let on_open = Closure::wrap(Box::new(move |id: String| {
                status_c.set(format!("Online as {}", id));
            }) as Box<dyn FnMut(String)>);
            peer.on("open", on_open.as_ref().unchecked_ref());
            on_open.forget();

            // Handle incoming requests for stats
            let on_connection = Closure::wrap(Box::new(move |conn: DataConnection| {
                let stats = my_stats_c.clone();
                let name = user_config_c.nickname.clone();
                let conn_c = conn.clone();
                let on_open_conn = Closure::wrap(Box::new(move || {
                    let share_data = serde_json::json!({
                        "type": "stats_exchange",
                        "nickname": name,
                        "stats": stats
                    }).to_string();
                    conn_c.send(&share_data);
                }) as Box<dyn FnMut()>);
                conn.on_conn("open", on_open_conn.as_ref().unchecked_ref());
                on_open_conn.forget();
            }) as Box<dyn FnMut(DataConnection)>);
            peer.on("connection", on_connection.as_ref().unchecked_ref());
            on_connection.forget();

            *peer_ref.borrow_mut() = Some(peer);
            
            move || {
                // Cleanup peer on unmount? PeerJS doesn't always like rapid re-connections
            }
        });
    }

    let on_add_friend = {
        let friends_state = friends.clone();
        let friend_id_input = friend_id_input.clone();
        let show_add = show_add_friend.clone();
        Callback::from(move |_| {
            let id = (*friend_id_input).clone();
            if id.is_empty() { return; }
            
            let mut current = (*friends_state).clone();
            if !current.iter().any(|f| f.id == id) {
                current.push(Friend {
                    id: id.clone(),
                    name: "New Friend".to_string(),
                    last_stats: None,
                });
                storage::save_friends(&current);
                friends_state.set(current);
            }
            show_add.set(false);
            friend_id_input.set(String::new());
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
                <div class="text-[10px] uppercase font-bold px-2 py-1 bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-400 rounded-full transition-colors">
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
                        {"Copy Link"}
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
                                <span class="text-3xl block mb-2">{"👋"}</span>
                                <p class="text-sm text-gray-500">{"No friends added yet."}</p>
                                <p class="text-xs text-gray-600 dark:text-gray-500">{"Share your code or add a friend to see rankings."}</p>
                            </div>
                        }
                    } else {
                        html! {
                            { for friends.iter().map(|f| {
                                html! {
                                    <div class="bg-gray-100 dark:bg-gray-800 p-3 rounded-xl flex justify-between items-center border border-gray-200 dark:border-transparent transition-colors">
                                        <div class="flex items-center gap-3">
                                            <div class="w-10 h-10 bg-blue-100 dark:bg-gray-700 text-blue-600 dark:text-gray-100 rounded-full flex items-center justify-center font-bold">
                                                {&f.name[..1]}
                                            </div>
                                            <div>
                                                <div class="font-bold text-gray-800 dark:text-gray-100">{&f.name}</div>
                                                <div class="text-[10px] text-gray-500 dark:text-gray-500 font-mono">{&f.id}</div>
                                            </div>
                                        </div>
                                        <div class="text-right">
                                            <div class="text-xs font-bold text-blue-600 dark:text-blue-400">{"Offline"}</div>
                                            <div class="text-[10px] text-gray-500">{"Last seen: -"}</div>
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
                    {"Social features are purely Peer-to-Peer. Your stats are only visible to friends when you both have the app open at the same time. No data is ever stored on a social server."}
                </p>
            </div>
        </div>
    }
}
