use yew::prelude::*;
use std::collections::HashSet;
use crate::models::{Exercise, Workout, WorkoutExercise, WorkoutSet, ExerciseTrackingType};

/// Epley formula: weight * (1 + reps/30)
fn estimate_1rm(weight: f64, reps: u32) -> f64 {
    weight * (1.0 + reps as f64 / 30.0)
}

/// Compute plates per side for a target weight given bar weight.
/// Returns vec of (plate_weight, count) pairs.
fn compute_plates(target: f64, bar: f64) -> Vec<(f64, u32)> {
    let available = [25.0, 20.0, 15.0, 10.0, 5.0, 2.5, 1.25];
    let mut remaining = (target - bar) / 2.0;
    if remaining <= 0.0 {
        return vec![];
    }
    let mut result = Vec::new();
    for &plate in &available {
        let count = (remaining / plate).floor() as u32;
        if count > 0 {
            result.push((plate, count));
            remaining -= plate * count as f64;
        }
    }
    result
}

/// Find the most recent previous workout that contains the given exercise_id.
fn find_previous_exercise<'a>(previous_workouts: &'a [Workout], exercise_id: &str) -> Option<&'a WorkoutExercise> {
    previous_workouts.iter()
        .rev()
        .flat_map(|w| w.exercises.iter())
        .find(|we| we.exercise_id == exercise_id)
}

/// Compute max weight ever lifted for an exercise across all previous workouts.
fn exercise_pr_weight(previous_workouts: &[Workout], exercise_id: &str) -> f64 {
    previous_workouts.iter()
        .flat_map(|w| w.exercises.iter())
        .filter(|we| we.exercise_id == exercise_id)
        .flat_map(|we| we.sets.iter())
        .filter(|s| s.completed && s.weight > 0.0)
        .map(|s| s.weight)
        .fold(0.0_f64, f64::max)
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub workout_exercises: Vec<WorkoutExercise>,
    pub all_exercises: Vec<Exercise>,
    pub on_update: Callback<Vec<WorkoutExercise>>,
    pub on_remove_exercise: Callback<usize>,
    #[prop_or_default]
    pub previous_workouts: Vec<Workout>,
    #[prop_or(90)]
    pub rest_seconds: u32,
    #[prop_or(20.0)]
    pub bar_weight: f64,
    #[prop_or_default]
    pub on_set_completed: Callback<()>,
}

