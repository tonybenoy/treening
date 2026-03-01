use crate::components::achievements::AchievementBadges;
use crate::components::ai_chat;
use crate::models::{self, Exercise, Workout};
use crate::pages::muscles::muscle_balance_summary;
use crate::storage;
use crate::Route;
use chrono::Datelike;
use gloo::storage::{LocalStorage, Storage};
use wasm_bindgen_futures::JsFuture;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(SummaryStats)]
fn summary_stats() -> Html {
    let workouts = storage::load_workouts();

    if workouts.is_empty() {
        return html! {};
    }

    let total_workouts = workouts.len();
    let total_volume: f64 = workouts.iter().map(|w| w.total_volume()).sum();

    let weight = storage::load_body_metrics().first().and_then(|m| m.weight);

    let volume_display = if let Some(w) = weight {
        format!("{:.1}x", total_volume / w)
    } else if total_volume >= 1_000_000.0 {
        format!("{:.1}M", total_volume / 1_000_000.0)
    } else if total_volume >= 1000.0 {
        format!("{:.0}k", total_volume / 1000.0)
    } else {
        format!("{:.0}", total_volume)
    };

    let volume_label = if weight.is_some() {
        "Rel. Volume"
    } else {
        "Total Volume"
    };

    let streak = models::current_streak(&workouts);
    let best = models::best_streak(&workouts);

    html! {
        <div class="space-y-3">
            <div class="flex justify-between items-center px-1">
                <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">{"Your Progress"}</h2>
                <Link<Route> to={Route::Analytics} classes="text-xs text-blue-600 dark:text-blue-400 hover:underline">
                    {"Full Analytics →"}
                </Link<Route>>
            </div>
            <div class="grid grid-cols-3 gap-3">
                <div class="bg-gray-100 dark:bg-gray-800/50 rounded-xl p-3 neu-flat text-center transition-colors">
                    <div class="text-xl mb-1">{"🏋️"}</div>
                    <div class="text-lg font-bold text-gray-800 dark:text-gray-200">{total_workouts}</div>
                    <div class="text-[10px] text-gray-500 dark:text-gray-500 uppercase font-bold">{"Sessions"}</div>
                </div>
                <div class="bg-gray-100 dark:bg-gray-800/50 rounded-xl p-3 neu-flat text-center transition-colors">
                    <div class="text-xl mb-1">{"💪"}</div>
                    <div class="text-lg font-bold text-gray-800 dark:text-gray-200">{volume_display}</div>
                    <div class="text-[10px] text-gray-500 dark:text-gray-500 uppercase font-bold">{volume_label}</div>
                </div>
                <div class="bg-gray-100 dark:bg-gray-800/50 rounded-xl p-3 neu-flat text-center transition-colors">
                    <div class="text-xl mb-1">{"🔥"}</div>
                    <div class="text-lg font-bold text-gray-800 dark:text-gray-200">{streak}</div>
                    <div class="text-[10px] text-gray-500 dark:text-gray-500 uppercase font-bold">{"Day Streak"}</div>
                    <div class="text-[9px] text-gray-400 dark:text-gray-600 mt-0.5">{format!("Best: {}d", best)}</div>
                </div>
            </div>
        </div>
    }
}

