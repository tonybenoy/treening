use crate::models;
use crate::storage;
use gloo::storage::Storage as _;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use yew::prelude::*;
use yew::virtual_dom::VNode;

/// Render simple markdown-like text to HTML nodes.
/// Handles: **bold**, paragraphs (double newline), bullet lists (- or *), and numbered lists.
fn render_markdown(text: &str) -> VNode {
    let paragraphs: Vec<&str> = text.split("\n\n").collect();
    let nodes: Vec<VNode> = paragraphs
        .iter()
        .map(|para| {
            let trimmed = para.trim();
            // Check if this paragraph is a list
            let lines: Vec<&str> = trimmed.lines().collect();
            let is_bullet_list = lines.iter().all(|l| {
                let t = l.trim();
                t.starts_with("- ") || t.starts_with("* ") || t.is_empty()
            }) && lines.iter().any(|l| {
                let t = l.trim();
                t.starts_with("- ") || t.starts_with("* ")
            });
            let is_numbered_list = lines.iter().all(|l| {
                let t = l.trim();
                t.is_empty()
                    || t.chars()
                        .next()
                        .map(|c| c.is_ascii_digit())
                        .unwrap_or(false)
                        && t.contains(". ")
            }) && lines.iter().any(|l| {
                let t = l.trim();
                t.chars()
                    .next()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false)
                    && t.contains(". ")
            });

            if is_bullet_list {
                let items: Vec<VNode> = lines
                    .iter()
                    .filter(|l| !l.trim().is_empty())
                    .map(|l| {
                        let content = l.trim().trim_start_matches("- ").trim_start_matches("* ");
                        html! { <li class="ml-4">{render_inline(content)}</li> }
                    })
                    .collect();
                html! { <ul class="list-disc pl-2 space-y-0.5">{items}</ul> }
            } else if is_numbered_list {
                let items: Vec<VNode> = lines
                    .iter()
                    .filter(|l| !l.trim().is_empty())
                    .map(|l| {
                        let content = l.trim();
                        // Strip "1. " prefix
                        let content = if let Some(pos) = content.find(". ") {
                            &content[pos + 2..]
                        } else {
                            content
                        };
                        html! { <li class="ml-4">{render_inline(content)}</li> }
                    })
                    .collect();
                html! { <ol class="list-decimal pl-2 space-y-0.5">{items}</ol> }
            } else {
                // Regular paragraph - handle single newlines as line breaks
                let line_nodes: Vec<VNode> = lines
                    .iter()
                    .enumerate()
                    .flat_map(|(i, line)| {
                        let mut nodes = vec![render_inline(line.trim())];
                        if i < lines.len() - 1 {
                            nodes.push(html! { <br /> });
                        }
                        nodes
                    })
                    .collect();
                html! { <p class="mb-1.5 last:mb-0">{line_nodes}</p> }
            }
        })
        .collect();

    html! { <div class="space-y-2">{nodes}</div> }
}

/// Render inline formatting: **bold** and *italic*
fn render_inline(text: &str) -> VNode {
    let mut nodes: Vec<VNode> = Vec::new();
    let mut remaining = text;

    while !remaining.is_empty() {
        // Look for **bold**
        if let Some(start) = remaining.find("**") {
            if start > 0 {
                nodes.push(html! { <>{&remaining[..start]}</> });
            }
            let after_start = &remaining[start + 2..];
            if let Some(end) = after_start.find("**") {
                nodes.push(html! { <strong class="font-bold">{&after_start[..end]}</strong> });
                remaining = &after_start[end + 2..];
            } else {
                nodes.push(html! { <>{"**"}</> });
                remaining = after_start;
            }
        } else {
            nodes.push(html! { <>{remaining}</> });
            break;
        }
    }

    html! { <>{nodes}</> }
}

// JS bindings for WebLLM wrapper functions
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = webllmIsSupported)]
    fn webllm_is_supported() -> bool;

    #[wasm_bindgen(js_name = webllmInit)]
    fn webllm_init(cb: &JsValue) -> js_sys::Promise;

    #[wasm_bindgen(js_name = webllmChat)]
    fn webllm_chat(sys: &str, msgs: &str) -> js_sys::Promise;

    #[wasm_bindgen(js_name = webllmReset)]
    fn webllm_reset() -> js_sys::Promise;
}

#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

const CHAT_HISTORY_KEY: &str = "treening_ai_chat_history";

