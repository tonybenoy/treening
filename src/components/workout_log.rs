use yew::prelude::*;
use crate::models::{Exercise, WorkoutExercise, WorkoutSet, ExerciseTrackingType};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub workout_exercises: Vec<WorkoutExercise>,
    pub all_exercises: Vec<Exercise>,
    pub on_update: Callback<Vec<WorkoutExercise>>,
    pub on_remove_exercise: Callback<usize>,
}

#[function_component(WorkoutLog)]
pub fn workout_log(props: &Props) -> Html {
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

                html! {
                    <div class="bg-gray-100 dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-transparent transition-colors shadow-sm">
                        <div class="flex justify-between items-center mb-3">
                            <h3 class="font-semibold text-lg text-gray-900 dark:text-gray-100">{name}</h3>
                            <button
                                class="text-red-600 dark:text-red-400 text-sm hover:text-red-500 dark:hover:text-red-300 transition-colors"
                                onclick={let on_remove = on_remove.clone(); Callback::from(move |_| on_remove.emit(ex_idx))}
                            >{"Remove"}</button>
                        </div>
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

                                html! {
                                    <div class={classes!(
                                        "grid", "grid-cols-12", "gap-2", "items-center", "transition-opacity",
                                        if completed { "opacity-50" } else { "" }
                                    )}>
                                        <div class="col-span-1 text-sm font-medium text-gray-400 dark:text-gray-500">{set_idx + 1}</div>
                                        
                                        { match tt {
                                            ExerciseTrackingType::Strength => html! {
                                                <>
                                                    <div class="col-span-4">
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

                                        <div class="col-span-2 flex justify-center">
                                            <input
                                                type="checkbox"
                                                checked={completed}
                                                class="w-5 h-5 accent-blue-600 cursor-pointer"
                                                onchange={Callback::from(move |_| {
                                                    let mut exs = exercises4.clone();
                                                    if let Some(we) = exs.get_mut(ex_idx) { if let Some(s) = we.sets.get_mut(set_idx) { s.completed = !s.completed; } }
                                                    on_update4.emit(exs);
                                                })}
                                            />
                                        </div>
                                        <div class="col-span-2 flex justify-end">
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
                                            weight: 0.0, reps: 10, completed: false, distance: None, duration_secs: None
                                        });
                                        we.sets.push(WorkoutSet {
                                            weight: last_set.weight,
                                            reps: last_set.reps,
                                            distance: last_set.distance,
                                            duration_secs: last_set.duration_secs,
                                            completed: false,
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
