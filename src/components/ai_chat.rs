use crate::models;
use crate::storage;
use chrono::Datelike;
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
    fn webllm_init(model_id: &str, cb: &JsValue) -> js_sys::Promise;

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
    use std::collections::HashMap;

    let config = storage::load_user_config();
    let workouts = storage::load_workouts();
    let body_metrics = storage::load_body_metrics();
    let routines = storage::load_routines();

    let us = &config.unit_system;
    let wl = us.weight_label();

    // All exercises for lookups
    let all_exercises = {
        let mut exs = crate::data::default_exercises();
        exs.extend(storage::load_custom_exercises());
        exs
    };
    let find_ex =
        |id: &str| -> Option<&models::Exercise> { all_exercises.iter().find(|e| e.id == id) };
    let ex_name = |id: &str| -> String {
        find_ex(id)
            .map(|e| e.name.clone())
            .unwrap_or_else(|| id.to_string())
    };

    let total_workouts = workouts.len();
    let total_volume: f64 = workouts.iter().map(|w| w.total_volume()).sum();
    let streak = models::current_streak(&workouts);
    let best_streak = models::best_streak(&workouts);

    // Sort workouts by date (newest first)
    let mut sorted: Vec<&crate::models::Workout> = workouts.iter().collect();
    sorted.sort_by(|a, b| b.date.cmp(&a.date));

    // --- Body metrics (latest + trend) ---
    let mut body_str = String::new();
    if !body_metrics.is_empty() {
        let latest = &body_metrics[0];
        if let Some(w) = latest.weight {
            body_str.push_str(&format!(
                "Current weight: {:.1}{} ({})",
                us.display_weight(w),
                wl,
                latest.date
            ));
            // Find oldest weight for trend
            if body_metrics.len() > 1 {
                if let Some(oldest_w) = body_metrics.last().and_then(|m| m.weight) {
                    let diff = w - oldest_w;
                    let sign = if diff > 0.0 { "+" } else { "" };
                    body_str.push_str(&format!(
                        ", change: {}{:.1}{}",
                        sign,
                        us.display_weight(diff),
                        wl
                    ));
                }
            }
        }
        if let Some(bf) = latest.body_fat {
            if !body_str.is_empty() {
                body_str.push_str(", ");
            }
            body_str.push_str(&format!("body fat: {:.1}%", bf));
        }
    }

    // --- PRs for every exercise (all-time best weight) ---
    let mut prs: HashMap<String, (f64, u32, String)> = HashMap::new(); // id -> (weight, reps, date)
    for w in &workouts {
        for we in &w.exercises {
            for s in &we.sets {
                if s.completed && s.weight > 0.0 {
                    let entry =
                        prs.entry(we.exercise_id.clone())
                            .or_insert((0.0, 0, String::new()));
                    if s.weight > entry.0 {
                        *entry = (s.weight, s.reps, w.date.clone());
                    }
                }
            }
        }
    }
    let mut pr_list: Vec<(String, f64, u32, String)> = prs
        .into_iter()
        .map(|(id, (w, r, d))| (id, w, r, d))
        .collect();
    pr_list.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let pr_str: Vec<String> = pr_list
        .iter()
        .take(15)
        .map(|(id, w, r, d)| {
            format!(
                "{}: {:.1}{}x{} ({})",
                ex_name(id),
                us.display_weight(*w),
                wl,
                r,
                d
            )
        })
        .collect();

    // --- Volume by muscle group (all-time + this week) ---
    let today = chrono::Local::now().date_naive();
    let week_start = today - chrono::Duration::days(today.weekday().num_days_from_monday() as i64);

    let mut muscle_total: HashMap<String, f64> = HashMap::new();
    let mut muscle_this_week: HashMap<String, f64> = HashMap::new();
    let mut muscle_last_trained: HashMap<String, String> = HashMap::new();
    for w in &sorted {
        let w_date = chrono::NaiveDate::parse_from_str(&w.date, "%Y-%m-%d").ok();
        for we in &w.exercises {
            if let Some(ex) = find_ex(&we.exercise_id) {
                let cat = ex.category.to_string();
                let vol = we.volume();
                *muscle_total.entry(cat.clone()).or_insert(0.0) += vol;
                if let Some(wd) = w_date {
                    if wd >= week_start {
                        *muscle_this_week.entry(cat.clone()).or_insert(0.0) += vol;
                    }
                }
                muscle_last_trained
                    .entry(cat)
                    .or_insert_with(|| w.date.clone());
            }
        }
    }
    let muscle_breakdown: Vec<String> = {
        let mut items: Vec<(String, f64)> =
            muscle_total.iter().map(|(k, v)| (k.clone(), *v)).collect();
        items.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        items
            .iter()
            .map(|(group, total)| {
                let week = muscle_this_week.get(group).copied().unwrap_or(0.0);
                let last = muscle_last_trained.get(group).cloned().unwrap_or_default();
                format!(
                    "{}: {:.0}{} total, {:.0}{} this week, last: {}",
                    group,
                    us.display_weight(*total),
                    wl,
                    us.display_weight(week),
                    wl,
                    last
                )
            })
            .collect()
    };

    // --- Exercise frequency + progression (first vs latest weight) ---
    let mut ex_stats: HashMap<String, (u32, f64, f64, String, String)> = HashMap::new(); // id -> (count, first_weight, latest_weight, first_date, latest_date)
    for w in sorted.iter().rev() {
        // iterate oldest first for first/latest tracking
        for we in &w.exercises {
            let best_weight = we
                .sets
                .iter()
                .filter(|s| s.completed && s.weight > 0.0)
                .map(|s| s.weight)
                .fold(0.0f64, f64::max);
            let entry = ex_stats.entry(we.exercise_id.clone()).or_insert((
                0,
                best_weight,
                best_weight,
                w.date.clone(),
                w.date.clone(),
            ));
            entry.0 += 1;
            if best_weight > 0.0 {
                entry.2 = best_weight; // latest (we iterate oldest->newest)
                entry.4 = w.date.clone();
            }
        }
    }
    let ex_progression: Vec<String> = {
        let mut items: Vec<(String, u32, f64, f64)> = ex_stats
            .iter()
            .map(|(id, (count, first, latest, _, _))| (id.clone(), *count, *first, *latest))
            .collect();
        items.sort_by(|a, b| b.1.cmp(&a.1));
        items
            .iter()
            .take(15)
            .map(|(id, count, first, latest)| {
                let cat = find_ex(id)
                    .map(|e| e.category.to_string())
                    .unwrap_or_default();
                if *first > 0.0 && *latest > 0.0 {
                    let diff = latest - first;
                    let sign = if diff > 0.0 { "+" } else { "" };
                    format!(
                        "{} [{}]: {}x, {:.1}{} -> {:.1}{} ({}{:.1}{})",
                        ex_name(id),
                        cat,
                        count,
                        us.display_weight(*first),
                        wl,
                        us.display_weight(*latest),
                        wl,
                        sign,
                        us.display_weight(diff),
                        wl,
                    )
                } else {
                    format!("{} [{}]: {}x", ex_name(id), cat, count)
                }
            })
            .collect()
    };

    // --- Weekly volume trend (last 4 weeks) ---
    let mut weekly_volumes: Vec<(String, f64, usize)> = Vec::new(); // (week_label, volume, session_count)
    for i in 0..4 {
        let ws = week_start - chrono::Duration::weeks(i);
        let we = ws + chrono::Duration::days(6);
        let mut vol = 0.0;
        let mut count = 0;
        for w in &sorted {
            if let Ok(wd) = chrono::NaiveDate::parse_from_str(&w.date, "%Y-%m-%d") {
                if wd >= ws && wd <= we {
                    vol += w.total_volume();
                    count += 1;
                }
            }
        }
        let label = if i == 0 {
            "This week".to_string()
        } else if i == 1 {
            "Last week".to_string()
        } else {
            format!("{}w ago", i)
        };
        weekly_volumes.push((label, vol, count));
    }

    // --- Last 7 workouts with full detail ---
    let recent: Vec<String> = sorted
        .iter()
        .take(7)
        .map(|w| {
            let exercises: Vec<String> = w
                .exercises
                .iter()
                .map(|we| {
                    let name = ex_name(&we.exercise_id);
                    let completed_sets: Vec<String> = we
                        .sets
                        .iter()
                        .filter(|s| s.completed)
                        .map(|s| {
                            if s.weight > 0.0 {
                                format!("{:.0}{}x{}", us.display_weight(s.weight), wl, s.reps)
                            } else if let Some(d) = s.duration_secs {
                                format!("{}s", d)
                            } else if s.reps > 0 {
                                format!("x{}", s.reps)
                            } else {
                                "done".to_string()
                            }
                        })
                        .collect();
                    if completed_sets.is_empty() {
                        name
                    } else {
                        format!("{}: {}", name, completed_sets.join(", "))
                    }
                })
                .collect();
            let dur = if w.duration_mins > 0 {
                format!(" ({}min)", w.duration_mins)
            } else {
                String::new()
            };
            format!(
                "  {} \"{}\"{}: {}",
                w.date,
                w.name,
                dur,
                exercises.join(" | ")
            )
        })
        .collect();

    // --- Routines with exercises ---
    let routine_info: Vec<String> = routines
        .iter()
        .map(|r| {
            let ex_names: Vec<String> = r.exercise_ids.iter().map(|id| ex_name(id)).collect();
            format!("{}: {}", r.name, ex_names.join(", "))
        })
        .collect();

    // === Build prompt ===
    let mut prompt = format!(
        "You are a friendly, knowledgeable personal gym coach for {}. \
         You have COMPLETE access to their workout data below. \
         Use this data to give accurate, specific answers.\n\n\
         RULES:\n\
         - ALWAYS reference their actual numbers (weights, sets, dates, muscle groups)\n\
         - Be encouraging and celebrate progress, but stay honest\n\
         - Give specific, actionable advice based on their data\n\
         - Use bullet points or numbered lists when helpful\n\
         - Keep responses focused but thorough\n\
         - Units: {}\n\
         - If they have no data yet, welcome them and suggest getting started\n\n\
         === USER DATA ===\n",
        config.nickname,
        if *us == models::UnitSystem::Metric {
            "kg/km"
        } else {
            "lbs/mi"
        }
    );

    // Overview
    if total_workouts > 0 {
        prompt.push_str(&format!(
            "OVERVIEW: {} workouts, {:.0}{} total volume, streak: {}d (best: {}d)\n",
            total_workouts,
            us.display_weight(total_volume),
            wl,
            streak,
            best_streak
        ));
    } else {
        prompt.push_str("OVERVIEW: New user, no workouts yet\n");
    }

    // Body
    if !body_str.is_empty() {
        prompt.push_str(&format!("BODY: {}\n", body_str));
    }

    // Weekly trend
    if weekly_volumes.iter().any(|(_, v, _)| *v > 0.0) {
        let trend: Vec<String> = weekly_volumes
            .iter()
            .map(|(label, vol, count)| {
                format!(
                    "{}: {:.0}{} ({}sessions)",
                    label,
                    us.display_weight(*vol),
                    wl,
                    count
                )
            })
            .collect();
        prompt.push_str(&format!("WEEKLY TREND: {}\n", trend.join(", ")));
    }

    // Muscle groups
    if !muscle_breakdown.is_empty() {
        prompt.push_str("MUSCLE GROUPS:\n");
        for m in &muscle_breakdown {
            prompt.push_str(&format!("  {}\n", m));
        }
    }

    // PRs
    if !pr_str.is_empty() {
        prompt.push_str("PERSONAL RECORDS (best weight):\n");
        for p in &pr_str {
            prompt.push_str(&format!("  {}\n", p));
        }
    }

    // Exercise progression
    if !ex_progression.is_empty() {
        prompt.push_str("EXERCISE PROGRESSION (first->latest):\n");
        for e in &ex_progression {
            prompt.push_str(&format!("  {}\n", e));
        }
    }

    // Recent workouts
    if !recent.is_empty() {
        prompt.push_str("RECENT WORKOUTS (newest first):\n");
        for r in &recent {
            prompt.push_str(&format!("{}\n", r));
        }
    }

    // Routines
    if !routine_info.is_empty() {
        prompt.push_str(&format!("ROUTINES: {}\n", routine_info.join("; ")));
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
    let chat_container_ref = use_node_ref();

    // Auto-scroll to bottom when messages change
    {
        let chat_container_ref = chat_container_ref.clone();
        let messages_len = messages.len();
        let model_state_dep = (*model_state).clone();
        use_effect_with((messages_len, model_state_dep), move |_| {
            if let Some(el) = chat_container_ref.cast::<web_sys::HtmlElement>() {
                el.set_scroll_top(el.scroll_height());
            }
            || ()
        });
    }

    let on_load_model = {
        let model_state = model_state.clone();
        Callback::from(move |_: MouseEvent| {
            let model_state = model_state.clone();
            let config = storage::load_user_config();
            let model_id = config.ai_model.model_id().to_string();
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
                    JsFuture::from(webllm_init(&model_id, progress_cb.as_ref().unchecked_ref()))
                        .await;
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

            save_chat_history(&msgs);
            messages.set(msgs.clone());
            model_state.set(ModelState::Generating);

            let messages = messages.clone();
            let model_state = model_state.clone();
            let input_ref = input_ref.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let system_prompt = build_system_prompt();
                // Only send last 6 messages to the model (context window limit)
                let context_msgs: Vec<&ChatMessage> = if msgs.len() > 6 {
                    msgs[msgs.len() - 6..].iter().collect()
                } else {
                    msgs.iter().collect()
                };
                let chat_msgs: Vec<serde_json::Value> = context_msgs
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
                save_chat_history(&msgs);
                messages.set(msgs.clone());
                model_state.set(ModelState::Generating);

                let messages = messages.clone();
                let model_state = model_state.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let system_prompt = build_system_prompt();
                    let context_msgs: Vec<&ChatMessage> = if msgs.len() > 6 {
                        msgs[msgs.len() - 6..].iter().collect()
                    } else {
                        msgs.iter().collect()
                    };
                    let chat_msgs: Vec<serde_json::Value> = context_msgs
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
            <div ref={chat_container_ref} class="flex-1 overflow-y-auto px-4 py-4 space-y-4">
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
                    ModelState::NotLoaded => {
                        let selected_model = storage::load_user_config().ai_model;
                        html! {
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
                                    {format!("Load {}", selected_model.display_name())}
                                </button>
                                <p class="text-[10px] text-gray-400">
                                    {"One-time download, cached for offline use. Runs entirely on your device. Change model in Settings."}
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
                    }},
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