#[function_component(CommunitySummary)]
fn community_summary() -> Html {
    let config = storage::load_user_config();
    let friends = storage::load_friends();

    if !config.social_enabled {
        return html! {
            <div class="bg-gradient-to-br from-gray-100 to-gray-200 dark:from-gray-800 dark:to-gray-900 rounded-2xl p-4 neu-flat transition-colors">
                <div class="flex justify-between items-center mb-3">
                    <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">{"Community"}</h2>
                    <span class="px-2 py-0.5 bg-blue-100 dark:bg-blue-900/40 text-blue-600 dark:text-blue-400 text-[10px] font-bold rounded-full uppercase tracking-wider">{"Optional"}</span>
                </div>
                <p class="text-sm text-gray-600 dark:text-gray-400 mb-4">{"Train with friends, share stats, and see your rankings in the community."}</p>
                <Link<Route> to={Route::Social} classes="flex items-center justify-center w-full py-2 bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 text-gray-900 dark:text-white rounded-lg text-sm font-bold transition neu-btn">
                    {"🚀 Join Community"}
                </Link<Route>>
            </div>
        };
    }

    html! {
        <div class="space-y-3">
            <div class="flex justify-between items-center px-1">
                <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">{"Friend Ranking"}</h2>
                <Link<Route> to={Route::Social} classes="text-xs text-blue-600 dark:text-blue-400 hover:underline">
                    {"View All →"}
                </Link<Route>>
            </div>

            <div class="bg-gray-100 dark:bg-gray-800/50 rounded-2xl p-4 neu-flat transition-colors">
                { if friends.is_empty() {
                    html! {
                        <div class="text-center py-4">
                            <p class="text-sm text-gray-500">{"No friends added yet."}</p>
                            <Link<Route> to={Route::Social} classes="text-xs text-blue-600 dark:text-blue-400 hover:underline mt-1 block">
                                {"Add your first friend →"}
                            </Link<Route>>
                        </div>
                    }
                } else {
                    html! {
                        <div class="space-y-3">
                            { for friends.iter().take(3).map(|f| {
                                html! {
                                    <div class="flex justify-between items-center">
                                        <div class="flex items-center gap-3">
                                            <div class="w-8 h-8 bg-blue-100 dark:bg-blue-600/20 text-blue-600 dark:text-blue-400 rounded-full flex items-center justify-center text-xs font-bold">
                                                {&f.name[..1]}
                                            </div>
                                            <div class="text-sm font-medium text-gray-800 dark:text-gray-200">{&f.name}</div>
                                        </div>
                                        <div class="text-xs text-gray-500">{"Offline"}</div>
                                    </div>
                                }
                            })}
                        </div>
                    }
                }}
            </div>
        </div>
    }
}

// --- Weekly AI Summary ---
const WEEKLY_SUMMARIES_KEY: &str = "treening_weekly_summaries";

#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct WeeklySummaryData {
    week: String, // ISO week like "2026-W08"
    summary: String,
}

fn current_iso_week() -> String {
    let now = chrono::Local::now().date_naive();
    format!("{}-W{:02}", now.iso_week().year(), now.iso_week().week())
}

fn load_weekly_summaries() -> Vec<WeeklySummaryData> {
    gloo::storage::LocalStorage::get(WEEKLY_SUMMARIES_KEY).unwrap_or_default()
}

fn save_weekly_summaries(summaries: &[WeeklySummaryData]) {
    let _ = gloo::storage::LocalStorage::set(WEEKLY_SUMMARIES_KEY, summaries);
}

