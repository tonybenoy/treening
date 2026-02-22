use crate::models;
use crate::storage;
use crate::Route;
use chrono::Datelike;
use gloo::storage::Storage as _;
use std::collections::HashSet;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use yew::prelude::*;
use yew::virtual_dom::VNode;
use yew_router::prelude::*;

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
    pub fn webllm_chat(sys: &str, msgs: &str) -> js_sys::Promise;

    #[wasm_bindgen(js_name = webllmChatStream)]
    fn webllm_chat_stream(sys: &str, msgs: &str, on_chunk: &JsValue) -> js_sys::Promise;

    #[wasm_bindgen(js_name = webllmReset)]
    fn webllm_reset() -> js_sys::Promise;

    #[wasm_bindgen(js_name = webllmIsLoaded)]
    pub fn webllm_is_loaded() -> bool;

    #[wasm_bindgen(js_name = speechRecognitionSupported)]
    fn speech_recognition_supported() -> bool;

    #[wasm_bindgen(js_name = startSpeechRecognition)]
    fn start_speech_recognition(on_result: &JsValue, on_end: &JsValue);

    #[wasm_bindgen(js_name = speechSynthesisSupported)]
    fn speech_synthesis_supported() -> bool;

    #[wasm_bindgen(js_name = speakText)]
    fn speak_text(text: &str, on_end: &JsValue);

    #[wasm_bindgen(js_name = stopSpeaking)]
    fn stop_speaking();

    #[wasm_bindgen(js_name = isSpeaking)]
    fn is_speaking() -> bool;
}

#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

// --- Multiple Chat Threads ---
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct ChatThread {
    id: String,
    title: String,
    messages: Vec<ChatMessage>,
    created_at: String,
}

const THREADS_KEY: &str = "treening_ai_chat_threads";
const ACTIVE_THREAD_KEY: &str = "treening_ai_active_thread";
const OLD_CHAT_HISTORY_KEY: &str = "treening_ai_chat_history";
const MAX_THREADS: usize = 20;

fn load_threads() -> Vec<ChatThread> {
    let mut threads: Vec<ChatThread> =
        gloo::storage::LocalStorage::get(THREADS_KEY).unwrap_or_default();

    // Migrate old chat history if it exists
    if threads.is_empty() {
        let old_msgs: Vec<ChatMessage> =
            gloo::storage::LocalStorage::get(OLD_CHAT_HISTORY_KEY).unwrap_or_default();
        if !old_msgs.is_empty() {
            let title = old_msgs
                .iter()
                .find(|m| m.role == "user")
                .map(|m| {
                    let t: String = m.content.chars().take(30).collect();
                    if m.content.len() > 30 {
                        format!("{}...", t)
                    } else {
                        t
                    }
                })
                .unwrap_or_else(|| "Chat".to_string());
            let thread = ChatThread {
                id: uuid(),
                title,
                messages: old_msgs,
                created_at: chrono::Local::now().format("%Y-%m-%d").to_string(),
            };
            threads.push(thread);
            save_threads(&threads);
            gloo::storage::LocalStorage::delete(OLD_CHAT_HISTORY_KEY);
        }
    }
    threads
}

fn save_threads(threads: &[ChatThread]) {
    let _ = gloo::storage::LocalStorage::set(THREADS_KEY, threads);
}

fn get_active_thread_id() -> Option<String> {
    gloo::storage::LocalStorage::get(ACTIVE_THREAD_KEY).ok()
}

fn set_active_thread_id(id: &str) {
    let _ = gloo::storage::LocalStorage::set(ACTIVE_THREAD_KEY, id.to_string());
}

fn uuid() -> String {
    let now = js_sys::Date::now() as u64;
    let rand = (js_sys::Math::random() * 1_000_000.0) as u64;
    format!("{:x}-{:x}", now, rand)
}

