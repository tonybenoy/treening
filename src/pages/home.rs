use crate::models::{Exercise, Workout};
use crate::storage;
use crate::Route;
use gloo::storage::{LocalStorage, Storage};
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

    let streak = {
        let mut dates: Vec<chrono::NaiveDate> = workouts
            .iter()
            .filter_map(|w| chrono::NaiveDate::parse_from_str(&w.date, "%Y-%m-%d").ok())
            .collect();
        dates.sort();
        dates.dedup();

        if dates.is_empty() {
            0
        } else {
            let today = chrono::Local::now().date_naive();
            let last = *dates.last().unwrap();

            if (today - last).num_days() > 1 {
                0
            } else {
                let mut s = 1u32;
                for i in (0..dates.len() - 1).rev() {
                    if (dates[i + 1] - dates[i]).num_days() == 1 {
                        s += 1;
                    } else {
                        break;
                    }
                }
                s
            }
        }
    };

    html! {
        <div class="space-y-3">
            <div class="flex justify-between items-center px-1">
                <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">{"Your Progress"}</h2>
                <Link<Route> to={Route::Analytics} classes="text-xs text-blue-600 dark:text-blue-400 hover:underline">
                    {"Full Analytics →"}
                </Link<Route>>
            </div>
            <div class="grid grid-cols-3 gap-3">
                <div class="bg-gray-100 dark:bg-gray-800/50 rounded-xl p-3 border border-gray-200 dark:border-gray-700/50 text-center transition-colors">
                    <div class="text-xl mb-1">{"🏋️"}</div>
                    <div class="text-lg font-bold text-gray-800 dark:text-gray-200">{total_workouts}</div>
                    <div class="text-[10px] text-gray-500 dark:text-gray-500 uppercase font-bold">{"Sessions"}</div>
                </div>
                <div class="bg-gray-100 dark:bg-gray-800/50 rounded-xl p-3 border border-gray-200 dark:border-gray-700/50 text-center transition-colors">
                    <div class="text-xl mb-1">{"💪"}</div>
                    <div class="text-lg font-bold text-gray-800 dark:text-gray-200">{volume_display}</div>
                    <div class="text-[10px] text-gray-500 dark:text-gray-500 uppercase font-bold">{volume_label}</div>
                </div>
                <div class="bg-gray-100 dark:bg-gray-800/50 rounded-xl p-3 border border-gray-200 dark:border-gray-700/50 text-center transition-colors">
                    <div class="text-xl mb-1">{"🔥"}</div>
                    <div class="text-lg font-bold text-gray-800 dark:text-gray-200">{streak}</div>
                    <div class="text-[10px] text-gray-500 dark:text-gray-500 uppercase font-bold">{"Day Streak"}</div>
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
            <div class="bg-gradient-to-br from-gray-100 to-gray-200 dark:from-gray-800 dark:to-gray-900 rounded-2xl p-4 border border-gray-200 dark:border-gray-700/50 transition-colors">
                <div class="flex justify-between items-center mb-3">
                    <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">{"Community"}</h2>
                    <span class="px-2 py-0.5 bg-blue-100 dark:bg-blue-900/40 text-blue-600 dark:text-blue-400 text-[10px] font-bold rounded-full uppercase tracking-wider">{"Optional"}</span>
                </div>
                <p class="text-sm text-gray-600 dark:text-gray-400 mb-4">{"Train with friends, share stats, and see your rankings in the community."}</p>
                <Link<Route> to={Route::Social} classes="flex items-center justify-center w-full py-2 bg-white dark:bg-gray-700 hover:bg-gray-50 dark:hover:bg-gray-600 text-gray-900 dark:text-white border border-gray-200 dark:border-transparent rounded-lg text-sm font-bold transition shadow-sm">
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

            <div class="bg-gray-100 dark:bg-gray-800/50 rounded-2xl p-4 border border-gray-200 dark:border-gray-700/50 transition-colors">
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

#[function_component(HomePage)]
pub fn home_page() -> Html {
    let workouts = use_state(storage::load_workouts);
    let routines = use_state(storage::load_routines);
    let navigator = use_navigator().unwrap();

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
                    <h1 class="text-3xl font-bold">{"Treening"}</h1>
                    <p class="text-gray-400 mt-1">{"Workout Tracker"}</p>
                </div>
            </div>

            <button
                class="w-full py-4 bg-blue-600 rounded-xl text-lg font-bold hover:bg-blue-700 active:bg-blue-800 transition shadow-lg shadow-blue-900/20 btn-press"
                onclick={start_empty}
            >{"Start New Workout"}</button>

            <SummaryStats />

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
                                        class="w-full py-4 bg-gray-100 dark:bg-gray-800/50 rounded-xl text-left px-5 hover:bg-gray-200 dark:hover:bg-gray-800 border border-gray-200 dark:border-gray-700/50 transition flex justify-between items-center group shadow-sm"
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
                        <div class="bg-gray-100 dark:bg-gray-800/50 rounded-2xl p-4 border border-gray-200 dark:border-gray-700/50 shadow-sm transition-colors">
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

                        <div class="bg-gray-100 dark:bg-gray-800/30 border border-gray-200 dark:border-gray-800 rounded-2xl p-6 space-y-6 shadow-sm">
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
                            </div>
                        </div>
                    </div>
                }
            }}

            <div class="pt-4 border-t border-gray-200 dark:border-gray-800">
                <Link<Route> to={Route::Settings} classes="flex items-center justify-between p-4 bg-gray-100 dark:bg-gray-800/50 rounded-xl hover:bg-gray-200 dark:hover:bg-gray-800 transition group border border-gray-200 dark:border-gray-700/50 shadow-sm transition-colors">
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
        </div>
    }
}
