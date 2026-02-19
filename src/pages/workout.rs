use crate::components::exercise_list::ExerciseList;
use crate::components::workout_log::WorkoutLog;
use crate::data;
use crate::models::{Exercise, ExerciseTrackingType, Workout, WorkoutExercise, WorkoutSet};
use crate::storage;
use crate::Route;
use gloo::storage::{LocalStorage, Storage};
use gloo::timers::callback::{Interval, Timeout};
use wasm_bindgen::prelude::*;
use yew::prelude::*;
use yew_router::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = navigator, js_name = vibrate)]
    fn vibrate(ms: u32) -> bool;
}

fn try_vibrate() {
    let window = web_sys::window().unwrap();
    let navigator = window.navigator();
    let nav_val: JsValue = navigator.into();
    let _ = js_sys::Reflect::get(&nav_val, &"vibrate".into())
        .ok()
        .and_then(|f| {
            let func = js_sys::Function::from(f);
            func.call1(&nav_val, &JsValue::from(200)).ok()
        });
}

/// Auto-fill a set from the most recent previous workout containing this exercise.
fn autofill_set(previous: &[Workout], exercise_id: &str, all_exercises: &[Exercise]) -> WorkoutSet {
    let tracking = all_exercises
        .iter()
        .find(|e| e.id == exercise_id)
        .map(|e| e.tracking_type.clone())
        .unwrap_or(ExerciseTrackingType::Strength);

    let prev_set = previous
        .iter()
        .rev()
        .flat_map(|w| w.exercises.iter())
        .find(|we| we.exercise_id == exercise_id)
        .and_then(|we| we.sets.first());

    match prev_set {
        Some(s) => WorkoutSet {
            weight: s.weight,
            reps: s.reps,
            distance: s.distance,
            duration_secs: s.duration_secs,
            completed: false,
            note: None,
        },
        None => match tracking {
            ExerciseTrackingType::Cardio => WorkoutSet {
                weight: 0.0,
                reps: 0,
                completed: false,
                distance: Some(0.0),
                duration_secs: Some(0),
                note: None,
            },
            ExerciseTrackingType::Duration => WorkoutSet {
                weight: 0.0,
                reps: 0,
                completed: false,
                distance: None,
                duration_secs: Some(0),
                note: None,
            },
            _ => WorkoutSet {
                weight: 0.0,
                reps: 10,
                completed: false,
                distance: None,
                duration_secs: None,
                note: None,
            },
        },
    }
}

