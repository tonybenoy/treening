use yew::prelude::*;
use crate::models::{Category, Equipment, Exercise, ExerciseTrackingType};
use gloo::file::callbacks::{self, FileReader};
use web_sys::HtmlInputElement;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub on_save: Callback<Exercise>,
    pub on_cancel: Callback<()>,
}

#[function_component(CustomExerciseForm)]
pub fn custom_exercise_form(props: &Props) -> Html {
    let name = use_state(String::new);
    let category = use_state(|| Category::Chest);
    let equipment = use_state(|| Equipment::Barbell);
    let tracking_type = use_state(|| ExerciseTrackingType::Strength);
    let muscles = use_state(String::new);
    let description = use_state(String::new);
    let image = use_state(|| None::<String>);
    let reader = use_state(|| None::<FileReader>);

    let on_file_select = {
        let image = image.clone();
        let reader = reader.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            if let Some(file_list) = input.files() {
                if let Some(file) = file_list.get(0) {
                    let gloo_file = gloo::file::File::from(file);
                    let image = image.clone();
                    let fr = callbacks::read_as_data_url(&gloo_file, move |result| {
                        if let Ok(data_url) = result {
                            image.set(Some(data_url));
                        }
                    });
                    reader.set(Some(fr));
                }
            }
        })
    };

    let on_save = {
        let name = name.clone();
        let category = category.clone();
        let equipment = equipment.clone();
        let tracking_type = tracking_type.clone();
        let muscles = muscles.clone();
        let description = description.clone();
        let image = image.clone();
        let cb = props.on_save.clone();
        Callback::from(move |_| {
            if !name.is_empty() {
                cb.emit(Exercise {
                    id: format!("custom-{}", uuid::Uuid::new_v4()),
                    name: (*name).clone(),
                    category: (*category).clone(),
                    equipment: (*equipment).clone(),
                    muscle_groups: muscles.split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect(),
                    description: (*description).clone(),
                    is_custom: true,
                    image: (*image).clone(),
                    tracking_type: (*tracking_type).clone(),
                });
            }
        })
    };

    html! {
        <div class="bg-gray-100 dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-transparent transition-colors shadow-sm">
            <h3 class="text-lg font-bold mb-4 text-gray-900 dark:text-gray-100">{"Add Custom Exercise"}</h3>
            <div class="space-y-4">
                <div>
                    <label class="block text-xs font-bold uppercase tracking-wider text-gray-500 dark:text-gray-500 mb-1">{"Name"}</label>
                    <input
                        type="text"
                        class="w-full px-3 py-2 bg-white dark:bg-gray-700 border border-gray-300 dark:border-transparent rounded text-gray-900 dark:text-gray-100 outline-none focus:ring-1 focus:ring-blue-500 transition-colors"
                        value={(*name).clone()}
                        oninput={let n = name.clone(); Callback::from(move |e: InputEvent| {
                            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                            n.set(input.value());
                        })}
                    />
                </div>
                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label class="block text-xs font-bold uppercase tracking-wider text-gray-500 dark:text-gray-500 mb-1">{"Category"}</label>
                        <select
                            class="w-full px-3 py-2 bg-white dark:bg-gray-700 border border-gray-300 dark:border-transparent rounded text-gray-900 dark:text-gray-100 outline-none focus:ring-1 focus:ring-blue-500 transition-colors"
                            onchange={let c = category.clone(); Callback::from(move |e: Event| {
                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                let val = match input.value().as_str() {
                                    "Chest" => Category::Chest,
                                    "Back" => Category::Back,
                                    "Legs" => Category::Legs,
                                    "Shoulders" => Category::Shoulders,
                                    "Arms" => Category::Arms,
                                    "Core" => Category::Core,
                                    "Cardio" => Category::Cardio,
                                    _ => Category::Chest,
                                };
                                c.set(val);
                            })}
                        >
                            { for Category::all().iter().map(|cat| {
                                html! { <option value={cat.to_string()} selected={*category == *cat}>{cat.to_string()}</option> }
                            })}
                        </select>
                    </div>
                    <div>
                        <label class="block text-xs font-bold uppercase tracking-wider text-gray-500 dark:text-gray-500 mb-1">{"Equipment"}</label>
                        <select
                            class="w-full px-3 py-2 bg-white dark:bg-gray-700 border border-gray-300 dark:border-transparent rounded text-gray-900 dark:text-gray-100 outline-none focus:ring-1 focus:ring-blue-500 transition-colors"
                            onchange={let eq = equipment.clone(); Callback::from(move |e: Event| {
                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                let val = match input.value().as_str() {
                                    "Barbell" => Equipment::Barbell,
                                    "Dumbbell" => Equipment::Dumbbell,
                                    "Machine" => Equipment::Machine,
                                    "Cable" => Equipment::Cable,
                                    "Bodyweight" => Equipment::Bodyweight,
                                    "Kettlebell" => Equipment::Kettlebell,
                                    "Band" => Equipment::Band,
                                    _ => Equipment::Other,
                                };
                                eq.set(val);
                            })}
                        >
                            <option value="Barbell">{"Barbell"}</option>
                            <option value="Dumbbell">{"Dumbbell"}</option>
                            <option value="Machine">{"Machine"}</option>
                            <option value="Cable">{"Cable"}</option>
                            <option value="Bodyweight">{"Bodyweight"}</option>
                            <option value="Kettlebell">{"Kettlebell"}</option>
                            <option value="Band">{"Band"}</option>
                            <option value="Other">{"Other"}</option>
                        </select>
                    </div>
                </div>
                <div>
                    <label class="block text-xs font-bold uppercase tracking-wider text-gray-500 dark:text-gray-500 mb-1">{"Tracking Type"}</label>
                    <select
                        class="w-full px-3 py-2 bg-white dark:bg-gray-700 border border-gray-300 dark:border-transparent rounded text-gray-900 dark:text-gray-100 outline-none focus:ring-1 focus:ring-blue-500 transition-colors"
                        onchange={let tt = tracking_type.clone(); Callback::from(move |e: Event| {
                            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                            let val = match input.value().as_str() {
                                "Strength" => ExerciseTrackingType::Strength,
                                "Cardio" => ExerciseTrackingType::Cardio,
                                "Duration" => ExerciseTrackingType::Duration,
                                "Bodyweight" => ExerciseTrackingType::Bodyweight,
                                _ => ExerciseTrackingType::Strength,
                            };
                            tt.set(val);
                        })}
                    >
                        <option value="Strength" selected={*tracking_type == ExerciseTrackingType::Strength}>{"Strength (Weight + Reps)"}</option>
                        <option value="Cardio" selected={*tracking_type == ExerciseTrackingType::Cardio}>{"Cardio (Distance + Time)"}</option>
                        <option value="Duration" selected={*tracking_type == ExerciseTrackingType::Duration}>{"Duration (Time only)"}</option>
                        <option value="Bodyweight" selected={*tracking_type == ExerciseTrackingType::Bodyweight}>{"Bodyweight (Reps only)"}</option>
                    </select>
                </div>
                <div>
                    <label class="block text-xs font-bold uppercase tracking-wider text-gray-500 dark:text-gray-500 mb-1">{"Muscle Groups (comma-separated)"}</label>
                    <input
                        type="text"
                        placeholder="e.g. Chest, Triceps"
                        class="w-full px-3 py-2 bg-white dark:bg-gray-700 border border-gray-300 dark:border-transparent rounded text-gray-900 dark:text-gray-100 outline-none focus:ring-1 focus:ring-blue-500 transition-colors"
                        value={(*muscles).clone()}
                        oninput={let m = muscles.clone(); Callback::from(move |e: InputEvent| {
                            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                            m.set(input.value());
                        })}
                    />
                </div>
                <div>
                    <label class="block text-xs font-bold uppercase tracking-wider text-gray-500 dark:text-gray-500 mb-1">{"Description"}</label>
                    <textarea
                        rows="3"
                        class="w-full px-3 py-2 bg-white dark:bg-gray-700 border border-gray-300 dark:border-transparent rounded text-gray-900 dark:text-gray-100 outline-none focus:ring-1 focus:ring-blue-500 transition-colors"
                        value={(*description).clone()}
                        oninput={let d = description.clone(); Callback::from(move |e: InputEvent| {
                            let input: web_sys::HtmlTextAreaElement = e.target_unchecked_into();
                            d.set(input.value());
                        })}
                    ></textarea>
                </div>
                <div>
                    <label class="block text-xs font-bold uppercase tracking-wider text-gray-500 dark:text-gray-500 mb-1">{"Image"}</label>
                    <input
                        type="file"
                        accept="image/*"
                        class="w-full text-sm text-gray-500 dark:text-gray-400 file:mr-3 file:py-1.5 file:px-3 file:rounded file:border-0 file:text-sm file:font-medium file:bg-blue-50 file:text-blue-600 dark:file:bg-blue-900/30 dark:file:text-blue-400 file:cursor-pointer"
                        onchange={on_file_select}
                    />
                    { if let Some(ref data_url) = *image {
                        let image_clear = image.clone();
                        html! {
                            <div class="mt-2 relative inline-block">
                                <img src={data_url.clone()} class="w-24 h-24 object-cover rounded border border-gray-300 dark:border-gray-600" />
                                <button
                                    class="absolute -top-2 -right-2 w-5 h-5 bg-red-500 text-white rounded-full text-xs flex items-center justify-center hover:bg-red-600 transition-colors"
                                    onclick={Callback::from(move |_| image_clear.set(None))}
                                >{"\u{2715}"}</button>
                            </div>
                        }
                    } else {
                        html! {}
                    }}
                </div>
                <div class="flex gap-2 pt-2">
                    <button
                        class="flex-1 py-2.5 bg-blue-600 text-white rounded-lg font-bold shadow-sm hover:bg-blue-700 transition-colors"
                        onclick={on_save}
                    >{"Save Exercise"}</button>
                    <button
                        class="flex-1 py-2.5 bg-white dark:bg-gray-700 text-gray-700 dark:text-gray-300 border border-gray-300 dark:border-transparent rounded-lg font-bold hover:bg-gray-50 dark:hover:bg-gray-600 transition-colors"
                        onclick={let cb = props.on_cancel.clone(); Callback::from(move |_| cb.emit(()))}
                    >{"Cancel"}</button>
                </div>
            </div>
        </div>
    }
}