fn load_chat_history() -> Vec<ChatMessage> {
    gloo::storage::LocalStorage::get(CHAT_HISTORY_KEY).unwrap_or_default()
}

fn save_chat_history(messages: &[ChatMessage]) {
    let _ = gloo::storage::LocalStorage::set(CHAT_HISTORY_KEY, messages);
}

#[derive(Clone, PartialEq)]
enum ModelState {
    NotLoaded,
    Downloading { progress: f64, text: String },
    Ready,
    Generating,
    Error(String),
    Unsupported,
}

fn build_system_prompt() -> String {
    let config = storage::load_user_config();
    let workouts = storage::load_workouts();
    let body_metrics = storage::load_body_metrics();
    let routines = storage::load_routines();

    let unit = match config.unit_system {
        models::UnitSystem::Metric => "kg/km",
        models::UnitSystem::Imperial => "lbs/mi",
    };

    let total_workouts = workouts.len();
    let total_volume: f64 = workouts.iter().map(|w| w.total_volume()).sum();
    let streak = models::current_streak(&workouts);
    let best_streak = models::best_streak(&workouts);

    // Latest body metrics
    let body_info = if let Some(m) = body_metrics.first() {
        let mut parts = Vec::new();
        if let Some(w) = m.weight {
            parts.push(format!(
                "weight: {:.1}{}",
                config.unit_system.display_weight(w),
                config.unit_system.weight_label()
            ));
        }
        if let Some(bf) = m.body_fat {
            parts.push(format!("body fat: {:.1}%", bf));
        }
        if parts.is_empty() {
            String::new()
        } else {
            format!("Body: {}.", parts.join(", "))
        }
    } else {
        String::new()
    };

    // Top 5 most-trained exercises
    let all_exercises = {
        let mut exs = crate::data::default_exercises();
        exs.extend(storage::load_custom_exercises());
        exs
    };
    let mut exercise_counts: std::collections::HashMap<String, u32> =
        std::collections::HashMap::new();
    for w in &workouts {
        for we in &w.exercises {
            *exercise_counts.entry(we.exercise_id.clone()).or_insert(0) += 1;
        }
    }
    let mut top_exercises: Vec<(String, u32)> = exercise_counts.into_iter().collect();
    top_exercises.sort_by(|a, b| b.1.cmp(&a.1));
    let top_exercises_str: Vec<String> = top_exercises
        .iter()
        .take(5)
        .map(|(id, count)| {
            let name = all_exercises
                .iter()
                .find(|e| e.id == *id)
                .map(|e| e.name.clone())
                .unwrap_or_else(|| id.clone());
            format!("{} ({}x)", name, count)
        })
        .collect();

    // Last 5 workouts summary
    let mut sorted_workouts: Vec<&crate::models::Workout> = workouts.iter().collect();
    sorted_workouts.sort_by(|a, b| b.date.cmp(&a.date));
    let recent: Vec<String> = sorted_workouts
        .iter()
        .take(5)
        .map(|w| {
            let ex_names: Vec<String> = w
                .exercises
                .iter()
                .take(3)
                .map(|we| {
                    all_exercises
                        .iter()
                        .find(|e| e.id == we.exercise_id)
                        .map(|e| e.name.clone())
                        .unwrap_or_else(|| we.exercise_id.clone())
                })
                .collect();
            let dur = if w.duration_mins > 0 {
                format!(", {}min", w.duration_mins)
            } else {
                String::new()
            };
            format!("{}: {}{}", w.date, ex_names.join(", "), dur)
        })
        .collect();

    let routine_names: Vec<String> = routines.iter().map(|r| r.name.clone()).collect();

    let mut prompt = format!(
        "You are a friendly, knowledgeable personal gym coach for {}. \
         You have access to their workout data below.\n\n\
         RULES:\n\
         - Reference their actual data (exercises, volume, streaks, recent workouts) in your answers\n\
         - Be encouraging and celebrate progress, but stay honest\n\
         - Give specific, actionable advice (not generic tips)\n\
         - Use bullet points or numbered lists when listing suggestions\n\
         - Keep responses focused: 3-5 sentences for simple questions, longer for detailed analysis\n\
         - Use their unit system ({})\n\
         - If they have no data yet, welcome them and suggest getting started\n\n\
         USER DATA:\n",
        config.nickname, unit
    );

    if total_workouts > 0 {
        prompt.push_str(&format!(
            "- Total: {} workouts, {:.0}{} volume lifted\n\
             - Current streak: {} days (personal best: {} days)\n",
            total_workouts,
            config.unit_system.display_weight(total_volume),
            config.unit_system.weight_label(),
            streak,
            best_streak
        ));
    } else {
        prompt.push_str("- No workouts logged yet (new user)\n");
    }

    if !body_info.is_empty() {
        prompt.push_str(&format!("- {}\n", body_info));
    }

    if !top_exercises_str.is_empty() {
        prompt.push_str(&format!(
            "- Most trained: {}\n",
            top_exercises_str.join(", ")
        ));
    }

    if !recent.is_empty() {
        prompt.push_str(&format!("- Recent workouts: {}\n", recent.join("; ")));
    }

    if !routine_names.is_empty() {
        prompt.push_str(&format!("- Saved routines: {}\n", routine_names.join(", ")));
    }

    prompt
}

