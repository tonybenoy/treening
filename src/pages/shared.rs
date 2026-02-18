use yew::prelude::*;
use crate::sharing::{self, ShareableData};
use crate::storage;
use crate::data;

#[function_component(SharedPage)]
pub fn shared_page() -> Html {
    let parsed = use_state(|| None::<Result<ShareableData, String>>);
    let imported = use_state(|| false);

    {
        let parsed = parsed.clone();
        use_effect_with((), move |_| {
            let window = gloo::utils::window();
            let hash = window.location().hash().unwrap_or_default();
            // Hash format: #/shared?d=ENCODED
            let result = if let Some(idx) = hash.find("d=") {
                let encoded = &hash[idx + 2..];
                // strip any trailing & params
                let encoded = encoded.split('&').next().unwrap_or(encoded);
                sharing::decode(encoded)
            } else {
                Err("No shared data found in URL".to_string())
            };
            parsed.set(Some(result));
            || ()
        });
    }

    let content = match &*parsed {
        None => html! { <p class="text-gray-500 text-center py-12">{"Loading..."}</p> },
        Some(Err(e)) => html! {
            <div class="text-center py-12 px-4">
                <p class="text-red-500 dark:text-red-400 font-bold mb-2">{"Failed to load shared data"}</p>
                <p class="text-gray-500 text-sm">{e}</p>
            </div>
        },
        Some(Ok(data)) => {
            match data {
                ShareableData::Workout { workout, exercises } => {
                    render_workout(workout, exercises, &imported)
                }
                ShareableData::Routine { routine, exercises } => {
                    render_routine(routine, exercises, &imported)
                }
                ShareableData::Exercise { exercise } => {
                    render_exercise(exercise, &imported)
                }
            }
        }
    };

    html! {
        <div class="pb-20 px-4 pt-4">
            <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100 mb-4">{"Shared with you"}</h1>
            {content}
        </div>
    }
}