/// Generate warm-up sets for a given working weight.
/// Percentages: [40%, 60%, 75%, 90%], reps: [10, 6, 4, 2], rounded to nearest 2.5kg
pub fn generate_warmup_sets(working_weight: f64) -> Vec<WorkoutSet> {
    let percentages = [0.40, 0.60, 0.75, 0.90];
    let reps = [10u32, 6, 4, 2];

    percentages
        .iter()
        .zip(reps.iter())
        .map(|(&pct, &r)| {
            let raw = working_weight * pct;
            let rounded = (raw / 2.5).round() * 2.5;
            WorkoutSet {
                weight: rounded,
                reps: r,
                completed: false,
                distance: None,
                duration_secs: None,
                note: None,
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Self-contained timer components – they own their own Interval + state so
// ticking never re-renders the parent WorkoutPage.
// ---------------------------------------------------------------------------

fn format_time(secs: u32) -> String {
    let m = secs / 60;
    let s = secs % 60;
    format!("{:02}:{:02}", m, s)
}

/// Displays the elapsed workout time. Fully self-contained: manages its own
/// Interval and display state so ticking never re-renders the parent.
#[derive(Properties, PartialEq)]
pub struct ElapsedTimerProps {
    pub active: bool,
}

#[function_component(ElapsedTimer)]
pub fn elapsed_timer(props: &ElapsedTimerProps) -> Html {
    let seconds = use_state(|| 0u32);
    {
        let seconds = seconds.clone();
        let active = props.active;
        use_effect_with(active, move |active| {
            let interval = if *active {
                Some(Interval::new(1000, move || {
                    seconds.set(*seconds + 1);
                }))
            } else {
                None
            };
            move || drop(interval)
        });
    }
    html! {
        <div class="text-xl font-mono text-blue-600 dark:text-blue-400 font-bold">
            {format_time(*seconds)}
        </div>
    }
}

/// Self-contained rest-timer bar. Triggered by a `(counter, seconds)` prop.
/// When the counter changes, a new countdown starts.
#[derive(Properties, PartialEq)]
pub struct RestTimerProps {
    pub trigger: (u32, u32), // (counter, rest_seconds)
}

#[function_component(RestTimer)]
pub fn rest_timer(props: &RestTimerProps) -> Html {
    let remaining = use_state(|| 0u32);
    let active = use_state(|| false);
    let last_counter = use_mut_ref(|| 0u32);

    // Detect new trigger
    {
        let remaining = remaining.clone();
        let active = active.clone();
        let last_counter = last_counter.clone();
        let (counter, seconds) = props.trigger;
        use_effect_with((counter, seconds), move |(counter, seconds)| {
            let mut lc = last_counter.borrow_mut();
            if *counter > 0 && *counter != *lc {
                *lc = *counter;
                remaining.set(*seconds);
                active.set(true);
            }
            || ()
        });
    }

    // Countdown interval
    {
        let remaining = remaining.clone();
        let active_handle = active.clone();
        use_effect_with(*active, move |is_active| {
            let interval = if *is_active {
                Some(Interval::new(1000, move || {
                    let r = *remaining;
                    if r <= 1 {
                        remaining.set(0);
                        active_handle.set(false);
                        try_vibrate();
                    } else {
                        remaining.set(r - 1);
                    }
                }))
            } else {
                None
            };
            move || drop(interval)
        });
    }

    if !*active && *remaining == 0 {
        return html! {};
    }

    let r = *remaining;
    let remaining_add = remaining.clone();
    let remaining_skip = remaining.clone();
    let active_skip = active.clone();

    html! {
        <div class="fixed bottom-16 left-0 right-0 z-50 px-4 pb-2">
            <div class="bg-gray-900 dark:bg-gray-700 rounded-xl px-4 py-3 flex items-center justify-between shadow-lg border border-gray-700 dark:border-gray-600">
                <div class="flex items-center gap-3">
                    <span class="text-xs text-gray-400 uppercase font-bold">{"Rest"}</span>
                    <span class="text-xl font-mono text-white font-bold">{format_time(r)}</span>
                </div>
                <div class="flex items-center gap-2">
                    <button
                        class="text-xs font-bold text-blue-400 bg-blue-400/10 px-2.5 py-1 rounded-lg hover:bg-blue-400/20 transition-colors"
                        onclick={Callback::from(move |_| remaining_add.set(*remaining_add + 30))}
                    >{"+30s"}</button>
                    <button
                        class="text-xs font-bold text-gray-400 bg-gray-600 px-2.5 py-1 rounded-lg hover:bg-gray-500 transition-colors"
                        onclick={Callback::from(move |_| {
                            remaining_skip.set(0);
                            active_skip.set(false);
                        })}
                    >{"Skip"}</button>
                </div>
            </div>
        </div>
    }
}

#[function_component(WorkoutPage)]
pub fn workout_page() -> Html {
    let workout_exercises = use_state(Vec::<WorkoutExercise>::new);
    let workout_name = use_state(|| "Workout".to_string());
    let show_exercise_picker = use_state(|| false);
    let elapsed_ref = use_mut_ref(|| 0u32);
    let workout_active = use_state(|| false);
    let saved = use_state(|| false);
    let navigator = use_navigator().unwrap();

    // Rest timer trigger: incremented to signal RestTimer to start
    let rest_trigger = use_state(|| (0u32, 0u32)); // (counter, seconds)

    // Undo state
    let undo_snapshot = use_state(|| None::<Vec<WorkoutExercise>>);
    let undo_timeout = use_state(|| None::<Timeout>);

    let config = use_memo((), |_| storage::load_user_config());
    let previous_workouts = use_memo((), |_| storage::load_workouts());

    let custom_exercises = storage::load_custom_exercises();
    let all_exercises: Vec<Exercise> = {
        let mut exs = data::default_exercises();
        exs.extend(custom_exercises);
        exs
    };

    // Load from routine if set
    {
        let workout_exercises = workout_exercises.clone();
        let workout_name = workout_name.clone();
        let workout_active = workout_active.clone();
        let previous = (*previous_workouts).clone();
        let all_ex = all_exercises.clone();
        use_effect_with((), move |_| {
            if let Ok(routine_id) = LocalStorage::get::<String>("treening_active_routine") {
                LocalStorage::delete("treening_active_routine");
                let routines = storage::load_routines();
                if let Some(routine) = routines.iter().find(|r| r.id == routine_id) {
                    workout_name.set(routine.name.clone());
                    let exs: Vec<WorkoutExercise> = routine
                        .exercise_ids
                        .iter()
                        .map(|eid| {
                            let set = autofill_set(&previous, eid, &all_ex);
                            WorkoutExercise {
                                exercise_id: eid.clone(),
                                sets: vec![set],
                                notes: String::new(),
                                superset_group: None,
                                rest_seconds_override: None,
                            }
                        })
                        .collect();
                    workout_exercises.set(exs);
                    workout_active.set(true);
                }
            }
            // Load from repeat if set
            if let Ok(repeat_json) = LocalStorage::get::<String>("treening_active_repeat") {
                LocalStorage::delete("treening_active_repeat");
                if let Ok(exs) = serde_json::from_str::<Vec<WorkoutExercise>>(&repeat_json) {
                    // Reset completed status on all sets
                    let exs: Vec<WorkoutExercise> = exs
                        .into_iter()
                        .map(|mut we| {
                            for s in we.sets.iter_mut() {
                                s.completed = false;
                            }
                            we
                        })
                        .collect();
                    workout_exercises.set(exs);
                    workout_active.set(true);
                }
            }
            || ()
        });
    }

    // Elapsed timer — uses a ref so ticking doesn't re-render the page.
    // An Interval writes to the ref; the ElapsedTimer component has its own
    // state for display.
    {
        let elapsed_ref = elapsed_ref.clone();
        let active = workout_active.clone();
        use_effect_with((*active,), move |(active,)| {
            let interval = if *active {
                Some(Interval::new(1000, move || {
                    let mut r = elapsed_ref.borrow_mut();
                    *r += 1;
                }))
            } else {
                None
            };
            move || drop(interval)
        });
    }

    // on_set_completed now receives resolved rest seconds
    let on_set_completed = {
        let rest_trigger = rest_trigger.clone();
        Callback::from(move |seconds: u32| {
            let (counter, _) = *rest_trigger;
            rest_trigger.set((counter + 1, seconds));
        })
    };

    let on_add_exercise = {
        let we = workout_exercises.clone();
        let show = show_exercise_picker.clone();
        let active = workout_active.clone();
        let previous = (*previous_workouts).clone();
        let all_ex = all_exercises.clone();
        Callback::from(move |ex: Exercise| {
            let mut exs = (*we).clone();
            let set = autofill_set(&previous, &ex.id, &all_ex);
            exs.push(WorkoutExercise {
                exercise_id: ex.id,
                sets: vec![set],
                notes: String::new(),
                superset_group: None,
                rest_seconds_override: None,
            });
            we.set(exs);
            show.set(false);
            active.set(true);
        })
    };

    let on_update = {
        let we = workout_exercises.clone();
        Callback::from(move |exs: Vec<WorkoutExercise>| {
            we.set(exs);
        })
    };

    let on_remove = {
        let we = workout_exercises.clone();
        let undo_snapshot = undo_snapshot.clone();
        let undo_timeout = undo_timeout.clone();
        Callback::from(move |idx: usize| {
            let mut exs = (*we).clone();
            if idx < exs.len() {
                // Save snapshot for undo
                undo_snapshot.set(Some((*we).clone()));
                exs.remove(idx);
                // Auto-dismiss undo after 5s
                let snap = undo_snapshot.clone();
                let timeout = Timeout::new(5000, move || {
                    snap.set(None);
                });
                undo_timeout.set(Some(timeout));
            }
            we.set(exs);
        })
    };

    // Undo callback for destructive actions (set deletions from WorkoutLog)
    let on_before_destructive = {
        let undo_snapshot = undo_snapshot.clone();
        let undo_timeout = undo_timeout.clone();
        Callback::from(move |snapshot: Vec<WorkoutExercise>| {
            undo_snapshot.set(Some(snapshot));
            let snap = undo_snapshot.clone();
            let timeout = Timeout::new(5000, move || {
                snap.set(None);
            });
            undo_timeout.set(Some(timeout));
        })
    };

    let on_save = {
        let we = workout_exercises.clone();
        let name = workout_name.clone();
        let elapsed_ref = elapsed_ref.clone();
        let saved = saved.clone();
        let nav = navigator.clone();
        Callback::from(move |_| {
            if we.is_empty() {
                return;
            }
            let now = chrono::Local::now();
            let elapsed = *elapsed_ref.borrow();
            let workout = Workout {
                id: uuid::Uuid::new_v4().to_string(),
                date: now.format("%Y-%m-%d").to_string(),
                name: (*name).clone(),
                exercises: (*we).clone(),
                duration_mins: elapsed / 60,
            };
            let mut workouts = storage::load_workouts();
            workouts.push(workout);
            storage::save_workouts(&workouts);
            saved.set(true);
            nav.replace(&Route::History);
        })
    };

    // Show exercise picker as a full page when explicitly opened OR when workout is empty
    let show_picker = *show_exercise_picker || (workout_exercises.is_empty() && !*saved);
    if show_picker {
        return html! {
            <div class="pb-20 transition-colors duration-200">
                <div class="px-4 pt-4 pb-2 flex justify-between items-center">
                    <h2 class="text-xl font-bold text-gray-900 dark:text-gray-100">
                        { if workout_exercises.is_empty() { "Start Workout" } else { "Add Exercise" } }
                    </h2>
                    { if !workout_exercises.is_empty() {
                        html! {
                            <button
                                class="text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 font-medium"
                                onclick={let s = show_exercise_picker.clone(); Callback::from(move |_| s.set(false))}
                            >{"Cancel"}</button>
                        }
                    } else { html! {} }}
                </div>
                <ExerciseList
                    exercises={all_exercises.clone()}
                    on_select={on_add_exercise.clone()}
                    on_add={on_add_exercise}
                    show_add_button={true}
                />
            </div>
        };
    }

    let rest_trigger_val = *rest_trigger;

    // Undo pill
    let undo_html = if undo_snapshot.is_some() {
        let we = workout_exercises.clone();
        let snap = undo_snapshot.clone();
        html! {
            <div class="fixed bottom-32 left-1/2 -translate-x-1/2 z-50">
                <button
                    class="bg-gray-900 dark:bg-gray-600 text-white px-4 py-2 rounded-full shadow-lg text-sm font-bold hover:bg-gray-800 dark:hover:bg-gray-500 transition-colors"
                    onclick={Callback::from(move |_| {
                        if let Some(ref snapshot) = *snap {
                            we.set(snapshot.clone());
                        }
                        snap.set(None);
                    })}
                >{"Undo"}</button>
            </div>
        }
    } else {
        html! {}
    };

    html! {
        <div class="px-4 py-4 pb-24 space-y-6 transition-colors duration-200">
            <div class="flex justify-between items-center">
                <div>
                    <input
                        type="text"
                        class="text-2xl font-bold bg-transparent border-b border-gray-200 dark:border-gray-700 focus:border-blue-500 focus:outline-none text-gray-900 dark:text-gray-100 transition-colors"
                        value={(*workout_name).clone()}
                        onchange={let n = workout_name.clone(); Callback::from(move |e: Event| {
                            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                            n.set(input.value());
                        })}
                    />
                </div>
                { if *workout_active {
                    html! { <ElapsedTimer active={true} /> }
                } else { html! {} }}
            </div>

            <WorkoutLog
                workout_exercises={(*workout_exercises).clone()}
                all_exercises={all_exercises.clone()}
                on_update={on_update}
                on_remove_exercise={on_remove}
                previous_workouts={(*previous_workouts).clone()}
                rest_seconds={config.rest_seconds}
                bar_weight={config.bar_weight}
                on_set_completed={on_set_completed}
                on_before_destructive={on_before_destructive}
                unit_system={config.unit_system.clone()}
            />

            <button
                class="w-full py-4 bg-gray-100 dark:bg-gray-800/50 rounded-xl text-blue-600 dark:text-blue-400 font-bold hover:bg-gray-200 dark:hover:bg-gray-800 border border-gray-200 dark:border-gray-700 border-dashed transition-all"
                onclick={let s = show_exercise_picker.clone(); Callback::from(move |_| s.set(true))}
            >{"+ Add Exercise"}</button>

            { if !workout_exercises.is_empty() {
                html! {
                    <button
                        class="w-full py-4 bg-green-600 text-white rounded-xl font-bold text-lg hover:bg-green-700 shadow-lg shadow-green-900/20 transition-all"
                        onclick={on_save}
                    >{"Finish & Save Workout"}</button>
                }
            } else { html! {} }}

            <RestTimer trigger={rest_trigger_val} />
            {undo_html}
        </div>
    }
}
