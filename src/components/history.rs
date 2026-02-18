use yew::prelude::*;
use crate::models::{Exercise, Workout, WorkoutSet};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub workouts: Vec<Workout>,
    pub all_exercises: Vec<Exercise>,
    pub on_delete: Callback<String>,
    pub on_update: Callback<Workout>,
}

#[function_component(HistoryList)]
pub fn history_list(props: &Props) -> Html {
    let expanded = use_state(|| None::<String>);
    let editing = use_state(|| None::<Workout>);

    let find_exercise = |id: &str| -> String {
        props.all_exercises.iter()
            .find(|e| e.id == id)
            .map(|e| e.name.clone())
            .unwrap_or_else(|| id.to_string())
    };

    let mut workouts = props.workouts.clone();
    workouts.sort_by(|a, b| b.date.cmp(&a.date));

    html! {
        <div class="space-y-3 px-4 pb-4">
            { for workouts.iter().map(|w| {
                let is_expanded = *expanded == Some(w.id.clone());
                let is_editing = editing.as_ref().map(|e| e.id == w.id).unwrap_or(false);
                let wid = w.id.clone();
                let expanded_clone = expanded.clone();
                let on_delete = props.on_delete.clone();
                let on_update = props.on_update.clone();
                let wid2 = w.id.clone();
                let total_sets: usize = w.exercises.iter().map(|e| e.sets.len()).sum();

                // Use the editing version if we're editing this workout
                let display_workout = if is_editing {
                    editing.as_ref().unwrap().clone()
                } else {
                    w.clone()
                };

                html! {
                    <div class="bg-gray-100 dark:bg-gray-800 rounded-lg overflow-hidden border border-gray-200 dark:border-transparent transition-colors shadow-sm">
                        <div
                            class="p-4 cursor-pointer hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors"
                            onclick={Callback::from(move |_| {
                                if is_expanded { expanded_clone.set(None) } else { expanded_clone.set(Some(wid.clone())) }
                            })}
                        >
                            <div class="flex justify-between items-start">
                                <div>
                                    <div class="font-bold text-gray-900 dark:text-gray-100">{&w.name}</div>
                                    <div class="text-sm text-gray-500 dark:text-gray-400 font-mono">{&w.date}</div>
                                </div>
                                <div class="text-right text-sm text-gray-500 dark:text-gray-400 font-medium">
                                    <div>{w.exercises.len()}{" exercises"}</div>
                                    <div>{total_sets}{" sets"}</div>
                                    { if w.duration_mins > 0 {
                                        html! { <div>{w.duration_mins}{"min"}</div> }
                                    } else { html! {} }}
                                </div>
                            </div>
                        </div>
                        { if *expanded == Some(w.id.clone()) {
                            if is_editing {
                                // Edit mode
                                let edit_workout = display_workout.clone();
                                let editing_state = editing.clone();

                                html! {
                                    <div class="px-4 pb-4 border-t border-gray-200 dark:border-gray-700 pt-3 space-y-4">
                                        // Workout name
                                        <div>
                                            <label class="block text-[10px] uppercase font-bold text-gray-500 dark:text-gray-500 mb-1 tracking-wider">{"Workout Name"}</label>
                                            <input
                                                type="text"
                                                class="w-full px-3 py-2 bg-white dark:bg-gray-700 border border-gray-300 dark:border-transparent rounded text-gray-900 dark:text-gray-100 outline-none focus:ring-1 focus:ring-blue-500 transition-colors"
                                                value={edit_workout.name.clone()}
                                                onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}
                                                onchange={{
                                                    let editing = editing_state.clone();
                                                    let w = edit_workout.clone();
                                                    Callback::from(move |e: Event| {
                                                        let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                        let mut updated = w.clone();
                                                        updated.name = input.value();
                                                        editing.set(Some(updated));
                                                    })
                                                }}
                                            />
                                        </div>

                                        // Exercises
                                        { for edit_workout.exercises.iter().enumerate().map(|(ex_idx, we)| {
                                            let name = find_exercise(&we.exercise_id);
                                            let editing = editing_state.clone();
                                            let workout = edit_workout.clone();

                                            html! {
                                                <div class="bg-white dark:bg-gray-700/50 border border-gray-200 dark:border-gray-700 rounded-lg p-3 transition-colors">
                                                    <div class="flex justify-between items-center mb-2">
                                                        <span class="font-bold text-sm text-gray-800 dark:text-gray-200">{name}</span>
                                                        <button
                                                            class="text-red-600 dark:text-red-400 text-xs hover:text-red-500 dark:hover:text-red-300 font-bold"
                                                            onclick={{
                                                                let editing = editing.clone();
                                                                let w = workout.clone();
                                                                Callback::from(move |e: MouseEvent| {
                                                                    e.stop_propagation();
                                                                    let mut updated = w.clone();
                                                                    updated.exercises.remove(ex_idx);
                                                                    editing.set(Some(updated));
                                                                })
                                                            }}
                                                        >{"Remove"}</button>
                                                    </div>
                                                    <div class="space-y-1">
                                                        <div class="grid grid-cols-12 gap-2 text-[10px] uppercase font-bold text-gray-500 dark:text-gray-500 px-1">
                                                            <div class="col-span-1">{"#"}</div>
                                                            <div class="col-span-4">{"Weight"}</div>
                                                            <div class="col-span-3">{"Reps"}</div>
                                                            <div class="col-span-2 text-center">{"Done"}</div>
                                                            <div class="col-span-2"></div>
                                                        </div>
                                                        { for we.sets.iter().enumerate().map(|(set_idx, set)| {
                                                            let editing = editing.clone();
                                                            let workout = workout.clone();
                                                            let editing2 = editing.clone();
                                                            let workout2 = workout.clone();
                                                            let editing3 = editing.clone();
                                                            let workout3 = workout.clone();
                                                            let editing4 = editing.clone();
                                                            let workout4 = workout.clone();
                                                            let completed = set.completed;

                                                            html! {
                                                                <div class={classes!(
                                                                    "grid", "grid-cols-12", "gap-2", "items-center", "transition-opacity",
                                                                    if completed { "opacity-50" } else { "" }
                                                                )}>
                                                                    <div class="col-span-1 text-xs font-bold text-gray-400 dark:text-gray-500">{set_idx + 1}</div>
                                                                    <div class="col-span-4">
                                                                        <input
                                                                            type="number"
                                                                            step="0.5"
                                                                            class="w-full px-2 py-1 bg-white dark:bg-gray-600 border border-gray-300 dark:border-transparent rounded text-xs text-center text-gray-900 dark:text-white outline-none focus:ring-1 focus:ring-blue-500 transition-colors"
                                                                            value={set.weight.to_string()}
                                                                            onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}
                                                                            onchange={Callback::from(move |e: Event| {
                                                                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                                                let val: f64 = input.value().parse().unwrap_or(0.0);
                                                                                let mut w = workout.clone();
                                                                                if let Some(we) = w.exercises.get_mut(ex_idx) {
                                                                                    if let Some(s) = we.sets.get_mut(set_idx) {
                                                                                        s.weight = val;
                                                                                    }
                                                                                }
                                                                                editing.set(Some(w));
                                                                            })}
                                                                        />
                                                                    </div>
                                                                    <div class="col-span-3">
                                                                        <input
                                                                            type="number"
                                                                            class="w-full px-2 py-1 bg-white dark:bg-gray-600 border border-gray-300 dark:border-transparent rounded text-xs text-center text-gray-900 dark:text-white outline-none focus:ring-1 focus:ring-blue-500 transition-colors"
                                                                            value={set.reps.to_string()}
                                                                            onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}
                                                                            onchange={Callback::from(move |e: Event| {
                                                                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                                                let val: u32 = input.value().parse().unwrap_or(0);
                                                                                let mut w = workout2.clone();
                                                                                if let Some(we) = w.exercises.get_mut(ex_idx) {
                                                                                    if let Some(s) = we.sets.get_mut(set_idx) {
                                                                                        s.reps = val;
                                                                                    }
                                                                                }
                                                                                editing2.set(Some(w));
                                                                            })}
                                                                        />
                                                                    </div>
                                                                    <div class="col-span-2 flex justify-center">
                                                                        <input
                                                                            type="checkbox"
                                                                            checked={completed}
                                                                            class="w-4 h-4 accent-blue-600 cursor-pointer"
                                                                            onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}
                                                                            onchange={Callback::from(move |_| {
                                                                                let mut w = workout3.clone();
                                                                                if let Some(we) = w.exercises.get_mut(ex_idx) {
                                                                                    if let Some(s) = we.sets.get_mut(set_idx) {
                                                                                        s.completed = !s.completed;
                                                                                    }
                                                                                }
                                                                                editing3.set(Some(w));
                                                                            })}
                                                                        />
                                                                    </div>
                                                                    <div class="col-span-2 flex justify-end">
                                                                        <button
                                                                            class="text-red-600 dark:text-red-400 text-xs hover:text-red-500 dark:hover:text-red-300 p-1 transition-colors"
                                                                            onclick={{
                                                                                Callback::from(move |e: MouseEvent| {
                                                                                    e.stop_propagation();
                                                                                    let mut w = workout4.clone();
                                                                                    if let Some(we) = w.exercises.get_mut(ex_idx) {
                                                                                        we.sets.remove(set_idx);
                                                                                    }
                                                                                    editing4.set(Some(w));
                                                                                })
                                                                            }}
                                                                        >{"\u{2715}"}</button>
                                                                    </div>
                                                                </div>
                                                            }
                                                        })}
                                                    </div>
                                                    <button
                                                        class="mt-2 text-xs font-bold text-blue-600 dark:text-blue-400 hover:underline transition-colors"
                                                        onclick={{
                                                            let editing = editing.clone();
                                                            let w = workout.clone();
                                                            Callback::from(move |e: MouseEvent| {
                                                                e.stop_propagation();
                                                                let mut updated = w.clone();
                                                                if let Some(we) = updated.exercises.get_mut(ex_idx) {
                                                                    let last = we.sets.last().cloned().unwrap_or(WorkoutSet {
                                                                        weight: 0.0, reps: 10, completed: false,
                                                                        distance: None, duration_secs: None,
                                                                    });
                                                                    we.sets.push(WorkoutSet {
                                                                        weight: last.weight,
                                                                        reps: last.reps,
                                                                        completed: false,
                                                                        distance: last.distance,
                                                                        duration_secs: last.duration_secs,
                                                                    });
                                                                }
                                                                editing.set(Some(updated));
                                                            })
                                                        }}
                                                    >{"+ Add Set"}</button>
                                                </div>
                                            }
                                        })}

                                        // Save / Cancel buttons
                                        <div class="flex gap-2 pt-1">
                                            <button
                                                class="flex-1 py-2.5 bg-blue-600 text-white rounded-lg text-sm font-bold hover:bg-blue-700 shadow-sm transition-colors"
                                                onclick={{
                                                    let editing = editing_state.clone();
                                                    let w = edit_workout.clone();
                                                    Callback::from(move |e: MouseEvent| {
                                                        e.stop_propagation();
                                                        on_update.emit(w.clone());
                                                        editing.set(None);
                                                    })
                                                }}
                                            >{"Save Changes"}</button>
                                            <button
                                                class="flex-1 py-2.5 bg-white dark:bg-gray-700 text-gray-700 dark:text-gray-200 border border-gray-300 dark:border-transparent rounded-lg text-sm font-bold hover:bg-gray-50 dark:hover:bg-gray-600 transition-colors"
                                                onclick={{
                                                    let editing = editing_state.clone();
                                                    Callback::from(move |e: MouseEvent| {
                                                        e.stop_propagation();
                                                        editing.set(None);
                                                    })
                                                }}
                                            >{"Cancel"}</button>
                                        </div>
                                    </div>
                                }
                            } else {
                                // View mode
                                html! {
                                    <div class="px-4 pb-4 border-t border-gray-200 dark:border-gray-700 pt-3 transition-colors">
                                        { for display_workout.exercises.iter().map(|we| {
                                            let name = find_exercise(&we.exercise_id);
                                            html! {
                                                <div class="mb-4">
                                                    <div class="font-bold text-sm text-gray-800 dark:text-gray-200 mb-1.5">{name}</div>
                                                    <div class="space-y-1">
                                                        { for we.sets.iter().enumerate().map(|(i, s)| {
                                                            html! {
                                                                <div class="text-xs text-gray-600 dark:text-gray-400 ml-2 flex items-center gap-2">
                                                                    <span class="w-10 font-medium">{"Set "}{i+1}{":"}</span>
                                                                    <span class="font-bold text-gray-800 dark:text-gray-200">{s.weight}{"kg x "}{s.reps}</span>
                                                                    { if s.completed { html!{<span class="text-green-600 dark:text-green-400 text-sm font-bold">{" \u{2713}"}</span>} } else { html!{} } }
                                                                </div>
                                                            }
                                                        })}
                                                    </div>
                                                    { if !we.notes.is_empty() {
                                                        html! { <div class="text-xs text-gray-500 dark:text-gray-500 ml-2 mt-1 italic">{&we.notes}</div> }
                                                    } else { html! {} }}
                                                </div>
                                            }
                                        })}
                                        <div class="flex gap-3 mt-4 border-t border-gray-200 dark:border-gray-700 pt-3">
                                            <button
                                                class="text-blue-600 dark:text-blue-400 text-xs font-bold hover:underline transition-colors"
                                                onclick={{
                                                    let editing = editing.clone();
                                                    let w = w.clone();
                                                    Callback::from(move |e: MouseEvent| {
                                                        e.stop_propagation();
                                                        editing.set(Some(w.clone()));
                                                    })
                                                }}
                                            >{"Edit Workout"}</button>
                                            <button
                                                class="text-red-600 dark:text-red-400 text-xs font-bold hover:underline transition-colors"
                                                onclick={Callback::from(move |e: MouseEvent| {
                                                    e.stop_propagation();
                                                    on_delete.emit(wid2.clone());
                                                })}
                                            >{"Delete Workout"}</button>
                                        </div>
                                    </div>
                                }
                            }
                        } else { html! {} }}
                    </div>
                }
            })}
            { if workouts.is_empty() {
                html! { <p class="text-gray-500 dark:text-gray-500 text-center py-12 bg-gray-50 dark:bg-gray-800/20 rounded-2xl border border-dashed border-gray-200 dark:border-gray-800 transition-colors">{"No workouts recorded yet."}</p> }
            } else { html! {} }}
        </div>
    }
}
