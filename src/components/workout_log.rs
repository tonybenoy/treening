use yew::prelude::*;
use crate::models::{Exercise, WorkoutExercise, WorkoutSet};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub workout_exercises: Vec<WorkoutExercise>,
    pub all_exercises: Vec<Exercise>,
    pub on_update: Callback<Vec<WorkoutExercise>>,
    pub on_remove_exercise: Callback<usize>,
}

#[function_component(WorkoutLog)]
pub fn workout_log(props: &Props) -> Html {
    let find_exercise = |id: &str| -> String {
        props.all_exercises.iter()
            .find(|e| e.id == id)
            .map(|e| e.name.clone())
            .unwrap_or_else(|| id.to_string())
    };

    html! {
        <div class="space-y-4">
            { for props.workout_exercises.iter().enumerate().map(|(ex_idx, we)| {
                let name = find_exercise(&we.exercise_id);
                let on_update = props.on_update.clone();
                let on_remove = props.on_remove_exercise.clone();
                let exercises = props.workout_exercises.clone();

                html! {
                    <div class="bg-gray-800 rounded-lg p-4">
                        <div class="flex justify-between items-center mb-3">
                            <h3 class="font-semibold text-lg">{name}</h3>
                            <button
                                class="text-red-400 text-sm hover:text-red-300"
                                onclick={let on_remove = on_remove.clone(); Callback::from(move |_| on_remove.emit(ex_idx))}
                            >{"Remove"}</button>
                        </div>
                        <div class="space-y-2">
                            <div class="grid grid-cols-12 gap-2 text-sm text-gray-400 px-1">
                                <div class="col-span-1">{"#"}</div>
                                <div class="col-span-4">{"Weight (kg)"}</div>
                                <div class="col-span-3">{"Reps"}</div>
                                <div class="col-span-2">{"Done"}</div>
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

                                html! {
                                    <div class={classes!(
                                        "grid", "grid-cols-12", "gap-2", "items-center",
                                        if completed { "opacity-60" } else { "" }
                                    )}>
                                        <div class="col-span-1 text-sm text-gray-400">{set_idx + 1}</div>
                                        <div class="col-span-4">
                                            <input
                                                type="number"
                                                step="0.5"
                                                class="w-full px-2 py-1 bg-gray-700 rounded text-sm text-center"
                                                value={set.weight.to_string()}
                                                onchange={Callback::from(move |e: Event| {
                                                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                    let val: f64 = input.value().parse().unwrap_or(0.0);
                                                    let mut exs = exercises2.clone();
                                                    if let Some(we) = exs.get_mut(ex_idx) {
                                                        if let Some(s) = we.sets.get_mut(set_idx) {
                                                            s.weight = val;
                                                        }
                                                    }
                                                    on_update2.emit(exs);
                                                })}
                                            />
                                        </div>
                                        <div class="col-span-3">
                                            <input
                                                type="number"
                                                class="w-full px-2 py-1 bg-gray-700 rounded text-sm text-center"
                                                value={set.reps.to_string()}
                                                onchange={Callback::from(move |e: Event| {
                                                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                    let val: u32 = input.value().parse().unwrap_or(0);
                                                    let mut exs = exercises3.clone();
                                                    if let Some(we) = exs.get_mut(ex_idx) {
                                                        if let Some(s) = we.sets.get_mut(set_idx) {
                                                            s.reps = val;
                                                        }
                                                    }
                                                    on_update3.emit(exs);
                                                })}
                                            />
                                        </div>
                                        <div class="col-span-2 flex justify-center">
                                            <input
                                                type="checkbox"
                                                checked={completed}
                                                class="w-5 h-5 accent-green-500"
                                                onchange={Callback::from(move |_| {
                                                    let mut exs = exercises4.clone();
                                                    if let Some(we) = exs.get_mut(ex_idx) {
                                                        if let Some(s) = we.sets.get_mut(set_idx) {
                                                            s.completed = !s.completed;
                                                        }
                                                    }
                                                    on_update4.emit(exs);
                                                })}
                                            />
                                        </div>
                                        <div class="col-span-2 flex justify-end">
                                            <button
                                                class="text-red-400 text-xs hover:text-red-300"
                                                onclick={Callback::from(move |_| {
                                                    let mut exs = exercises5.clone();
                                                    if let Some(we) = exs.get_mut(ex_idx) {
                                                        we.sets.remove(set_idx);
                                                    }
                                                    on_update5.emit(exs);
                                                })}
                                            >{"\u{2715}"}</button>
                                        </div>
                                    </div>
                                }
                            })}
                        </div>
                        <button
                            class="mt-2 text-sm text-blue-400 hover:text-blue-300"
                            onclick={{
                                let exercises = exercises.clone();
                                let on_update = on_update.clone();
                                Callback::from(move |_| {
                                    let mut exs = exercises.clone();
                                    if let Some(we) = exs.get_mut(ex_idx) {
                                        let last_set = we.sets.last().cloned().unwrap_or(WorkoutSet {
                                            weight: 0.0, reps: 10, completed: false,
                                        });
                                        we.sets.push(WorkoutSet {
                                            weight: last_set.weight,
                                            reps: last_set.reps,
                                            completed: false,
                                        });
                                    }
                                    on_update.emit(exs);
                                })
                            }}
                        >{"+ Add Set"}</button>
                    </div>
                }
            })}
            { if props.workout_exercises.is_empty() {
                html! {
                    <p class="text-gray-500 text-center py-8">{"No exercises added yet. Tap \"+ Add Exercise\" to start."}</p>
                }
            } else {
                html! {}
            }}
        </div>
    }
}