fn get_or_create_active_thread(threads: &mut Vec<ChatThread>) -> usize {
    let active_id = get_active_thread_id();
    if let Some(id) = &active_id {
        if let Some(idx) = threads.iter().position(|t| t.id == *id) {
            return idx;
        }
    }
    // Create new thread
    let thread = ChatThread {
        id: uuid(),
        title: "New Chat".to_string(),
        messages: Vec::new(),
        created_at: chrono::Local::now().format("%Y-%m-%d").to_string(),
    };
    let id = thread.id.clone();
    threads.insert(0, thread);
    if threads.len() > MAX_THREADS {
        threads.truncate(MAX_THREADS);
    }
    set_active_thread_id(&id);
    save_threads(threads);
    0
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

// --- Context-Aware Quick Prompts ---
fn generate_quick_prompts() -> Vec<String> {
    let workouts = storage::load_workouts();
    let streak = models::current_streak(&workouts);
    let all_exercises = {
        let mut exs = crate::data::default_exercises();
        exs.extend(storage::load_custom_exercises());
        exs
    };

    let mut prompts = vec![
        "Suggest a workout for today".to_string(),
        "Log a workout".to_string(),
        "What muscles am I neglecting?".to_string(),
    ];

    if workouts.len() >= 7 {
        prompts.push("Compare this week vs last week".to_string());
    }

    if streak >= 2 {
        prompts.push(format!("How's my {} day streak?", streak));
    }

    // Find latest exercise for dynamic prompt
    let mut sorted: Vec<&models::Workout> = workouts.iter().collect();
    sorted.sort_by(|a, b| b.date.cmp(&a.date));
    if let Some(latest) = sorted.first() {
        if let Some(we) = latest.exercises.first() {
            let name = all_exercises
                .iter()
                .find(|e| e.id == we.exercise_id)
                .map(|e| e.name.clone())
                .unwrap_or_else(|| we.exercise_id.clone());
            prompts.push(format!("Rate my {} progress", name));
        }
    }

    // Rotate by day of week for variety: shift the list
    let day = chrono::Local::now().weekday().num_days_from_monday() as usize;
    if prompts.len() > 2 {
        let shift = day % (prompts.len() - 1);
        // Keep first 2 fixed, rotate the rest
        let fixed = prompts[..2].to_vec();
        let mut dynamic: Vec<String> = prompts[2..].to_vec();
        let len = dynamic.len().max(1);
        dynamic.rotate_left(shift % len);
        prompts = fixed;
        prompts.extend(dynamic);
    }

    prompts.truncate(5);
    prompts
}

// --- Actionable AI Outputs ---
fn parse_workout_exercises(text: &str) -> Vec<(String, String)> {
    // Returns vec of (exercise_id, exercise_name) for matched exercises
    let all_exercises = {
        let mut exs = crate::data::default_exercises();
        exs.extend(storage::load_custom_exercises());
        exs
    };

    let mut matched: Vec<(String, String)> = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    for line in text.lines() {
        let trimmed = line.trim();
        // Check lines starting with "- " or numbered like "1. "
        let content = if let Some(rest) = trimmed.strip_prefix("- ") {
            rest
        } else if let Some(rest) = trimmed.strip_prefix("* ") {
            rest
        } else if trimmed
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
            && trimmed.contains(". ")
        {
            if let Some(pos) = trimmed.find(". ") {
                &trimmed[pos + 2..]
            } else {
                continue;
            }
        } else {
            continue;
        };

        let content_lower = content.to_lowercase();
        // Try to match against known exercises (case-insensitive substring)
        for ex in &all_exercises {
            let ex_lower = ex.name.to_lowercase();
            if content_lower.contains(&ex_lower) && !seen_ids.contains(&ex.id) {
                seen_ids.insert(ex.id.clone());
                matched.push((ex.id.clone(), ex.name.clone()));
                break;
            }
        }
    }

    matched
}

/// Parse a [WORKOUT LOG] block from AI response into exercise data ready to save.
/// Returns None if no block found, Some(vec) of (exercise_id, exercise_name, Vec<WorkoutSet>).
fn parse_workout_log(text: &str) -> Option<Vec<(String, String, Vec<models::WorkoutSet>)>> {
    let start_tag = "[WORKOUT LOG]";
    let end_tag = "[/WORKOUT LOG]";
    let start_idx = text.find(start_tag)?;
    let after_start = start_idx + start_tag.len();
    let end_idx = text[after_start..].find(end_tag).map(|i| after_start + i)?;
    let block = &text[after_start..end_idx];

    let all_exercises = {
        let mut exs = crate::data::default_exercises();
        exs.extend(storage::load_custom_exercises());
        exs
    };

    let config = storage::load_user_config();
    let us = &config.unit_system;

    let mut results: Vec<(String, String, Vec<models::WorkoutSet>)> = Vec::new();

    for line in block.lines() {
        let trimmed = line.trim();
        let content = if let Some(rest) = trimmed.strip_prefix("- ") {
            rest
        } else {
            continue;
        };

        // Split on first ':' → exercise name + sets string
        let colon_pos = match content.find(':') {
            Some(p) => p,
            None => continue,
        };
        let ex_name_raw = content[..colon_pos].trim();
        let sets_str = content[colon_pos + 1..].trim();

        // Match exercise name case-insensitively
        let ex_lower = ex_name_raw.to_lowercase();
        let matched_ex = all_exercises
            .iter()
            .find(|e| e.name.to_lowercase() == ex_lower)
            .or_else(|| {
                all_exercises.iter().find(|e| {
                    e.name.to_lowercase().contains(&ex_lower)
                        || ex_lower.contains(&e.name.to_lowercase())
                })
            });

        let (ex_id, ex_display_name) = match matched_ex {
            Some(ex) => (ex.id.clone(), ex.name.clone()),
            None => continue,
        };

        // Parse sets: split on ','
        let mut sets: Vec<models::WorkoutSet> = Vec::new();
        for set_token in sets_str.split(',') {
            let token = set_token.trim().to_lowercase();
            if token.is_empty() {
                continue;
            }

            // Try "weight x reps" pattern (e.g. "80 x 10", "80kg x 10", "80kgx10")
            if let Some(set) = parse_weight_reps_token(&token, us) {
                sets.push(set);
            }
            // Try bodyweight "x15" or "15 reps"
            else if let Some(reps) = parse_bodyweight_token(&token) {
                sets.push(models::WorkoutSet {
                    weight: 0.0,
                    reps,
                    completed: true,
                    duration_secs: None,
                    distance: None,
                    note: None,
                });
            }
            // Try duration "30s" or "2min"
            else if let Some(secs) = parse_duration_token(&token) {
                sets.push(models::WorkoutSet {
                    weight: 0.0,
                    reps: 0,
                    completed: true,
                    duration_secs: Some(secs),
                    distance: None,
                    note: None,
                });
            }
        }

        if !sets.is_empty() {
            results.push((ex_id, ex_display_name, sets));
        }
    }

    if results.is_empty() {
        None
    } else {
        Some(results)
    }
}

/// Parse a token like "80 x 10", "80kg x 10", "80kgx10", "80 x10"
fn parse_weight_reps_token(token: &str, us: &models::UnitSystem) -> Option<models::WorkoutSet> {
    // Find 'x' separator
    let x_pos = token.find('x')?;
    let weight_part = token[..x_pos]
        .trim()
        .trim_end_matches("kg")
        .trim_end_matches("lbs")
        .trim();
    let reps_part = token[x_pos + 1..].trim();

    let weight_val: f64 = weight_part.parse().ok()?;
    let reps_val: u32 = reps_part.parse().ok()?;

    // Convert to kg for storage (user enters in their unit system)
    let weight_kg = us.to_kg(weight_val);

    Some(models::WorkoutSet {
        weight: weight_kg,
        reps: reps_val,
        completed: true,
        duration_secs: None,
        distance: None,
        note: None,
    })
}

/// Parse bodyweight token like "x15", "15 reps", "15"
fn parse_bodyweight_token(token: &str) -> Option<u32> {
    let cleaned = token
        .trim_start_matches('x')
        .trim()
        .trim_end_matches("reps")
        .trim();
    cleaned.parse().ok()
}

/// Parse duration token like "30s", "2min", "120s"
fn parse_duration_token(token: &str) -> Option<u32> {
    if token.ends_with('s') {
        let num = token.trim_end_matches('s').trim();
        num.parse().ok()
    } else if token.ends_with("min") {
        let num = token.trim_end_matches("min").trim();
        let mins: u32 = num.parse().ok()?;
        Some(mins * 60)
    } else {
        None
    }
}

pub fn build_system_prompt() -> String {
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
         - If they have no data yet, welcome them and suggest getting started\n\
         - When suggesting a workout, format each exercise on its own line starting with \"- \" \
           followed by the exercise name, e.g. \"- Bench Press: 3x10\"\n\
         - When the user tells you about a workout they already completed (e.g. \"I did bench press 80kg 3x10\"), \
           summarize it in a structured block so it can be saved:\n\
           [WORKOUT LOG]\n\
           - ExerciseName: weight x reps, weight x reps, weight x reps\n\
           - ExerciseName: weight x reps, weight x reps\n\
           [/WORKOUT LOG]\n\
           Use the user's unit system. Include all sets they mention. For bodyweight exercises use \"x reps\". \
           For duration exercises use \"30s\" or \"2min\". Always use exact exercise names from the database.\n\n\
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
    let threads = use_state(load_threads);
    let active_thread_id = use_state(|| get_active_thread_id().unwrap_or_default());
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
    let scroll_trigger = use_state(|| 0u32);
    let show_thread_list = use_state(|| false);
    let is_recording = use_state(|| false);
    let show_export_toast = use_state(|| false);
    let speaking_msg_idx = use_state(|| Option::<usize>::None);
    let saved_workout_indices = use_state(HashSet::<usize>::new);
    let navigator = use_navigator().unwrap();

    // Get active thread messages
    let active_messages: Vec<ChatMessage> = {
        let tid = &*active_thread_id;
        (*threads)
            .iter()
            .find(|t| t.id == *tid)
            .map(|t| t.messages.clone())
            .unwrap_or_default()
    };

    let active_title: String = {
        let tid = &*active_thread_id;
        (*threads)
            .iter()
            .find(|t| t.id == *tid)
            .map(|t| t.title.clone())
            .unwrap_or_else(|| "New Chat".to_string())
    };

    // Auto-scroll to bottom when messages change or scroll_trigger fires
    {
        let chat_container_ref = chat_container_ref.clone();
        let msgs_len = active_messages.len();
        let scroll_val = *scroll_trigger;
        let model_state_dep = (*model_state).clone();
        use_effect_with((msgs_len, model_state_dep, scroll_val), move |_| {
            if let Some(el) = chat_container_ref.cast::<web_sys::HtmlElement>() {
                el.set_scroll_top(el.scroll_height());
            }
            || ()
        });
    }

    // Ensure there's an active thread on first render
    {
        let threads = threads.clone();
        let active_thread_id = active_thread_id.clone();
        use_effect_with((), move |_| {
            let mut ts = (*threads).clone();
            let idx = get_or_create_active_thread(&mut ts);
            let id = ts[idx].id.clone();
            active_thread_id.set(id);
            threads.set(ts);
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

    // Helper: send a message (used by on_send and on_quick)
    let do_send = {
        let threads = threads.clone();
        let active_thread_id = active_thread_id.clone();
        let model_state = model_state.clone();
        let input_ref = input_ref.clone();
        let scroll_trigger = scroll_trigger.clone();
        move |text: String| {
            if text.is_empty() {
                return;
            }

            let tid = (*active_thread_id).clone();
            let mut ts = load_threads();
            let idx = ts.iter().position(|t| t.id == tid);
            let idx = match idx {
                Some(i) => i,
                None => return,
            };

            // Add user message
            ts[idx].messages.push(ChatMessage {
                role: "user".to_string(),
                content: text,
            });

            // Auto-title from first user message
            if ts[idx].title == "New Chat" {
                if let Some(first_user) = ts[idx].messages.iter().find(|m| m.role == "user") {
                    let t: String = first_user.content.chars().take(30).collect();
                    ts[idx].title = if first_user.content.len() > 30 {
                        format!("{}...", t)
                    } else {
                        t
                    };
                }
            }

            // Push empty assistant message for streaming
            ts[idx].messages.push(ChatMessage {
                role: "assistant".to_string(),
                content: String::new(),
            });

            save_threads(&ts);
            threads.set(ts.clone());
            model_state.set(ModelState::Generating);

            let threads_handle = threads.clone();
            let model_state = model_state.clone();
            let input_ref = input_ref.clone();
            let scroll_trigger = scroll_trigger.clone();
            let tid = tid.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let system_prompt = build_system_prompt();
                // Get current thread messages (excluding the empty assistant message)
                let ts = load_threads();
                let thread = ts.iter().find(|t| t.id == tid);
                let msgs: Vec<ChatMessage> = thread
                    .map(|t| {
                        t.messages
                            .iter()
                            .filter(|m| !(m.role == "assistant" && m.content.is_empty()))
                            .cloned()
                            .collect()
                    })
                    .unwrap_or_default();

                // Only send last 6 messages to the model
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

                // Stream response — only update in-memory state (no localStorage per chunk)
                let chunk_cb = {
                    let threads_handle = threads_handle.clone();
                    let scroll_trigger = scroll_trigger.clone();
                    let tid = tid.clone();
                    Closure::wrap(Box::new(move |chunk: JsValue| {
                        if let Some(content) = chunk.as_string() {
                            let mut ts = (*threads_handle).clone();
                            if let Some(thread) = ts.iter_mut().find(|t| t.id == tid) {
                                if let Some(last) = thread.messages.last_mut() {
                                    if last.role == "assistant" {
                                        last.content = content;
                                    }
                                }
                            }
                            threads_handle.set(ts);
                            scroll_trigger.set(*scroll_trigger + 1);
                        }
                    }) as Box<dyn FnMut(JsValue)>)
                };

                let result = JsFuture::from(webllm_chat_stream(
                    &system_prompt,
                    &msgs_json,
                    chunk_cb.as_ref().unchecked_ref(),
                ))
                .await;
                chunk_cb.forget();

                match result {
                    Ok(response) => {
                        let final_content = response.as_string().unwrap_or_default();
                        // Update with final content
                        let mut ts = load_threads();
                        if let Some(thread) = ts.iter_mut().find(|t| t.id == tid) {
                            if let Some(last) = thread.messages.last_mut() {
                                if last.role == "assistant" {
                                    last.content = final_content;
                                }
                            }
                        }
                        save_threads(&ts);
                        threads_handle.set(ts);
                        model_state.set(ModelState::Ready);
                    }
                    Err(e) => {
                        // Remove empty assistant message on error
                        let mut ts = load_threads();
                        if let Some(thread) = ts.iter_mut().find(|t| t.id == tid) {
                            if let Some(last) = thread.messages.last() {
                                if last.role == "assistant" && last.content.is_empty() {
                                    thread.messages.pop();
                                }
                            }
                        }
                        save_threads(&ts);
                        threads_handle.set(ts);
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
        }
    };

    let on_send = {
        let input_text = input_text.clone();
        let do_send = do_send.clone();
        Callback::from(move |_: ()| {
            let text = (*input_text).trim().to_string();
            if text.is_empty() {
                return;
            }
            input_text.set(String::new());
            do_send(text);
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
        let do_send = do_send.clone();
        move |prompt: String| {
            let do_send = do_send.clone();
            Callback::from(move |_: MouseEvent| {
                do_send(prompt.clone());
            })
        }
    };

    // --- Thread management ---
    let on_new_thread = {
        let threads = threads.clone();
        let active_thread_id = active_thread_id.clone();
        let show_thread_list = show_thread_list.clone();
        Callback::from(move |_: MouseEvent| {
            let thread = ChatThread {
                id: uuid(),
                title: "New Chat".to_string(),
                messages: Vec::new(),
                created_at: chrono::Local::now().format("%Y-%m-%d").to_string(),
            };
            let id = thread.id.clone();
            let mut ts = load_threads();
            ts.insert(0, thread);
            if ts.len() > MAX_THREADS {
                ts.truncate(MAX_THREADS);
            }
            save_threads(&ts);
            set_active_thread_id(&id);
            active_thread_id.set(id);
            threads.set(ts);
            show_thread_list.set(false);
            // Reset KV cache
            wasm_bindgen_futures::spawn_local(async move {
                let _ = JsFuture::from(webllm_reset()).await;
            });
        })
    };

    let on_toggle_thread_list = {
        let show_thread_list = show_thread_list.clone();
        Callback::from(move |_: MouseEvent| {
            show_thread_list.set(!*show_thread_list);
        })
    };

    let on_select_thread = {
        let threads = threads.clone();
        let active_thread_id = active_thread_id.clone();
        let show_thread_list = show_thread_list.clone();
        move |tid: String| {
            let threads = threads.clone();
            let active_thread_id = active_thread_id.clone();
            let show_thread_list = show_thread_list.clone();
            Callback::from(move |_: MouseEvent| {
                set_active_thread_id(&tid);
                active_thread_id.set(tid.clone());
                threads.set(load_threads());
                show_thread_list.set(false);
                // Reset KV cache when switching threads
                wasm_bindgen_futures::spawn_local(async move {
                    let _ = JsFuture::from(webllm_reset()).await;
                });
            })
        }
    };

    let on_delete_thread = {
        let threads = threads.clone();
        let active_thread_id = active_thread_id.clone();
        Callback::from(move |_: MouseEvent| {
            let tid = (*active_thread_id).clone();
            let mut ts = load_threads();
            ts.retain(|t| t.id != tid);
            save_threads(&ts);

            // Switch to first thread or create new
            if ts.is_empty() {
                let thread = ChatThread {
                    id: uuid(),
                    title: "New Chat".to_string(),
                    messages: Vec::new(),
                    created_at: chrono::Local::now().format("%Y-%m-%d").to_string(),
                };
                let id = thread.id.clone();
                ts.push(thread);
                save_threads(&ts);
                set_active_thread_id(&id);
                active_thread_id.set(id);
            } else {
                let id = ts[0].id.clone();
                set_active_thread_id(&id);
                active_thread_id.set(id);
            }
            threads.set(ts);
            wasm_bindgen_futures::spawn_local(async move {
                let _ = JsFuture::from(webllm_reset()).await;
            });
        })
    };

    // --- Export Chat ---
    let on_export = {
        let show_export_toast = show_export_toast.clone();
        let active_thread_id = active_thread_id.clone();
        Callback::from(move |_: MouseEvent| {
            let ts = load_threads();
            let tid = (*active_thread_id).clone();
            if let Some(thread) = ts.iter().find(|t| t.id == tid) {
                let text: String = thread
                    .messages
                    .iter()
                    .map(|m| {
                        let role = if m.role == "user" { "You" } else { "AI" };
                        format!("{}: {}", role, m.content)
                    })
                    .collect::<Vec<_>>()
                    .join("\n\n");

                let window = web_sys::window().unwrap();
                let clipboard = window.navigator().clipboard();
                let _ = clipboard.write_text(&text);
                show_export_toast.set(true);
                let toast = show_export_toast.clone();
                gloo::timers::callback::Timeout::new(2_000, move || {
                    toast.set(false);
                })
                .forget();
            }
        })
    };

    // --- Voice Input ---
    let on_voice = {
        let is_recording = is_recording.clone();
        let input_text = input_text.clone();
        Callback::from(move |_: MouseEvent| {
            if *is_recording {
                return;
            }
            is_recording.set(true);

            let input_text = input_text.clone();
            let is_recording = is_recording.clone();

            let on_result = Closure::wrap(Box::new(move |transcript: JsValue| {
                if let Some(text) = transcript.as_string() {
                    input_text.set(text);
                }
            }) as Box<dyn FnMut(JsValue)>);

            let is_rec_end = is_recording.clone();
            let on_end = Closure::wrap(Box::new(move || {
                is_rec_end.set(false);
            }) as Box<dyn FnMut()>);

            start_speech_recognition(
                on_result.as_ref().unchecked_ref(),
                on_end.as_ref().unchecked_ref(),
            );
            on_result.forget();
            on_end.forget();
        })
    };

    // --- Actionable outputs: start workout button ---
    let render_start_workout_btn = {
        let navigator = navigator.clone();
        move |text: &str| -> Html {
            let matched = parse_workout_exercises(text);
            if matched.len() < 2 {
                return html! {};
            }
            let count = matched.len();
            let exercise_ids: Vec<String> = matched.iter().map(|(id, _)| id.clone()).collect();
            let nav = navigator.clone();
            let on_start = Callback::from(move |_: MouseEvent| {
                // Build WorkoutExercise list
                let exercises: Vec<models::WorkoutExercise> = exercise_ids
                    .iter()
                    .map(|id| models::WorkoutExercise {
                        exercise_id: id.clone(),
                        sets: vec![
                            models::WorkoutSet {
                                weight: 0.0,
                                reps: 0,
                                completed: false,
                                duration_secs: None,
                                distance: None,
                                note: None,
                            };
                            3
                        ],
                        notes: String::new(),
                        superset_group: None,
                        rest_seconds_override: None,
                    })
                    .collect();
                if let Ok(json) = serde_json::to_string(&exercises) {
                    let _ = gloo::storage::LocalStorage::set("treening_active_repeat", json);
                }
                nav.push(&Route::Workout);
            });
            html! {
                <button
                    onclick={on_start}
                    class="mt-2 w-full py-2 bg-green-600 hover:bg-green-700 text-white rounded-lg text-xs font-bold transition neu-btn btn-press flex items-center justify-center gap-1"
                >
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                    </svg>
                    {format!("Start workout ({} exercises)", count)}
                </button>
            }
        }
    };

    // --- Actionable outputs: save workout log button ---
    let render_save_workout_btn = {
        let saved_workout_indices = saved_workout_indices.clone();
        move |text: &str, msg_idx: usize| -> Html {
            let parsed = parse_workout_log(text);
            let exercises_data = match parsed {
                Some(data) if !data.is_empty() => data,
                _ => return html! {},
            };

            if saved_workout_indices.contains(&msg_idx) {
                return html! {
                    <div class="mt-2 w-full py-2 bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-400 rounded-lg text-xs font-bold text-center flex items-center justify-center gap-1">
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                        </svg>
                        {"Saved!"}
                    </div>
                };
            }

            let total_sets: usize = exercises_data.iter().map(|(_, _, sets)| sets.len()).sum();
            let ex_count = exercises_data.len();
            let label = format!("Save workout ({} exercises, {} sets)", ex_count, total_sets);

            let exercises_for_save: Vec<(String, Vec<models::WorkoutSet>)> = exercises_data
                .iter()
                .map(|(id, _, sets)| (id.clone(), sets.clone()))
                .collect();
            let saved_indices2 = saved_workout_indices.clone();
            let on_save = Callback::from(move |_: MouseEvent| {
                let workout_exercises: Vec<models::WorkoutExercise> = exercises_for_save
                    .iter()
                    .map(|(id, sets)| models::WorkoutExercise {
                        exercise_id: id.clone(),
                        sets: sets.clone(),
                        notes: String::new(),
                        superset_group: None,
                        rest_seconds_override: None,
                    })
                    .collect();

                let workout = models::Workout {
                    id: uuid(),
                    date: chrono::Local::now().format("%Y-%m-%d").to_string(),
                    name: "AI Logged Workout".to_string(),
                    exercises: workout_exercises,
                    duration_mins: 0,
                };

                let mut workouts = storage::load_workouts();
                workouts.push(workout);
                storage::save_workouts(&workouts);

                let mut saved = (*saved_indices2).clone();
                saved.insert(msg_idx);
                saved_indices2.set(saved);
            });

            html! {
                <button
                    onclick={on_save}
                    class="mt-2 w-full py-2 bg-green-600 hover:bg-green-700 text-white rounded-lg text-xs font-bold transition neu-btn btn-press flex items-center justify-center gap-1"
                >
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                    </svg>
                    {label}
                </button>
            }
        }
    };

    let quick_prompts = generate_quick_prompts();

    // Check voice support
    let voice_supported = speech_recognition_supported();
    let tts_supported = speech_synthesis_supported();

    html! {
        <div class="flex flex-col h-[calc(100vh-5rem)]">
            // Header
            <div class="px-4 py-3 flex items-center justify-between border-b border-gray-200 dark:border-gray-700/50">
                <div class="flex items-center gap-2 min-w-0 flex-1">
                    <span class="text-xl flex-shrink-0">{"🤖"}</span>
                    <button
                        onclick={on_toggle_thread_list.clone()}
                        class="text-sm font-bold text-gray-900 dark:text-gray-100 truncate hover:text-blue-600 dark:hover:text-blue-400 transition flex items-center gap-1"
                    >
                        {&active_title}
                        <svg class="w-3 h-3 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                        </svg>
                    </button>
                    <span class="px-2 py-0.5 bg-blue-100 dark:bg-blue-600/20 text-blue-600 dark:text-blue-400 text-[10px] font-bold rounded-full uppercase flex-shrink-0">{"Local"}</span>
                </div>
                if *model_state == ModelState::Ready || *model_state == ModelState::Generating {
                    <div class="flex items-center gap-1 flex-shrink-0">
                        <button
                            onclick={on_export}
                            class="text-xs text-gray-500 hover:text-gray-700 dark:hover:text-gray-300 px-2 py-1 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800 transition"
                            title="Export chat"
                        >
                            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 5H6a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2v-1M8 5a2 2 0 002 2h2a2 2 0 002-2M8 5a2 2 0 012-2h2a2 2 0 012 2m0 0h2a2 2 0 012 2v3m2 4H10m0 0l3-3m-3 3l3 3" />
                            </svg>
                        </button>
                        <button
                            onclick={on_new_thread.clone()}
                            class="text-xs text-gray-500 hover:text-gray-700 dark:hover:text-gray-300 px-2 py-1 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800 transition"
                            title="New chat"
                        >
                            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
                            </svg>
                        </button>
                        <button
                            onclick={on_delete_thread}
                            class="text-xs text-gray-500 hover:text-red-500 dark:hover:text-red-400 px-2 py-1 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800 transition"
                            title="Delete thread"
                        >
                            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                            </svg>
                        </button>
                    </div>
                }
            </div>

            // Thread list overlay
            if *show_thread_list {
                <div class="absolute inset-0 z-50 bg-black/30" onclick={
                    let stl = show_thread_list.clone();
                    Callback::from(move |_: MouseEvent| stl.set(false))
                }>
                    <div class="absolute top-14 left-4 right-4 bg-white dark:bg-gray-800 rounded-xl shadow-xl max-h-80 overflow-y-auto neu-flat"
                        onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}
                    >
                        <div class="p-3 border-b border-gray-200 dark:border-gray-700/50 flex justify-between items-center">
                            <span class="text-sm font-bold text-gray-700 dark:text-gray-300">{"Chat Threads"}</span>
                            <button
                                onclick={on_new_thread.clone()}
                                class="text-xs text-blue-600 dark:text-blue-400 font-bold"
                            >
                                {"+ New Chat"}
                            </button>
                        </div>
                        { for (*threads).iter().map(|t| {
                            let is_active = t.id == *active_thread_id;
                            let tid = t.id.clone();
                            html! {
                                <button
                                    onclick={on_select_thread(tid)}
                                    class={format!(
                                        "w-full text-left px-3 py-2.5 border-b border-gray-100 dark:border-gray-700/30 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition {}",
                                        if is_active { "bg-blue-50 dark:bg-blue-900/20" } else { "" }
                                    )}
                                >
                                    <div class="text-sm font-medium text-gray-800 dark:text-gray-200 truncate">{&t.title}</div>
                                    <div class="text-[10px] text-gray-400 mt-0.5">{&t.created_at}{" · "}{t.messages.len()}{" messages"}</div>
                                </button>
                            }
                        })}
                    </div>
                </div>
            }

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
                            if active_messages.is_empty() {
                                <div class="space-y-3 pt-4">
                                    <p class="text-center text-sm text-gray-500">{"Ask me about your workouts!"}</p>
                                    <div class="flex flex-wrap gap-2 justify-center">
                                        { for quick_prompts.iter().map(|p| {
                                            let prompt = p.clone();
                                            html! {
                                                <button
                                                    onclick={on_quick(prompt.clone())}
                                                    class="neu-chip rounded-full px-3 py-1.5 text-xs text-gray-600 dark:text-gray-300 bg-gray-100 dark:bg-gray-800 hover:bg-gray-200 dark:hover:bg-gray-700 transition btn-press"
                                                >
                                                    {prompt}
                                                </button>
                                            }
                                        })}
                                    </div>
                                </div>
                            }

                            { for active_messages.iter().enumerate().map(|(idx, msg)| {
                                let is_user = msg.role == "user";
                                let is_empty_assistant = !is_user && msg.content.is_empty();
                                let content = msg.content.clone();
                                let is_this_speaking = *speaking_msg_idx == Some(idx);
                                // TTS button for non-empty assistant messages
                                let speak_btn = if !is_user && !is_empty_assistant && tts_supported {
                                    let speaking_msg_idx = speaking_msg_idx.clone();
                                    let content_for_tts = content.clone();
                                    let on_speak = Callback::from(move |_: MouseEvent| {
                                        if is_speaking() {
                                            stop_speaking();
                                            speaking_msg_idx.set(None);
                                        } else {
                                            speaking_msg_idx.set(Some(idx));
                                            let smi = speaking_msg_idx.clone();
                                            let on_end = Closure::wrap(Box::new(move || {
                                                smi.set(None);
                                            }) as Box<dyn FnMut()>);
                                            speak_text(&content_for_tts, on_end.as_ref().unchecked_ref());
                                            on_end.forget();
                                        }
                                    });
                                    html! {
                                        <button
                                            onclick={on_speak}
                                            class={format!(
                                                "mt-1.5 flex items-center gap-1 text-[10px] transition {}",
                                                if is_this_speaking {
                                                    "text-blue-500 dark:text-blue-400"
                                                } else {
                                                    "text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                                                }
                                            )}
                                            title={if is_this_speaking { "Stop speaking" } else { "Read aloud" }}
                                        >
                                            { if is_this_speaking {
                                                html! {
                                                    <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 10h6v4H9z" />
                                                    </svg>
                                                }
                                            } else {
                                                html! {
                                                    <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.536 8.464a5 5 0 010 7.072m2.828-9.9a9 9 0 010 12.728M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707A1 1 0 0112 5.586v12.828a1 1 0 01-1.707.707L5.586 15z" />
                                                    </svg>
                                                }
                                            }}
                                            { if is_this_speaking { "Stop" } else { "Listen" } }
                                        </button>
                                    }
                                } else {
                                    html! {}
                                };
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
                                            } else if is_empty_assistant {
                                                html! {
                                                    <div class="flex gap-1.5 items-center">
                                                        <div class="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style="animation-delay: 0ms;" />
                                                        <div class="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style="animation-delay: 150ms;" />
                                                        <div class="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style="animation-delay: 300ms;" />
                                                    </div>
                                                }
                                            } else {
                                                html! {
                                                    <>
                                                        {render_markdown(&msg.content)}
                                                        {render_save_workout_btn(&content, idx)}
                                                        {render_start_workout_btn(&content)}
                                                        {speak_btn}
                                                    </>
                                                }
                                            }}
                                        </div>
                                    </div>
                                }
                            })}

                            <div ref={messages_end_ref} />
                        </>
                    },
                }}
            </div>

            // Export toast
            if *show_export_toast {
                <div class="fixed top-4 left-4 right-4 z-50 bg-green-600 text-white px-4 py-2 rounded-xl shadow-lg text-center text-sm font-bold" style="animation: modalContentIn 200ms ease-out;">
                    {"Copied to clipboard!"}
                </div>
            }

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
                        if voice_supported {
                            <button
                                onclick={on_voice}
                                disabled={*model_state == ModelState::Generating}
                                class={format!(
                                    "p-3 rounded-xl transition neu-btn btn-press {}",
                                    if *is_recording {
                                        "bg-red-500 hover:bg-red-600 text-white animate-pulse"
                                    } else {
                                        "bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 text-gray-600 dark:text-gray-300"
                                    }
                                )}
                                title="Voice input"
                            >
                                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11a7 7 0 01-7 7m0 0a7 7 0 01-7-7m7 7v4m0 0H8m4 0h4m-4-8a3 3 0 01-3-3V5a3 3 0 116 0v6a3 3 0 01-3 3z" />
                                </svg>
                            </button>
                        }
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
