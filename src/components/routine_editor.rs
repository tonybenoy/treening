use yew::prelude::*;
use crate::models::{Exercise, Routine};
use crate::sharing::{self, ShareableData};
use crate::components::share_modal::ShareModal;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub routines: Vec<Routine>,
    pub all_exercises: Vec<Exercise>,
    pub on_save: Callback<Routine>,
    pub on_delete: Callback<String>,
    pub on_start_workout: Callback<Routine>,
}

#[function_component(RoutineEditor)]
pub fn routine_editor(props: &Props) -> Html {
    let editing = use_state(|| None::<Routine>);
    let show_exercise_picker = use_state(|| false);
    let share_target = use_state(|| None::<(ShareableData, String)>);

    let find_exercise = |id: &str| -> String {
        props.all_exercises.iter()
            .find(|e| e.id == id)
            .map(|e| e.name.clone())
            .unwrap_or_else(|| id.to_string())
    };

    let start_new = {
        let editing = editing.clone();
        Callback::from(move |_| {
            editing.set(Some(Routine {
                id: uuid::Uuid::new_v4().to_string(),
                name: String::new(),
                exercise_ids: Vec::new(),
            }));
        })
    };

    let edit_existing = |r: Routine| {
        let editing = editing.clone();
        Callback::from(move |_| {
            editing.set(Some(r.clone()));
        })
    };

    html! {
        <div class="px-4 space-y-4 pb-4">
            { if let Some(routine) = &*editing {
                let routine = routine.clone();
                let editing2 = editing.clone();
                let editing3 = editing.clone();
                let editing4 = editing.clone();
                let on_save = props.on_save.clone();
                let show_picker = show_exercise_picker.clone();

                html! {
                    <div class="bg-gray-100 dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-transparent transition-colors">
                        <h3 class="text-lg font-semibold mb-3 text-gray-900 dark:text-gray-100">
                            { if routine.name.is_empty() { "New Routine" } else { "Edit Routine" } }
                        </h3>
                        <input
                            type="text"
                            placeholder="Routine name (e.g., Push Day)"
                            class="w-full px-3 py-2 bg-white dark:bg-gray-700 border border-gray-300 dark:border-transparent rounded mb-3 text-gray-900 dark:text-gray-100 outline-none focus:ring-1 focus:ring-blue-500"
                            value={routine.name.clone()}
                            onchange={{
                                let editing = editing2.clone();
                                let routine = routine.clone();
                                Callback::from(move |e: Event| {
                                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                    let mut r = routine.clone();
                                    r.name = input.value();
                                    editing.set(Some(r));
                                })
                            }}
                        />
                        <div class="space-y-1 mb-3">
                            { for routine.exercise_ids.iter().enumerate().map(|(i, eid)| {
                                let name = find_exercise(eid);
                                let editing = editing3.clone();
                                let routine = routine.clone();
                                html! {
                                    <div class="flex justify-between items-center bg-white dark:bg-gray-700 border border-gray-200 dark:border-transparent rounded px-3 py-2 transition-colors">
                                        <span class="text-sm text-gray-800 dark:text-gray-100">{name}</span>
                                        <button
                                            class="text-red-600 dark:text-red-400 text-sm hover:text-red-500 dark:hover:text-red-300"
                                            onclick={Callback::from(move |_| {
                                                let mut r = routine.clone();
                                                r.exercise_ids.remove(i);
                                                editing.set(Some(r));
                                            })}
                                        >{"\u{2715}"}</button>
                                    </div>
                                }
                            })}
                        </div>

                        { if *show_exercise_picker {
                            let exercises = props.all_exercises.clone();
                            let show_picker2 = show_picker.clone();
                            let editing5 = editing4.clone();
                            let routine2 = routine.clone();
                            html! {
                                <div class="bg-white dark:bg-gray-700 border border-gray-300 dark:border-transparent rounded-lg p-3 mb-3 max-h-60 overflow-y-auto transition-colors">
                                    { for exercises.iter().map(|ex| {
                                        let eid = ex.id.clone();
                                        let editing = editing5.clone();
                                        let routine = routine2.clone();
                                        let show_picker = show_picker2.clone();
                                        html! {
                                            <button
                                                class="block w-full text-left px-2 py-1.5 hover:bg-gray-100 dark:hover:bg-gray-600 rounded text-sm text-gray-800 dark:text-gray-100 transition-colors"
                                                onclick={Callback::from(move |_| {
                                                    let mut r = routine.clone();
                                                    r.exercise_ids.push(eid.clone());
                                                    editing.set(Some(r));
                                                    show_picker.set(false);
                                                })}
                                            >
                                                {&ex.name}
                                                <span class="text-gray-500 dark:text-gray-400 ml-2 text-xs">{ex.category.to_string()}</span>
                                            </button>
                                        }
                                    })}
                                </div>
                            }
                        } else {
                            html! {
                                <button
                                    class="text-blue-600 dark:text-blue-400 text-sm mb-3 font-medium hover:underline"
                                    onclick={let sp = show_picker.clone(); Callback::from(move |_| sp.set(true))}
                                >{"+ Add Exercise"}</button>
                            }
                        }}

                        <div class="flex gap-2">
                            <button
                                class="flex-1 py-2 bg-blue-600 text-white rounded font-medium hover:bg-blue-700 shadow-sm transition-colors"
                                onclick={{
                                    let routine = routine.clone();
                                    let editing = editing.clone();
                                    Callback::from(move |_| {
                                        if !routine.name.is_empty() {
                                            on_save.emit(routine.clone());
                                            editing.set(None);
                                        }
                                    })
                                }}
                            >{"Save"}</button>
                            <button
                                class="flex-1 py-2 bg-gray-200 dark:bg-gray-700 text-gray-800 dark:text-gray-200 rounded font-medium hover:bg-gray-300 dark:hover:bg-gray-600 border border-gray-300 dark:border-transparent transition-colors"
                                onclick={let e = editing.clone(); Callback::from(move |_| e.set(None))}
                            >{"Cancel"}</button>
                        </div>
                    </div>
                }
            } else {
                html! {
                    <>
                        <button
                            class="w-full py-3 bg-blue-600 rounded-lg font-medium hover:bg-blue-700"
                            onclick={start_new}
                        >{"+ New Routine"}</button>
                        { for props.routines.iter().map(|r| {
                            let r2 = r.clone();
                            let r3 = r.clone();
                            let on_delete = props.on_delete.clone();
                            let on_start = props.on_start_workout.clone();
                            let rid = r.id.clone();
                            html! {
                                <div class="bg-gray-800 rounded-lg p-4">
                                    <div class="flex justify-between items-start mb-2">
                                        <h3 class="font-semibold">{&r.name}</h3>
                                        <span class="text-sm text-gray-400">{r.exercise_ids.len()}{" exercises"}</span>
                                    </div>
                                    <div class="text-sm text-gray-400 mb-3">
                                        { for r.exercise_ids.iter().map(|eid| {
                                            let name = find_exercise(eid);
                                            html! { <span class="mr-2">{name}{","}</span> }
                                        })}
                                    </div>
                                    <div class="flex gap-2">
                                        <button
                                            class="flex-1 py-2 bg-green-700 rounded text-sm font-medium hover:bg-green-600"
                                            onclick={Callback::from(move |_| on_start.emit(r2.clone()))}
                                        >{"Start Workout"}</button>
                                        <button
                                            class="px-3 py-2 bg-gray-700 rounded text-sm hover:bg-gray-600"
                                            onclick={edit_existing(r3.clone())}
                                        >{"Edit"}</button>
                                        <button
                                            class="px-3 py-2 bg-gray-700 rounded text-sm hover:bg-gray-600"
                                            onclick={{
                                                let share_target = share_target.clone();
                                                let r = r.clone();
                                                let all_ex = props.all_exercises.clone();
                                                Callback::from(move |_| {
                                                    let exercises = sharing::collect_routine_exercises(&r, &all_ex);
                                                    let text = sharing::format_routine_text(&r, &exercises);
                                                    let data = ShareableData::Routine { routine: r.clone(), exercises };
                                                    share_target.set(Some((data, text)));
                                                })
                                            }}
                                        >{"Share"}</button>
                                        <button
                                            class="px-3 py-2 bg-red-900 rounded text-sm hover:bg-red-800"
                                            onclick={Callback::from(move |_| on_delete.emit(rid.clone()))}
                                        >{"Delete"}</button>
                                    </div>
                                </div>
                            }
                        })}
                        { if props.routines.is_empty() {
                            html! { <p class="text-gray-500 text-center py-4">{"No routines yet. Create one to quickly start workouts."}</p> }
                        } else { html! {} }}
                    </>
                }
            }}
            { if let Some((ref data, ref text)) = *share_target {
                let share_target = share_target.clone();
                html! {
                    <ShareModal
                        shareable={data.clone()}
                        formatted_text={text.clone()}
                        on_close={Callback::from(move |_| share_target.set(None))}
                    />
                }
            } else { html! {} }}
        </div>
    }
}
