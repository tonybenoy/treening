use yew::prelude::*;
use yew_router::prelude::*;
use gloo::storage::{LocalStorage, Storage};
use crate::models::{Exercise, Workout};
use crate::storage;
use crate::Route;

#[function_component(HomePage)]
pub fn home_page() -> Html {
    let workouts = use_state(|| storage::load_workouts());
    let routines = use_state(|| storage::load_routines());
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
        all_exercises.iter()
            .find(|e| e.id == id)
            .map(|e| e.name.clone())
            .unwrap_or_else(|| id.to_string())
    };

    let start_empty = {
        let nav = navigator.clone();
        Callback::from(move |_| { nav.push(&Route::Workout); })
    };

    html! {
        <div class="px-4 py-4 space-y-6">
            <div>
                <h1 class="text-3xl font-bold">{"Treening"}</h1>
                <p class="text-gray-400 mt-1">{"Workout Tracker"}</p>
            </div>

            <button
                class="w-full py-4 bg-blue-600 rounded-xl text-lg font-bold hover:bg-blue-700 active:bg-blue-800 transition"
                onclick={start_empty}
            >{"Start New Workout"}</button>

            { if !routines.is_empty() {
                html! {
                    <div>
                        <h2 class="text-lg font-semibold mb-2">{"Quick Start from Routine"}</h2>
                        <div class="space-y-2">
                            { for routines.iter().map(|r| {
                                let nav = navigator.clone();
                                let routine = r.clone();
                                html! {
                                    <button
                                        class="w-full py-3 bg-gray-800 rounded-lg text-left px-4 hover:bg-gray-700"
                                        onclick={Callback::from(move |_| {
                                            // Store routine ID to load on workout page
                                            let _ = LocalStorage::set("treening_active_routine", routine.id.clone());
                                            nav.push(&Route::Workout);
                                        })}
                                    >
                                        <div class="font-medium">{&r.name}</div>
                                        <div class="text-sm text-gray-400">{r.exercise_ids.len()}{" exercises"}</div>
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
                    <div>
                        <h2 class="text-lg font-semibold mb-2">{"Last Workout"}</h2>
                        <div class="bg-gray-800 rounded-lg p-4">
                            <div class="flex justify-between mb-2">
                                <span class="font-medium">{&w.name}</span>
                                <span class="text-sm text-gray-400">{&w.date}</span>
                            </div>
                            <div class="text-sm text-gray-400 mb-2">
                                {w.exercises.len()}{" exercises · "}{total_sets}{" sets"}
                                { if w.duration_mins > 0 {
                                    html! { <>{" · "}{w.duration_mins}{"min"}</> }
                                } else { html! {} }}
                            </div>
                            <div class="text-sm text-gray-500">
                                { for w.exercises.iter().take(5).map(|we| {
                                    let name = find_exercise(&we.exercise_id);
                                    html! { <div>{name}{" - "}{we.sets.len()}{" sets"}</div> }
                                })}
                                { if w.exercises.len() > 5 {
                                    html! { <div class="text-gray-600">{"+"}{w.exercises.len() - 5}{" more..."}</div> }
                                } else { html! {} }}
                            </div>
                        </div>
                    </div>
                }
            } else {
                html! {
                    <div class="text-center text-gray-500 py-8">
                        <p class="text-lg">{"Welcome to Treening!"}</p>
                        <p class="mt-1">{"Start your first workout or browse exercises."}</p>
                    </div>
                }
            }}
        </div>
    }
}