fn render_workout(workout: &crate::models::Workout, exercises: &[crate::models::Exercise], imported: &UseStateHandle<bool>) -> Html {
    let find_name = |id: &str| -> String {
        exercises.iter().find(|e| e.id == id).map(|e| e.name.clone()).unwrap_or_else(|| id.to_string())
    };

    let workout_c = workout.clone();
    let exercises_c = exercises.to_vec();
    let imported_c = imported.clone();
    let on_import = Callback::from(move |_| {
        // Import custom exercises first
        let mut custom = storage::load_custom_exercises();
        let defaults = data::default_exercises();
        for ex in &exercises_c {
            let exists = custom.iter().any(|e| e.id == ex.id) || defaults.iter().any(|e| e.id == ex.id);
            if !exists {
                custom.push(ex.clone());
            }
        }
        storage::save_custom_exercises(&custom);

        // Import workout
        let mut workouts = storage::load_workouts();
        if !workouts.iter().any(|w| w.id == workout_c.id) {
            workouts.push(workout_c.clone());
        }
        storage::save_workouts(&workouts);
        imported_c.set(true);
    });

    html! {
        <div class="space-y-4">
            <div class="bg-gray-100 dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-transparent">
                <h2 class="text-xl font-bold text-gray-900 dark:text-gray-100 mb-1">{&workout.name}</h2>
                <div class="text-sm text-gray-500 dark:text-gray-400 mb-3">
                    <span>{&workout.date}</span>
                    { if workout.duration_mins > 0 {
                        html! { <span>{" · "}{workout.duration_mins}{" min"}</span> }
                    } else { html! {} }}
                </div>

                { for workout.exercises.iter().map(|we| {
                    let name = find_name(&we.exercise_id);
                    html! {
                        <div class="mb-3">
                            <div class="font-bold text-sm text-gray-800 dark:text-gray-200 mb-1">{name}</div>
                            <div class="space-y-0.5">
                                { for we.sets.iter().enumerate().map(|(i, s)| {
                                    html! {
                                        <div class="text-xs text-gray-600 dark:text-gray-400 ml-2">
                                            <span class="font-medium">{"Set "}{i+1}{": "}</span>
                                            { if let Some(dist) = s.distance {
                                                let dur = s.duration_secs.unwrap_or(0);
                                                html! { <span class="font-bold text-gray-800 dark:text-gray-200">{format!("{:.1}km / {}:{:02}", dist, dur / 60, dur % 60)}</span> }
                                            } else if let Some(secs) = s.duration_secs {
                                                html! { <span class="font-bold text-gray-800 dark:text-gray-200">{format!("{}:{:02}", secs / 60, secs % 60)}</span> }
                                            } else {
                                                html! { <span class="font-bold text-gray-800 dark:text-gray-200">{s.weight}{"kg x "}{s.reps}</span> }
                                            }}
                                            { if s.completed { html!{<span class="text-green-600 dark:text-green-400">{" ✓"}</span>} } else { html!{} } }
                                        </div>
                                    }
                                })}
                            </div>
                        </div>
                    }
                })}

                { if workout.total_volume() > 0.0 {
                    html! { <div class="text-sm font-bold text-gray-700 dark:text-gray-300 border-t border-gray-200 dark:border-gray-700 pt-2 mt-2">{"Total volume: "}{format!("{:.0}", workout.total_volume())}{" kg"}</div> }
                } else { html! {} }}
            </div>

            { if **imported {
                html! {
                    <div class="bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg p-3 text-center">
                        <p class="text-green-700 dark:text-green-400 font-bold text-sm">{"Imported successfully!"}</p>
                        <a href="#/" class="text-blue-600 dark:text-blue-400 text-sm hover:underline">{"Go to Home"}</a>
                    </div>
                }
            } else {
                html! {
                    <button
                        class="w-full py-3 bg-blue-600 text-white rounded-lg font-bold hover:bg-blue-700 shadow-sm transition-colors"
                        onclick={on_import}
                    >{"Import Workout"}</button>
                }
            }}
        </div>
    }
}

fn render_routine(routine: &crate::models::Routine, exercises: &[crate::models::Exercise], imported: &UseStateHandle<bool>) -> Html {
    let routine_c = routine.clone();
    let exercises_c = exercises.to_vec();
    let imported_c = imported.clone();
    let on_import = Callback::from(move |_| {
        let mut custom = storage::load_custom_exercises();
        let defaults = data::default_exercises();
        for ex in &exercises_c {
            let exists = custom.iter().any(|e| e.id == ex.id) || defaults.iter().any(|e| e.id == ex.id);
            if !exists {
                custom.push(ex.clone());
            }
        }
        storage::save_custom_exercises(&custom);

        let mut routines = storage::load_routines();
        if !routines.iter().any(|r| r.id == routine_c.id) {
            routines.push(routine_c.clone());
        }
        storage::save_routines(&routines);
        imported_c.set(true);
    });

    html! {
        <div class="space-y-4">
            <div class="bg-gray-100 dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-transparent">
                <h2 class="text-xl font-bold text-gray-900 dark:text-gray-100 mb-3">{&routine.name}</h2>
                <div class="space-y-2">
                    { for routine.exercise_ids.iter().map(|eid| {
                        if let Some(ex) = exercises.iter().find(|e| &e.id == eid) {
                            html! {
                                <div class="flex items-center gap-2 text-sm text-gray-700 dark:text-gray-300">
                                    <span class="font-medium">{&ex.name}</span>
                                    <span class="px-1.5 py-0.5 bg-blue-100 dark:bg-blue-900/40 text-blue-700 dark:text-blue-300 rounded text-[10px] font-bold uppercase">{ex.category.to_string()}</span>
                                    <span class="px-1.5 py-0.5 bg-gray-200 dark:bg-gray-700 text-gray-600 dark:text-gray-400 rounded text-[10px] font-bold uppercase">{ex.equipment.to_string()}</span>
                                </div>
                            }
                        } else {
                            html! { <div class="text-sm text-gray-500">{eid}</div> }
                        }
                    })}
                </div>
            </div>

            { if **imported {
                html! {
                    <div class="bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg p-3 text-center">
                        <p class="text-green-700 dark:text-green-400 font-bold text-sm">{"Imported successfully!"}</p>
                        <a href="#/routines" class="text-blue-600 dark:text-blue-400 text-sm hover:underline">{"Go to Routines"}</a>
                    </div>
                }
            } else {
                html! {
                    <button
                        class="w-full py-3 bg-blue-600 text-white rounded-lg font-bold hover:bg-blue-700 shadow-sm transition-colors"
                        onclick={on_import}
                    >{"Import Routine"}</button>
                }
            }}
        </div>
    }
}