#[function_component(WeeklySummary)]
fn weekly_summary() -> Html {
    let config = storage::load_user_config();
    let summary_text = use_state(|| Option::<String>::None);
    let is_generating = use_state(|| false);

    // Check for existing summary
    let week = current_iso_week();
    let existing: Option<String> = {
        let summaries = load_weekly_summaries();
        summaries
            .iter()
            .find(|s| s.week == week)
            .map(|s| s.summary.clone())
    };

    // Set existing summary on first render
    {
        let summary_text = summary_text.clone();
        let existing = existing.clone();
        use_effect_with((), move |_| {
            if let Some(text) = existing {
                summary_text.set(Some(text));
            }
            || ()
        });
    }

    // Auto-generate if conditions are met
    {
        let summary_text = summary_text.clone();
        let is_generating = is_generating.clone();
        let week = week.clone();
        let ai_enabled = config.ai_enabled;
        let has_existing = existing.is_some();
        use_effect_with((), move |_| {
            if ai_enabled
                && !has_existing
                && ai_chat::webllm_is_loaded()
                && !storage::load_workouts().is_empty()
            {
                is_generating.set(true);
                let summary_text = summary_text.clone();
                let is_generating = is_generating.clone();
                let week = week.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let system_prompt = ai_chat::build_system_prompt();
                    let user_msg = serde_json::json!([{
                        "role": "user",
                        "content": "Give me a brief weekly summary of my training. Include: sessions this week, total volume, highlights, and one tip for next week. Keep it to 3-4 sentences."
                    }]);
                    let msgs_json = serde_json::to_string(&user_msg).unwrap_or_default();
                    if let Ok(response) =
                        JsFuture::from(ai_chat::webllm_chat(&system_prompt, &msgs_json)).await
                    {
                        let reply = response.as_string().unwrap_or_default();
                        if !reply.is_empty() {
                            let mut summaries = load_weekly_summaries();
                            // Keep only last 8 weeks
                            summaries.retain(|s| s.week != week);
                            summaries.insert(
                                0,
                                WeeklySummaryData {
                                    week,
                                    summary: reply.clone(),
                                },
                            );
                            summaries.truncate(8);
                            save_weekly_summaries(&summaries);
                            summary_text.set(Some(reply));
                        }
                    }
                    is_generating.set(false);
                });
            }
            || ()
        });
    }

    // Regenerate callback
    let on_regenerate = {
        let summary_text = summary_text.clone();
        let is_generating = is_generating.clone();
        let week = week.clone();
        Callback::from(move |_: MouseEvent| {
            if *is_generating {
                return;
            }
            if !ai_chat::webllm_is_loaded() {
                return;
            }
            is_generating.set(true);
            let summary_text = summary_text.clone();
            let is_generating = is_generating.clone();
            let week = week.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let system_prompt = ai_chat::build_system_prompt();
                let user_msg = serde_json::json!([{
                    "role": "user",
                    "content": "Give me a brief weekly summary of my training. Include: sessions this week, total volume, highlights, and one tip for next week. Keep it to 3-4 sentences."
                }]);
                let msgs_json = serde_json::to_string(&user_msg).unwrap_or_default();
                if let Ok(response) =
                    JsFuture::from(ai_chat::webllm_chat(&system_prompt, &msgs_json)).await
                {
                    let reply = response.as_string().unwrap_or_default();
                    if !reply.is_empty() {
                        let mut summaries = load_weekly_summaries();
                        summaries.retain(|s| s.week != week);
                        summaries.insert(
                            0,
                            WeeklySummaryData {
                                week,
                                summary: reply.clone(),
                            },
                        );
                        summaries.truncate(8);
                        save_weekly_summaries(&summaries);
                        summary_text.set(Some(reply));
                    }
                }
                is_generating.set(false);
            });
        })
    };

    if !config.ai_enabled {
        return html! {};
    }

    // Show card if we have a summary or are generating
    if summary_text.is_none() && !*is_generating {
        return html! {};
    }

    html! {
        <div class="space-y-3">
            <div class="flex justify-between items-center px-1">
                <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">{"Coach T Weekly Summary"}</h2>
                <button
                    onclick={on_regenerate}
                    disabled={*is_generating}
                    class="text-xs text-blue-600 dark:text-blue-400 hover:underline disabled:opacity-50"
                >
                    { if *is_generating { "Generating..." } else { "Regenerate" } }
                </button>
            </div>
            <div class="bg-gray-100 dark:bg-gray-800/50 rounded-2xl p-4 neu-flat transition-colors">
                { if *is_generating && summary_text.is_none() {
                    html! {
                        <div class="flex items-center gap-2 text-sm text-gray-500">
                            <div class="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style="animation-delay: 0ms;" />
                            <div class="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style="animation-delay: 150ms;" />
                            <div class="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style="animation-delay: 300ms;" />
                            <span class="ml-1">{"Generating summary..."}</span>
                        </div>
                    }
                } else if let Some(text) = &*summary_text {
                    html! {
                        <p class="text-sm text-gray-700 dark:text-gray-300 whitespace-pre-wrap">{text}</p>
                    }
                } else {
                    html! {}
                }}
            </div>
        </div>
    }
}

#[function_component(MuscleBalanceCard)]
fn muscle_balance_card() -> Html {
    let workouts = storage::load_workouts();
    if workouts.is_empty() {
        return html! {};
    }

    let all_exercises: Vec<Exercise> = {
        let mut exs = crate::data::default_exercises();
        exs.extend(storage::load_custom_exercises());
        exs
    };

    let (undertrained, overtrained) = muscle_balance_summary(&workouts, &all_exercises);

    let (summary, color) = if undertrained == 0 && overtrained == 0 {
        ("All muscles balanced".to_string(), "text-green-500")
    } else {
        let mut parts = Vec::new();
        if undertrained > 0 {
            parts.push(format!("{} undertrained", undertrained));
        }
        if overtrained > 0 {
            parts.push(format!("{} overtrained", overtrained));
        }
        (
            parts.join(", "),
            if overtrained > 0 {
                "text-red-500"
            } else {
                "text-yellow-500"
            },
        )
    };

    html! {
        <Link<Route> to={Route::Muscles} classes="block bg-gray-100 dark:bg-gray-800/50 rounded-xl p-4 hover:bg-gray-200 dark:hover:bg-gray-800 transition neu-flat group">
            <div class="flex justify-between items-center">
                <div class="flex items-center gap-3">
                    <span class="text-2xl group-hover:scale-110 transition-transform">{"🧠"}</span>
                    <div>
                        <div class="font-medium text-gray-800 dark:text-gray-200">{"Training Intelligence"}</div>
                        <div class={classes!("text-xs", "font-medium", color)}>{summary}</div>
                    </div>
                </div>
                <span class="text-gray-400 dark:text-gray-600 group-hover:text-blue-600 dark:group-hover:text-blue-400 group-hover:translate-x-1 transition-all">{"→"}</span>
            </div>
        </Link<Route>>
    }
}