#[function_component(AiChat)]
pub fn ai_chat() -> Html {
    let messages = use_state(load_chat_history);
    let model_state = use_state(|| {
        if webllm_is_supported() {
            ModelState::NotLoaded
        } else {
            ModelState::Unsupported
        }
    });
    let input_text = use_state(String::new);
    let input_ref = use_node_ref();
    let messages_end_ref = use_node_ref();

    // Auto-scroll to bottom when messages change
    {
        let messages_end_ref = messages_end_ref.clone();
        let messages_len = messages.len();
        use_effect_with(messages_len, move |_| {
            if let Some(el) = messages_end_ref.cast::<web_sys::HtmlElement>() {
                el.scroll_into_view();
            }
            || ()
        });
    }

    let on_load_model = {
        let model_state = model_state.clone();
        Callback::from(move |_: MouseEvent| {
            let model_state = model_state.clone();
            model_state.set(ModelState::Downloading {
                progress: 0.0,
                text: "Initializing...".to_string(),
            });
            let ms = model_state.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let progress_cb = {
                    let ms = ms.clone();
                    Closure::wrap(Box::new(move |data: JsValue| {
                        if let Some(s) = data.as_string() {
                            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&s) {
                                let progress =
                                    val.get("progress").and_then(|p| p.as_f64()).unwrap_or(0.0);
                                let text = val
                                    .get("text")
                                    .and_then(|t| t.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                ms.set(ModelState::Downloading { progress, text });
                            }
                        }
                    }) as Box<dyn FnMut(JsValue)>)
                };
                let result =
                    JsFuture::from(webllm_init(progress_cb.as_ref().unchecked_ref())).await;
                progress_cb.forget();
                match result {
                    Ok(_) => ms.set(ModelState::Ready),
                    Err(e) => {
                        let msg = e
                            .as_string()
                            .unwrap_or_else(|| "Failed to load model".to_string());
                        ms.set(ModelState::Error(msg));
                    }
                }
            });
        })
    };

    let on_send = {
        let messages = messages.clone();
        let model_state = model_state.clone();
        let input_text = input_text.clone();
        let input_ref = input_ref.clone();
        Callback::from(move |_: ()| {
            let text = (*input_text).trim().to_string();
            if text.is_empty() {
                return;
            }
            input_text.set(String::new());

            // Add user message
            let mut msgs = (*messages).clone();
            msgs.push(ChatMessage {
                role: "user".to_string(),
                content: text,
            });

            // Trim to last 6 messages (3 turns)
            if msgs.len() > 6 {
                msgs = msgs[msgs.len() - 6..].to_vec();
            }

            save_chat_history(&msgs);
            messages.set(msgs.clone());
            model_state.set(ModelState::Generating);

            let messages = messages.clone();
            let model_state = model_state.clone();
            let input_ref = input_ref.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let system_prompt = build_system_prompt();
                let chat_msgs: Vec<serde_json::Value> = msgs
                    .iter()
                    .map(|m| {
                        serde_json::json!({
                            "role": m.role,
                            "content": m.content
                        })
                    })
                    .collect();
                let msgs_json = serde_json::to_string(&chat_msgs).unwrap_or_default();

                match JsFuture::from(webllm_chat(&system_prompt, &msgs_json)).await {
                    Ok(response) => {
                        let reply = response.as_string().unwrap_or_default();
                        let mut updated = (*messages).clone();
                        updated.push(ChatMessage {
                            role: "assistant".to_string(),
                            content: reply,
                        });
                        if updated.len() > 6 {
                            updated = updated[updated.len() - 6..].to_vec();
                        }
                        save_chat_history(&updated);
                        messages.set(updated);
                        model_state.set(ModelState::Ready);
                    }
                    Err(e) => {
                        let msg = e
                            .as_string()
                            .unwrap_or_else(|| "Generation failed".to_string());
                        model_state.set(ModelState::Error(msg));
                    }
                }

                // Re-focus input
                if let Some(el) = input_ref.cast::<web_sys::HtmlInputElement>() {
                    let _ = el.focus();
                }
            });
        })
    };

    let on_input = {
        let input_text = input_text.clone();
        Callback::from(move |e: InputEvent| {
            let target: web_sys::HtmlInputElement = e.target_unchecked_into();
            input_text.set(target.value());
        })
    };

    let on_keypress = {
        let on_send = on_send.clone();
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" {
                e.prevent_default();
                on_send.emit(());
            }
        })
    };

    let on_send_click = {
        let on_send = on_send.clone();
        Callback::from(move |_: MouseEvent| {
            on_send.emit(());
        })
    };

    let on_quick = {
        let messages = messages.clone();
        let model_state = model_state.clone();
        move |prompt: &'static str| {
            let messages = messages.clone();
            let model_state = model_state.clone();
            Callback::from(move |_: MouseEvent| {
                let mut msgs = (*messages).clone();
                msgs.push(ChatMessage {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                });
                if msgs.len() > 6 {
                    msgs = msgs[msgs.len() - 6..].to_vec();
                }
                save_chat_history(&msgs);
                messages.set(msgs.clone());
                model_state.set(ModelState::Generating);

                let messages = messages.clone();
                let model_state = model_state.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let system_prompt = build_system_prompt();
                    let chat_msgs: Vec<serde_json::Value> = msgs
                        .iter()
                        .map(|m| serde_json::json!({"role": m.role, "content": m.content}))
                        .collect();
                    let msgs_json = serde_json::to_string(&chat_msgs).unwrap_or_default();

                    match JsFuture::from(webllm_chat(&system_prompt, &msgs_json)).await {
                        Ok(response) => {
                            let reply = response.as_string().unwrap_or_default();
                            let mut updated = (*messages).clone();
                            updated.push(ChatMessage {
                                role: "assistant".to_string(),
                                content: reply,
                            });
                            if updated.len() > 6 {
                                updated = updated[updated.len() - 6..].to_vec();
                            }
                            save_chat_history(&updated);
                            messages.set(updated);
                            model_state.set(ModelState::Ready);
                        }
                        Err(e) => {
                            let msg = e
                                .as_string()
                                .unwrap_or_else(|| "Generation failed".to_string());
                            model_state.set(ModelState::Error(msg));
                        }
                    }
                });
            })
        }
    };

    let on_reset = {
        let messages = messages.clone();
        Callback::from(move |_: MouseEvent| {
            save_chat_history(&[]);
            messages.set(Vec::new());
            wasm_bindgen_futures::spawn_local(async move {
                let _ = JsFuture::from(webllm_reset()).await;
            });
        })
    };

    html! {
        <div class="flex flex-col h-[calc(100vh-5rem)]">
            // Header
            <div class="px-4 py-3 flex items-center justify-between border-b border-gray-200 dark:border-gray-700/50">
                <div class="flex items-center gap-2">
                    <span class="text-xl">{"🤖"}</span>
                    <h1 class="text-lg font-bold text-gray-900 dark:text-gray-100">{"AI Assistant"}</h1>
                    <span class="px-2 py-0.5 bg-blue-100 dark:bg-blue-600/20 text-blue-600 dark:text-blue-400 text-[10px] font-bold rounded-full uppercase">{"Local"}</span>
                </div>
                if *model_state == ModelState::Ready || *model_state == ModelState::Generating {
                    <button
                        onclick={on_reset}
                        class="text-xs text-gray-500 hover:text-gray-700 dark:hover:text-gray-300 px-2 py-1 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800 transition"
                    >
                        {"Clear Chat"}
                    </button>
                }
            </div>

            // Main content area
            <div class="flex-1 overflow-y-auto px-4 py-4 space-y-4">
                { match &*model_state {
                    ModelState::Unsupported => html! {
                        <div class="bg-yellow-50 dark:bg-yellow-900/20 rounded-2xl p-6 neu-flat">
                            <div class="text-center space-y-3">
                                <span class="text-3xl">{"⚠️"}</span>
                                <h2 class="text-lg font-bold text-gray-900 dark:text-gray-100">{"WebGPU Not Available"}</h2>
                                <p class="text-sm text-gray-600 dark:text-gray-400">
                                    {"The AI assistant requires WebGPU, which is not supported in your current browser."}
                                </p>
                                <div class="text-xs text-gray-500 space-y-1">
                                    <p class="font-bold">{"Supported browsers:"}</p>
                                    <p>{"Chrome/Edge 113+ (Desktop & Android)"}</p>
                                    <p>{"Safari 18+ (macOS)"}</p>
                                </div>
                            </div>
                        </div>
                    },
                    ModelState::NotLoaded => html! {
                        <div class="space-y-4">
                            <div class="bg-gray-100 dark:bg-gray-800/50 rounded-2xl p-6 neu-flat text-center space-y-4">
                                <span class="text-4xl">{"🤖"}</span>
                                <h2 class="text-lg font-bold text-gray-900 dark:text-gray-100">{"AI Workout Assistant"}</h2>
                                <p class="text-sm text-gray-600 dark:text-gray-400">
                                    {"Ask questions about your workouts, get training advice, and track your progress — all powered by AI running locally in your browser."}
                                </p>
                                <button
                                    onclick={on_load_model}
                                    class="w-full py-3 bg-blue-600 hover:bg-blue-700 active:bg-blue-800 text-white rounded-xl font-bold transition neu-btn btn-press"
                                >
                                    {"Load AI Model (~500MB)"}
                                </button>
                                <p class="text-[10px] text-gray-400">
                                    {"One-time download, cached for offline use. Runs entirely on your device."}
                                </p>
                            </div>

                            <div class="space-y-2">
                                <p class="text-xs font-bold text-gray-500 dark:text-gray-400 px-1">{"Example questions:"}</p>
                                <div class="flex flex-wrap gap-2">
                                    <span class="neu-chip rounded-full px-3 py-1.5 text-xs text-gray-600 dark:text-gray-300 bg-gray-100 dark:bg-gray-800 cursor-default">{"\"How was my week?\""}</span>
                                    <span class="neu-chip rounded-full px-3 py-1.5 text-xs text-gray-600 dark:text-gray-300 bg-gray-100 dark:bg-gray-800 cursor-default">{"\"What should I train today?\""}</span>
                                    <span class="neu-chip rounded-full px-3 py-1.5 text-xs text-gray-600 dark:text-gray-300 bg-gray-100 dark:bg-gray-800 cursor-default">{"\"Am I making progress?\""}</span>
                                </div>
                            </div>
                        </div>
                    },
                    ModelState::Downloading { progress, text } => html! {
                        <div class="bg-gray-100 dark:bg-gray-800/50 rounded-2xl p-6 neu-flat text-center space-y-4">
                            <span class="text-3xl">{"⏳"}</span>
                            <h2 class="text-lg font-bold text-gray-900 dark:text-gray-100">{"Loading AI Model"}</h2>
                            <div class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-3 overflow-hidden">
                                <div
                                    class="bg-blue-600 h-3 rounded-full transition-all duration-300"
                                    style={format!("width: {}%", (progress * 100.0).min(100.0))}
                                />
                            </div>
                            <p class="text-sm font-bold text-gray-700 dark:text-gray-300">
                                {format!("{:.0}%", (progress * 100.0).min(100.0))}
                            </p>
                            <p class="text-xs text-gray-500 truncate">{text.clone()}</p>
                        </div>
                    },
                    ModelState::Error(msg) => html! {
                        <div class="bg-red-50 dark:bg-red-900/20 rounded-2xl p-6 neu-flat text-center space-y-4">
                            <span class="text-3xl">{"❌"}</span>
                            <h2 class="text-lg font-bold text-gray-900 dark:text-gray-100">{"Error"}</h2>
                            <p class="text-sm text-gray-600 dark:text-gray-400">{msg.clone()}</p>
                            <button
                                onclick={on_load_model.clone()}
                                class="px-6 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-xl font-bold transition neu-btn btn-press"
                            >
                                {"Retry"}
                            </button>
                        </div>
                    },
                    ModelState::Ready | ModelState::Generating => html! {
                        <>
                            if messages.is_empty() {
                                <div class="space-y-3 pt-4">
                                    <p class="text-center text-sm text-gray-500">{"Ask me about your workouts!"}</p>
                                    <div class="flex flex-wrap gap-2 justify-center">
                                        <button
                                            onclick={on_quick("How was my week?")}
                                            class="neu-chip rounded-full px-3 py-1.5 text-xs text-gray-600 dark:text-gray-300 bg-gray-100 dark:bg-gray-800 hover:bg-gray-200 dark:hover:bg-gray-700 transition btn-press"
                                        >
                                            {"How was my week?"}
                                        </button>
                                        <button
                                            onclick={on_quick("What should I train today?")}
                                            class="neu-chip rounded-full px-3 py-1.5 text-xs text-gray-600 dark:text-gray-300 bg-gray-100 dark:bg-gray-800 hover:bg-gray-200 dark:hover:bg-gray-700 transition btn-press"
                                        >
                                            {"What should I train today?"}
                                        </button>
                                        <button
                                            onclick={on_quick("Am I making progress?")}
                                            class="neu-chip rounded-full px-3 py-1.5 text-xs text-gray-600 dark:text-gray-300 bg-gray-100 dark:bg-gray-800 hover:bg-gray-200 dark:hover:bg-gray-700 transition btn-press"
                                        >
                                            {"Am I making progress?"}
                                        </button>
                                    </div>
                                </div>
                            }

                            { for messages.iter().map(|msg| {
                                let is_user = msg.role == "user";
                                html! {
                                    <div class={if is_user { "flex justify-end" } else { "flex justify-start" }}>
                                        <div class={format!(
                                            "max-w-[85%] rounded-2xl px-4 py-3 text-sm {}",
                                            if is_user {
                                                "bg-blue-600 text-white rounded-br-md"
                                            } else {
                                                "bg-gray-100 dark:bg-gray-800 text-gray-900 dark:text-gray-100 neu-flat rounded-bl-md"
                                            }
                                        )}>
                                            { if is_user {
                                                html! { <p class="whitespace-pre-wrap">{&msg.content}</p> }
                                            } else {
                                                render_markdown(&msg.content)
                                            }}
                                        </div>
                                    </div>
                                }
                            })}

                            if *model_state == ModelState::Generating {
                                <div class="flex justify-start">
                                    <div class="bg-gray-100 dark:bg-gray-800 rounded-2xl rounded-bl-md px-4 py-3 neu-flat">
                                        <div class="flex gap-1.5 items-center">
                                            <div class="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style="animation-delay: 0ms;" />
                                            <div class="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style="animation-delay: 150ms;" />
                                            <div class="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style="animation-delay: 300ms;" />
                                        </div>
                                    </div>
                                </div>
                            }

                            <div ref={messages_end_ref} />
                        </>
                    },
                }}
            </div>

            // Disclaimer
            <div class="px-4 py-1">
                <p class="text-[10px] text-gray-400 text-center">
                    {"AI responses are generated locally and may not always be accurate."}
                </p>
            </div>

            // Input bar (only show when Ready or Generating)
            if *model_state == ModelState::Ready || *model_state == ModelState::Generating {
                <div class="px-4 py-3 border-t border-gray-200 dark:border-gray-700/50">
                    <div class="flex gap-2 items-center">
                        <input
                            ref={input_ref}
                            type="text"
                            placeholder="Ask about your workouts..."
                            value={(*input_text).clone()}
                            oninput={on_input}
                            onkeypress={on_keypress}
                            disabled={*model_state == ModelState::Generating}
                            class="flex-1 px-4 py-3 bg-gray-100 dark:bg-gray-800 rounded-xl text-sm text-gray-900 dark:text-gray-100 placeholder-gray-400 outline-none focus:ring-2 focus:ring-blue-500 neu-pressed transition disabled:opacity-50"
                        />
                        <button
                            onclick={on_send_click}
                            disabled={*model_state == ModelState::Generating || input_text.is_empty()}
                            class="p-3 bg-blue-600 hover:bg-blue-700 disabled:opacity-50 disabled:hover:bg-blue-600 text-white rounded-xl transition neu-btn btn-press"
                        >
                            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 19V5m-7 7l7-7 7 7" />
                            </svg>
                        </button>
                    </div>
                </div>
            }
        </div>
    }
}