fn render_exercise(exercise: &crate::models::Exercise, imported: &UseStateHandle<bool>) -> Html {
    let exercise_c = exercise.clone();
    let imported_c = imported.clone();
    let on_import = Callback::from(move |_| {
        let mut custom = storage::load_custom_exercises();
        let defaults = data::default_exercises();
        let exists = custom.iter().any(|e| e.id == exercise_c.id) || defaults.iter().any(|e| e.id == exercise_c.id);
        if !exists {
            custom.push(exercise_c.clone());
        }
        storage::save_custom_exercises(&custom);
        imported_c.set(true);
    });

    html! {
        <div class="space-y-4">
            <div class="bg-gray-100 dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-transparent">
                <h2 class="text-xl font-bold text-gray-900 dark:text-gray-100 mb-2">{&exercise.name}</h2>
                <div class="flex flex-wrap gap-2 mb-4">
                    <span class="px-2 py-1 bg-blue-100 dark:bg-blue-900/40 text-blue-700 dark:text-blue-300 border border-blue-200 dark:border-transparent rounded text-xs font-bold uppercase">{exercise.category.to_string()}</span>
                    <span class="px-2 py-1 bg-gray-200 dark:bg-gray-700 text-gray-600 dark:text-gray-300 border border-gray-200 dark:border-transparent rounded text-xs font-bold uppercase">{exercise.equipment.to_string()}</span>
                </div>
                { if !exercise.muscle_groups.is_empty() {
                    html! {
                        <div class="mb-4">
                            <h3 class="text-sm font-bold text-gray-700 dark:text-gray-300 mb-1">{"Muscles"}</h3>
                            <div class="flex flex-wrap gap-1">
                                { for exercise.muscle_groups.iter().map(|m| html! {
                                    <span class="px-2 py-0.5 bg-gray-200 dark:bg-gray-700 rounded text-xs text-gray-700 dark:text-gray-300">{m}</span>
                                })}
                            </div>
                        </div>
                    }
                } else { html! {} }}
                { if !exercise.description.is_empty() {
                    html! {
                        <div>
                            <h3 class="text-sm font-bold text-gray-700 dark:text-gray-300 mb-1">{"Description"}</h3>
                            <p class="text-sm text-gray-600 dark:text-gray-400 leading-relaxed">{&exercise.description}</p>
                        </div>
                    }
                } else { html! {} }}
            </div>

            { if **imported {
                html! {
                    <div class="bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg p-3 text-center">
                        <p class="text-green-700 dark:text-green-400 font-bold text-sm">{"Imported successfully!"}</p>
                        <a href="#/exercises" class="text-blue-600 dark:text-blue-400 text-sm hover:underline">{"Go to Exercises"}</a>
                    </div>
                }
            } else {
                html! {
                    <button
                        class="w-full py-3 bg-blue-600 text-white rounded-lg font-bold hover:bg-blue-700 shadow-sm transition-colors"
                        onclick={on_import}
                    >{"Import Exercise"}</button>
                }
            }}
        </div>
    }
}