#[function_component(HomePage)]
pub fn home_page() -> Html {
    let workouts = use_state(storage::load_workouts);
    let routines = use_state(storage::load_routines);
    let navigator = use_navigator().unwrap();

    let treen_taps = use_state(|| 0u32);
    let show_treen_toast = use_state(|| false);

    let on_title_click = {
        let treen_taps = treen_taps.clone();
        let show_treen_toast = show_treen_toast.clone();
        Callback::from(move |_: MouseEvent| {
            let new_count = *treen_taps + 1;
            treen_taps.set(new_count);
            if new_count >= 5 {
                treen_taps.set(0);
                // Activate Treen theme
                let mut config = storage::load_user_config();
                config.theme = crate::models::Theme::Treen;
                storage::save_user_config(&config);
                if let Some(html) = gloo::utils::document().document_element() {
                    let _ = html.set_attribute("class", "dark treen");
                }
                show_treen_toast.set(true);
                let toast = show_treen_toast.clone();
                gloo::timers::callback::Timeout::new(3_000, move || {
                    toast.set(false);
                })
                .forget();
            }
        })
    };

    let last_workout: Option<&Workout> = {
        let mut sorted: Vec<&Workout> = workouts.iter().collect();
        sorted.sort_by(|a, b| b.date.cmp(&a.date));
        sorted.first().copied()
    };

    let all_exercises: Vec<Exercise> = {
        let mut exs = crate::data::default_exercises();
        exs.extend(storage::load_custom_exercises());
        exs
    };

    let find_exercise = |id: &str| -> String {
        all_exercises
            .iter()
            .find(|e| e.id == id)
            .map(|e| e.name.clone())
            .unwrap_or_else(|| id.to_string())
    };

    let start_empty = {
        let nav = navigator.clone();
        Callback::from(move |_| {
            nav.push(&Route::Workout);
        })
    };

    html! {
        <div class="px-4 py-4 space-y-6">
            <div class="flex justify-between items-start">
                <div>
                    <h1 class="text-3xl font-bold cursor-pointer select-none" onclick={on_title_click}>{"Treening"}</h1>
                    <p class="text-gray-400 mt-1">{"Workout Tracker"}</p>
                </div>
            </div>

            <button
                class="w-full py-4 bg-blue-600 rounded-xl text-lg font-bold hover:bg-blue-700 active:bg-blue-800 transition neu-btn btn-press"
                onclick={start_empty}
            >{"Start New Workout"}</button>

            <div class="flex gap-3">
                <Link<Route> to={Route::PlateCalc} classes="flex-1 py-3 bg-gray-100 dark:bg-gray-800/50 rounded-xl text-center hover:bg-gray-200 dark:hover:bg-gray-800 transition neu-flat">
                    <div class="text-lg">{"🏋️"}</div>
                    <div class="text-xs font-bold text-gray-600 dark:text-gray-400">{"Plate Calc"}</div>
                </Link<Route>>
                { if storage::load_user_config().ai_enabled {
                    html! {
                        <Link<Route> to={Route::AiChat} classes="flex-1 py-3 bg-gray-100 dark:bg-gray-800/50 rounded-xl text-center hover:bg-gray-200 dark:hover:bg-gray-800 transition neu-flat">
                            <div class="text-lg">{"🏋️‍♂️"}</div>
                            <div class="text-xs font-bold text-gray-600 dark:text-gray-400">{"Coach T"}</div>
                        </Link<Route>>
                    }
                } else {
                    html! {}
                }}
            </div>

            <MuscleBalanceCard />

            <SummaryStats />

            <WeeklySummary />

            <AchievementBadges />

            <CommunitySummary />

            { if !routines.is_empty() {
                html! {
                    <div class="space-y-3">
                        <div class="flex justify-between items-center px-1">
                            <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">{"Routines"}</h2>
                            <Link<Route> to={Route::Routines} classes="text-xs text-blue-600 dark:text-blue-400 hover:underline">
                                {"Edit All →"}
                            </Link<Route>>
                        </div>
                        <div class="space-y-2">
                            { for routines.iter().take(2).map(|r| {
                                let nav = navigator.clone();
                                let routine = r.clone();
                                html! {
                                    <button
                                        class="w-full py-4 bg-gray-100 dark:bg-gray-800/50 rounded-xl text-left px-5 hover:bg-gray-200 dark:hover:bg-gray-800 transition flex justify-between items-center group neu-flat"
                                        onclick={Callback::from(move |_| {
                                            // Store routine ID to load on workout page
                                            let _ = LocalStorage::set("treening_active_routine", routine.id.clone());
                                            nav.push(&Route::Workout);
                                        })}
                                    >
                                        <div>
                                            <div class="font-bold text-gray-800 dark:text-gray-200">{&r.name}</div>
                                            <div class="text-xs text-gray-500 mt-0.5">{r.exercise_ids.len()}{" exercises"}</div>
                                        </div>
                                        <span class="text-gray-400 dark:text-gray-600 group-hover:text-blue-600 dark:group-hover:text-blue-400 group-hover:translate-x-1 transition-all">{"→"}</span>
                                    </button>
                                }
                            })}
                        </div>
                    </div>
                }
            } else { html! {} }}

            { if let Some(w) = last_workout {
                let total_sets: usize = w.exercises.iter().map(|e| e.sets.len()).sum();
                html! {
                    <div class="space-y-3">
                        <div class="flex justify-between items-center px-1">
                            <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">{"Last Workout"}</h2>
                            <Link<Route> to={Route::History} classes="text-xs text-blue-600 dark:text-blue-400 hover:underline">
                                {"History →"}
                            </Link<Route>>
                        </div>
                        <div class="bg-gray-100 dark:bg-gray-800/50 rounded-2xl p-4 neu-flat transition-colors">
                            <div class="flex justify-between mb-3 items-start">
                                <div>
                                    <span class="font-bold text-gray-800 dark:text-gray-200 block">{&w.name}</span>
                                    <span class="text-xs text-gray-500 font-medium font-mono uppercase tracking-wider">{&w.date}</span>
                                </div>
                                <span class="bg-blue-100 dark:bg-blue-600/20 text-blue-600 dark:text-blue-400 text-[10px] font-bold px-2 py-0.5 rounded-full uppercase">{"Summary"}</span>
                            </div>
                            <div class="text-xs text-gray-600 dark:text-gray-400 mb-4 flex gap-3">
                                <span class="flex items-center gap-1.5"><span class="opacity-70">{"💪"}</span> {w.exercises.len()}{" exercises"}</span>
                                <span class="flex items-center gap-1.5"><span class="opacity-70">{"⚡"}</span> {total_sets}{" sets"}</span>
                                { if w.duration_mins > 0 {
                                    html! { <span class="flex items-center gap-1.5"><span class="opacity-70">{"⏱️"}</span> {w.duration_mins}{"min"}</span> }
                                } else { html! {} }}
                            </div>
                            <div class="text-[11px] text-gray-500 space-y-1.5 border-t border-gray-200 dark:border-gray-700/50 pt-3">
                                { for w.exercises.iter().take(3).map(|we| {
                                    let name = find_exercise(&we.exercise_id);
                                    html! { <div class="flex justify-between"><span>{name}</span> <span class="text-gray-400 dark:text-gray-600">{we.sets.len()}{" sets"}</span></div> }
                                })}
                                { if w.exercises.len() > 3 {
                                    html! { <div class="text-gray-400 dark:text-gray-600 italic">{"and "}{w.exercises.len() - 3}{" more..."}</div> }
                                } else { html! {} }}
                            </div>
                        </div>
                    </div>
                }
            } else {
                html! {
                    <div class="space-y-6 py-4">
                        <div class="text-center">
                            <p class="text-2xl font-bold text-gray-900 dark:text-gray-100">{"Welcome to Treening!"}</p>
                            <p class="mt-2 text-gray-600 dark:text-gray-400">{"Your privacy-first, offline workout tracker."}</p>
                        </div>

                        <div class="bg-gray-100 dark:bg-gray-800/30 rounded-2xl p-6 space-y-6 neu-flat">
                            <h3 class="font-bold text-gray-800 dark:text-gray-200">{"Quick Start Guide"}</h3>
                            <div class="space-y-4">
                                <div class="flex gap-4">
                                    <div class="flex-shrink-0 w-8 h-8 bg-blue-600 text-white rounded-full flex items-center justify-center font-bold">{"1"}</div>
                                    <div>
                                        <div class="font-bold text-gray-800 dark:text-gray-200">{"Explore Exercises"}</div>
                                        <p class="text-sm text-gray-600 dark:text-gray-400">{"Browse over 80 built-in exercises with muscle group info and images."}</p>
                                    </div>
                                </div>
                                <div class="flex gap-4">
                                    <div class="flex-shrink-0 w-8 h-8 bg-blue-600 text-white rounded-full flex items-center justify-center font-bold">{"2"}</div>
                                    <div>
                                        <div class="font-bold text-gray-800 dark:text-gray-200">{"Create a Routine"}</div>
                                        <p class="text-sm text-gray-600 dark:text-gray-400">{"Save your favorite workouts (e.g., 'Push Day') for one-tap starting."}</p>
                                    </div>
                                </div>
                                <div class="flex gap-4">
                                    <div class="flex-shrink-0 w-8 h-8 bg-blue-600 text-white rounded-full flex items-center justify-center font-bold">{"3"}</div>
                                    <div>
                                        <div class="font-bold text-gray-800 dark:text-gray-200">{"Log Your Session"}</div>
                                        <p class="text-sm text-gray-600 dark:text-gray-400">{"Track sets, reps, and weight. Your data stays 100% on your device."}</p>
                                    </div>
                                </div>
                                <div class="flex gap-4">
                                    <div class="flex-shrink-0 w-8 h-8 bg-blue-600 text-white rounded-full flex items-center justify-center font-bold">{"4"}</div>
                                    <div>
                                        <div class="font-bold text-gray-800 dark:text-gray-200">{"Track Body Progress"}</div>
                                        <p class="text-sm text-gray-600 dark:text-gray-400">{"Log your weight in Settings to unlock 'Relative Volume' stats and charts."}</p>
                                    </div>
                                </div>
                                <div class="flex gap-4">
                                    <div class="flex-shrink-0 w-8 h-8 bg-blue-600 text-white rounded-full flex items-center justify-center font-bold">{"5"}</div>
                                    <div>
                                        <div class="font-bold text-gray-800 dark:text-gray-200">{"Meet Coach T"}</div>
                                        <p class="text-sm text-gray-600 dark:text-gray-400">{"Enable Coach T in Settings for a personal AI coach. Ask for advice, log workouts via chat, and get weekly summaries — all offline."}</p>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                }
            }}

            <div class="pt-4 border-t border-gray-200 dark:border-gray-800">
                <Link<Route> to={Route::Settings} classes="flex items-center justify-between p-4 bg-gray-100 dark:bg-gray-800/50 rounded-xl hover:bg-gray-200 dark:hover:bg-gray-800 transition group neu-flat transition-colors">
                    <div class="flex items-center gap-3">
                        <span class="text-2xl group-hover:scale-110 transition-transform">{"⚙️"}</span>
                        <div>
                            <div class="font-medium text-gray-800 dark:text-gray-200">{"Settings & Sync"}</div>
                            <div class="text-xs text-gray-500">{"App preferences, data backup, and P2P sync"}</div>
                        </div>
                    </div>
                    <span class="text-gray-400 dark:text-gray-600 group-hover:text-blue-600 dark:group-hover:text-blue-400 group-hover:translate-x-1 transition-all">{"→"}</span>
                </Link<Route>>
            </div>

            { if *show_treen_toast {
                html! {
                    <div class="fixed top-4 left-4 right-4 z-50 bg-amber-800 text-amber-100 px-4 py-3 rounded-xl shadow-lg text-center font-bold text-sm" style="animation: modalContentIn 200ms ease-out;">
                        {"🪵 Treen Mode activated! Wood grain unlocked."}
                    </div>
                }
            } else {
                html! {}
            }}
        </div>
    }
}