#[function_component(WorkoutLog)]
pub fn workout_log(props: &Props) -> Html {
    let expanded_notes = use_state(HashSet::<(usize, usize)>::new);
    let plate_calc_target = use_state(|| None::<(usize, usize)>);

    let get_exercise = |id: &str| -> Option<&Exercise> {
        props.all_exercises.iter().find(|e| e.id == id)
    };

    html! {
        <div class="space-y-4">
            { for props.workout_exercises.iter().enumerate().map(|(ex_idx, we)| {
                let exercise = get_exercise(&we.exercise_id);
                let name = exercise.map(|e| e.name.clone()).unwrap_or_else(|| we.exercise_id.clone());
                let tracking_type = exercise.map(|e| e.tracking_type.clone()).unwrap_or(ExerciseTrackingType::Strength);

                let on_update = props.on_update.clone();
                let on_remove = props.on_remove_exercise.clone();
                let exercises = props.workout_exercises.clone();
                let on_set_completed = props.on_set_completed.clone();

                // Superset styling
                let is_superset = we.superset_group.is_some();
                let superset_border = if is_superset { "border-l-4 border-l-purple-500" } else { "" };

                // Previous performance
                let prev_exercise = find_previous_exercise(&props.previous_workouts, &we.exercise_id);
                let prev_text = prev_exercise.map(|prev_we| {
                    prev_we.sets.iter().enumerate().map(|(i, s)| {
                        if s.weight > 0.0 {
                            format!("S{}: {}kg x{}", i + 1, s.weight, s.reps)
                        } else if let Some(d) = s.distance {
                            format!("S{}: {}km", i + 1, d)
                        } else if let Some(dur) = s.duration_secs {
                            format!("S{}: {}s", i + 1, dur)
                        } else {
                            format!("S{}: x{}", i + 1, s.reps)
                        }
                    }).collect::<Vec<_>>().join(", ")
                });

                // PR weight for this exercise
                let pr_weight = exercise_pr_weight(&props.previous_workouts, &we.exercise_id);

                html! {
                    <div class={classes!("bg-gray-100", "dark:bg-gray-800", "rounded-lg", "p-4", "border", "border-gray-200", "dark:border-transparent", "transition-colors", "shadow-sm", superset_border)}>
                        <div class="flex justify-between items-center mb-1">
                            <div class="flex items-center gap-2">
                                <h3 class="font-semibold text-lg text-gray-900 dark:text-gray-100">{&name}</h3>
                                { if is_superset {
                                    html! { <span class="text-[10px] font-bold bg-purple-500/20 text-purple-400 px-1.5 py-0.5 rounded uppercase">{"Superset"}</span> }
                                } else { html! {} }}
                            </div>
                            <div class="flex items-center gap-2">
                                // Superset buttons
                                { if ex_idx > 0 && !is_superset {
                                    let exercises_c = exercises.clone();
                                    let on_update_c = on_update.clone();
                                    html! {
                                        <button
                                            class="text-purple-500 text-[10px] font-bold hover:text-purple-400 transition-colors"
                                            onclick={Callback::from(move |_| {
                                                let mut exs = exercises_c.clone();
                                                // Find group of exercise above, or create new group
                                                let above_group = exs[ex_idx - 1].superset_group;
                                                let group = above_group.unwrap_or_else(|| {
                                                    let max_g = exs.iter().filter_map(|e| e.superset_group).max().unwrap_or(0);
                                                    max_g + 1
                                                });
                                                if let Some(above) = exs.get_mut(ex_idx - 1) {
                                                    above.superset_group = Some(group);
                                                }
                                                if let Some(current) = exs.get_mut(ex_idx) {
                                                    current.superset_group = Some(group);
                                                }
                                                on_update_c.emit(exs);
                                            })}
                                        >{"Group"}</button>
                                    }
                                } else if is_superset {
                                    let exercises_c = exercises.clone();
                                    let on_update_c = on_update.clone();
                                    html! {
                                        <button
                                            class="text-purple-500 text-[10px] font-bold hover:text-purple-400 transition-colors"
                                            onclick={Callback::from(move |_| {
                                                let mut exs = exercises_c.clone();
                                                if let Some(current) = exs.get_mut(ex_idx) {
                                                    let old_group = current.superset_group;
                                                    current.superset_group = None;
                                                    // If only one exercise left in group, ungroup it too
                                                    if let Some(g) = old_group {
                                                        let count = exs.iter().filter(|e| e.superset_group == Some(g)).count();
                                                        if count == 1 {
                                                            for e in exs.iter_mut() {
                                                                if e.superset_group == Some(g) {
                                                                    e.superset_group = None;
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                                on_update_c.emit(exs);
                                            })}
                                        >{"Ungroup"}</button>
                                    }
                                } else { html! {} }}
                                <button
                                    class="text-red-600 dark:text-red-400 text-sm hover:text-red-500 dark:hover:text-red-300 transition-colors"
                                    onclick={let on_remove = on_remove.clone(); Callback::from(move |_| on_remove.emit(ex_idx))}
                                >{"Remove"}</button>
                            </div>
                        </div>

                        // Previous performance overlay
                        { if let Some(ref text) = prev_text {
                            html! { <div class="text-[10px] text-gray-400 dark:text-gray-500 mb-2 font-mono">{"Last: "}{text}</div> }
                        } else { html! {} }}

                        <div class="space-y-2">
                            <div class="grid grid-cols-12 gap-2 text-[10px] uppercase font-bold text-gray-500 dark:text-gray-500 px-1 tracking-wider">
                                <div class="col-span-1">{"#"}</div>
                                { match tracking_type {
                                    ExerciseTrackingType::Strength => html! {
                                        <>
                                            <div class="col-span-4">{"Weight (kg)"}</div>
                                            <div class="col-span-3">{"Reps"}</div>
                                        </>
                                    },
                                    ExerciseTrackingType::Cardio => html! {
                                        <>
                                            <div class="col-span-4">{"Dist (km)"}</div>
                                            <div class="col-span-3">{"Time (m)"}</div>
                                        </>
                                    },
                                    ExerciseTrackingType::Duration => html! {
                                        <div class="col-span-7 text-center">{"Duration (secs)"}</div>
                                    },
                                    ExerciseTrackingType::Bodyweight => html! {
                                        <div class="col-span-7 text-center">{"Reps"}</div>
                                    },
                                }}
                                <div class="col-span-2 text-center">{"Done"}</div>
                                <div class="col-span-2"></div>
                            </div>
                            { for we.sets.iter().enumerate().map(|(set_idx, set)| {
                                let exercises2 = exercises.clone();
                                let on_update2 = on_update.clone();
                                let exercises3 = exercises.clone();
                                let on_update3 = on_update.clone();
                                let exercises4 = exercises.clone();
                                let on_update4 = on_update.clone();
                                let exercises5 = exercises.clone();
                                let on_update5 = on_update.clone();
                                let completed = set.completed;
                                let tt = tracking_type.clone();
                                let on_set_completed2 = on_set_completed.clone();

                                // 1RM calculation for completed strength sets
                                let show_1rm = completed && set.weight > 0.0 && set.reps > 1
                                    && matches!(tt, ExerciseTrackingType::Strength);
                                let est_1rm = if show_1rm { estimate_1rm(set.weight, set.reps) } else { 0.0 };

                                // PR highlight
                                let is_pr = completed && set.weight > 0.0 && set.weight > pr_weight
                                    && matches!(tt, ExerciseTrackingType::Strength);

                                // Per-set note state
                                let note_expanded = {
                                    let notes = expanded_notes.clone();
                                    (*notes).contains(&(ex_idx, set_idx))
                                };
                                let note_text = set.note.clone().unwrap_or_default();

                                // Plate calculator state
                                let show_plate_calc = {
                                    let pc = plate_calc_target.clone();
                                    *pc == Some((ex_idx, set_idx))
                                };
                                let bar_weight = props.bar_weight;

                                html! {
                                    <>
                                    <div class={classes!(
                                        "grid", "grid-cols-12", "gap-2", "items-center", "transition-opacity",
                                        if completed { "opacity-50" } else { "" }
                                    )}>
                                        <div class="col-span-1 text-sm font-medium text-gray-400 dark:text-gray-500 flex items-center gap-0.5">
                                            {set_idx + 1}
                                            { if is_pr {
                                                html! { <span class="text-yellow-500 text-[9px] font-bold">{"PR"}</span> }
                                            } else { html! {} }}
                                        </div>

                                        { match tt {
                                            ExerciseTrackingType::Strength => html! {
                                                <>
                                                    <div class="col-span-4 flex gap-1">
                                                        <input
                                                            type="number" step="0.5"
                                                            class="w-full px-2 py-1 bg-white dark:bg-gray-700 border border-gray-300 dark:border-transparent rounded text-sm text-center text-gray-900 dark:text-gray-100 outline-none focus:ring-1 focus:ring-blue-500 transition-colors"
                                                            value={set.weight.to_string()}
                                                            onchange={Callback::from(move |e: Event| {
                                                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                                let val: f64 = input.value().parse().unwrap_or(0.0);
                                                                let mut exs = exercises2.clone();
                                                                if let Some(we) = exs.get_mut(ex_idx) { if let Some(s) = we.sets.get_mut(set_idx) { s.weight = val; } }
                                                                on_update2.emit(exs);
                                                            })}
                                                        />
                                                        <button
                                                            class="text-gray-400 hover:text-blue-400 text-xs flex-shrink-0 transition-colors"
                                                            title="Plate calculator"
                                                            onclick={{
                                                                let pc = plate_calc_target.clone();
                                                                Callback::from(move |_| {
                                                                    if *pc == Some((ex_idx, set_idx)) {
                                                                        pc.set(None);
                                                                    } else {
                                                                        pc.set(Some((ex_idx, set_idx)));
                                                                    }
                                                                })
                                                            }}
                                                        >{"\u{1f3cb}"}</button>
                                                    </div>
                                                    <div class="col-span-3">
                                                        <input
                                                            type="number"
                                                            class="w-full px-2 py-1 bg-white dark:bg-gray-700 border border-gray-300 dark:border-transparent rounded text-sm text-center text-gray-900 dark:text-gray-100 outline-none focus:ring-1 focus:ring-blue-500 transition-colors"
                                                            value={set.reps.to_string()}
                                                            onchange={Callback::from(move |e: Event| {
                                                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                                let val: u32 = input.value().parse().unwrap_or(0);
                                                                let mut exs = exercises3.clone();
                                                                if let Some(we) = exs.get_mut(ex_idx) { if let Some(s) = we.sets.get_mut(set_idx) { s.reps = val; } }
                                                                on_update3.emit(exs);
                                                            })}
                                                        />
                                                    </div>
                                                </>
                                            },
                                            ExerciseTrackingType::Cardio => html! {
                                                <>
                                                    <div class="col-span-4">
                                                        <input
                                                            type="number" step="0.1"
                                                            class="w-full px-2 py-1 bg-white dark:bg-gray-700 border border-gray-300 dark:border-transparent rounded text-sm text-center text-gray-900 dark:text-gray-100 outline-none focus:ring-1 focus:ring-blue-500 transition-colors"
                                                            value={set.distance.unwrap_or(0.0).to_string()}
                                                            onchange={Callback::from(move |e: Event| {
                                                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                                let val: f64 = input.value().parse().unwrap_or(0.0);
                                                                let mut exs = exercises2.clone();
                                                                if let Some(we) = exs.get_mut(ex_idx) { if let Some(s) = we.sets.get_mut(set_idx) { s.distance = Some(val); } }
                                                                on_update2.emit(exs);
                                                            })}
                                                        />
                                                    </div>
                                                    <div class="col-span-3">
                                                        <input
                                                            type="number"
                                                            class="w-full px-2 py-1 bg-white dark:bg-gray-700 border border-gray-300 dark:border-transparent rounded text-sm text-center text-gray-900 dark:text-gray-100 outline-none focus:ring-1 focus:ring-blue-500 transition-colors"
                                                            value={(set.duration_secs.unwrap_or(0) / 60).to_string()}
                                                            onchange={Callback::from(move |e: Event| {
                                                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                                let val: u32 = input.value().parse().unwrap_or(0);
                                                                let mut exs = exercises3.clone();
                                                                if let Some(we) = exs.get_mut(ex_idx) { if let Some(s) = we.sets.get_mut(set_idx) { s.duration_secs = Some(val * 60); } }
                                                                on_update3.emit(exs);
                                                            })}
                                                        />
                                                    </div>
                                                </>
                                            },
                                            ExerciseTrackingType::Duration => html! {
                                                <div class="col-span-7 px-4">
                                                    <input
                                                        type="number"
                                                        class="w-full px-2 py-1 bg-white dark:bg-gray-700 border border-gray-300 dark:border-transparent rounded text-sm text-center text-gray-900 dark:text-gray-100 outline-none focus:ring-1 focus:ring-blue-500 transition-colors"
                                                        value={set.duration_secs.unwrap_or(0).to_string()}
                                                        onchange={Callback::from(move |e: Event| {
                                                            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                            let val: u32 = input.value().parse().unwrap_or(0);
                                                            let mut exs = exercises2.clone();
                                                            if let Some(we) = exs.get_mut(ex_idx) { if let Some(s) = we.sets.get_mut(set_idx) { s.duration_secs = Some(val); } }
                                                            on_update2.emit(exs);
                                                        })}
                                                    />
                                                </div>
                                            },
                                            ExerciseTrackingType::Bodyweight => html! {
                                                <div class="col-span-7 px-4">
                                                    <input
                                                        type="number"
                                                        class="w-full px-2 py-1 bg-white dark:bg-gray-700 border border-gray-300 dark:border-transparent rounded text-sm text-center text-gray-900 dark:text-gray-100 outline-none focus:ring-1 focus:ring-blue-500 transition-colors"
                                                        value={set.reps.to_string()}
                                                        onchange={Callback::from(move |e: Event| {
                                                            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                            let val: u32 = input.value().parse().unwrap_or(0);
                                                            let mut exs = exercises2.clone();
                                                            if let Some(we) = exs.get_mut(ex_idx) { if let Some(s) = we.sets.get_mut(set_idx) { s.reps = val; } }
                                                            on_update2.emit(exs);
                                                        })}
                                                    />
                                                </div>
                                            },
                                        }}

                                        <div class="col-span-2 flex justify-center items-center gap-1">
                                            <input
                                                type="checkbox"
                                                checked={completed}
                                                class="w-5 h-5 accent-blue-600 cursor-pointer"
                                                onchange={Callback::from(move |_| {
                                                    let mut exs = exercises4.clone();
                                                    if let Some(we) = exs.get_mut(ex_idx) {
                                                        if let Some(s) = we.sets.get_mut(set_idx) {
                                                            let was_completed = s.completed;
                                                            s.completed = !s.completed;
                                                            if !was_completed && s.completed {
                                                                on_set_completed2.emit(());
                                                            }
                                                        }
                                                    }
                                                    on_update4.emit(exs);
                                                })}
                                            />
                                        </div>
                                        <div class="col-span-2 flex justify-end gap-1">
                                            // Note toggle button
                                            <button
                                                class={classes!(
                                                    "text-xs", "p-1", "transition-colors",
                                                    if note_text.is_empty() && !note_expanded {
                                                        "text-gray-400 hover:text-gray-300"
                                                    } else {
                                                        "text-blue-400 hover:text-blue-300"
                                                    }
                                                )}
                                                title="Set note"
                                                onclick={{
                                                    let notes = expanded_notes.clone();
                                                    Callback::from(move |_| {
                                                        let mut set = (*notes).clone();
                                                        let key = (ex_idx, set_idx);
                                                        if set.contains(&key) {
                                                            set.remove(&key);
                                                        } else {
                                                            set.insert(key);
                                                        }
                                                        notes.set(set);
                                                    })
                                                }}
                                            >{"\u{1f4dd}"}</button>
                                            <button
                                                class="text-red-600 dark:text-red-400 text-xs hover:text-red-500 dark:hover:text-red-300 p-1 transition-colors"
                                                onclick={Callback::from(move |_| {
                                                    let mut exs = exercises5.clone();
                                                    if let Some(we) = exs.get_mut(ex_idx) { we.sets.remove(set_idx); }
                                                    on_update5.emit(exs);
                                                })}
                                            >{"\u{2715}"}</button>
                                        </div>
                                    </div>

                                    // 1RM estimate
                                    { if show_1rm {
                                        html! { <div class="text-[10px] text-gray-400 dark:text-gray-500 pl-6 -mt-1 mb-1 font-mono">{format!("Est. 1RM: {:.1} kg", est_1rm)}</div> }
                                    } else { html! {} }}

                                    // Plate calculator popup
                                    { if show_plate_calc && set.weight > bar_weight {
                                        let plates = compute_plates(set.weight, bar_weight);
                                        html! {
                                            <div class="ml-6 mb-2 p-2 bg-gray-200 dark:bg-gray-700 rounded text-xs text-gray-700 dark:text-gray-300">
                                                <div class="font-bold mb-1">{format!("Plates per side ({:.1}kg bar):", bar_weight)}</div>
                                                { if plates.is_empty() {
                                                    html! { <span class="text-gray-500">{"Bar only"}</span> }
                                                } else {
                                                    html! {
                                                        <div class="flex flex-wrap gap-1">
                                                            { for plates.iter().map(|(w, c)| {
                                                                html! { <span class="bg-blue-500/20 text-blue-400 px-1.5 py-0.5 rounded font-mono">{format!("{}kg x{}", w, c)}</span> }
                                                            })}
                                                        </div>
                                                    }
                                                }}
                                            </div>
                                        }
                                    } else { html! {} }}

                                    // Expanded per-set note
                                    { if note_expanded {
                                        let exercises_note = exercises.clone();
                                        let on_update_note = on_update.clone();
                                        html! {
                                            <div class="ml-6 mb-2">
                                                <input
                                                    type="text"
                                                    placeholder="Set note..."
                                                    class="w-full px-2 py-1 bg-white dark:bg-gray-700 border border-gray-300 dark:border-transparent rounded text-xs text-gray-900 dark:text-gray-100 outline-none focus:ring-1 focus:ring-blue-500 transition-colors"
                                                    value={note_text}
                                                    onchange={Callback::from(move |e: Event| {
                                                        let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                        let val = input.value();
                                                        let mut exs = exercises_note.clone();
                                                        if let Some(we) = exs.get_mut(ex_idx) {
                                                            if let Some(s) = we.sets.get_mut(set_idx) {
                                                                s.note = if val.is_empty() { None } else { Some(val) };
                                                            }
                                                        }
                                                        on_update_note.emit(exs);
                                                    })}
                                                />
                                            </div>
                                        }
                                    } else if !set.note.as_ref().is_none_or(|n| n.is_empty()) {
                                        html! { <div class="text-[10px] text-gray-400 dark:text-gray-500 pl-6 -mt-1 mb-1 italic">{set.note.as_ref().unwrap()}</div> }
                                    } else { html! {} }}
                                    </>
                                }
                            })}
                        </div>
                        <button
                            class="mt-3 text-sm font-medium text-blue-600 dark:text-blue-400 hover:underline transition-colors"
                            onclick={{
                                let exercises = exercises.clone();
                                let on_update = on_update.clone();
                                Callback::from(move |_| {
                                    let mut exs = exercises.clone();
                                    if let Some(we) = exs.get_mut(ex_idx) {
                                        let last_set = we.sets.last().cloned().unwrap_or(WorkoutSet {
                                            weight: 0.0, reps: 10, completed: false, distance: None, duration_secs: None, note: None,
                                        });
                                        we.sets.push(WorkoutSet {
                                            weight: last_set.weight,
                                            reps: last_set.reps,
                                            distance: last_set.distance,
                                            duration_secs: last_set.duration_secs,
                                            completed: false,
                                            note: None,
                                        });
                                    }
                                    on_update.emit(exs);
                                })
                            }}
                        >{"+ Add Set"}</button>
                        <div class="mt-2">
                            <input
                                type="text"
                                placeholder="Add notes..."
                                class="w-full px-3 py-1.5 bg-white dark:bg-gray-700 border border-gray-300 dark:border-transparent rounded text-sm text-gray-900 dark:text-gray-100 outline-none focus:ring-1 focus:ring-blue-500 transition-colors"
                                value={we.notes.clone()}
                                onchange={{
                                    let exercises = exercises.clone();
                                    let on_update = on_update.clone();
                                    Callback::from(move |e: Event| {
                                        let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                        let mut exs = exercises.clone();
                                        if let Some(we) = exs.get_mut(ex_idx) {
                                            we.notes = input.value();
                                        }
                                        on_update.emit(exs);
                                    })
                                }}
                            />
                        </div>
                    </div>
                }
            })}
            { if props.workout_exercises.is_empty() {
                html! {
                    <p class="text-gray-500 dark:text-gray-500 text-center py-12 bg-gray-50 dark:bg-gray-800/20 rounded-2xl border border-dashed border-gray-200 dark:border-gray-700 transition-colors">{"No exercises added yet. Tap \"+ Add Exercise\" to start."}</p>
                }
            } else {
                html! {}
            }}
        </div>
    }
}
