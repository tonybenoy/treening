use yew::prelude::*;
use crate::models::{Category, Equipment, Exercise};

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
    let muscles = use_state(String::new);
    let description = use_state(String::new);

    let on_save = {
        let name = name.clone();
        let category = category.clone();
        let equipment = equipment.clone();
        let muscles = muscles.clone();
        let description = description.clone();
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
                    image: None,
                });
            }
        })
    };

    html! {
        <div class="bg-gray-800 rounded-lg p-4">
            <h3 class="text-lg font-semibold mb-4">{"Add Custom Exercise"}</h3>
            <div class="space-y-3">
                <div>
                    <label class="block text-sm text-gray-400 mb-1">{"Name"}</label>
                    <input
                        type="text"
                        class="w-full px-3 py-2 bg-gray-700 rounded text-gray-100"
                        value={(*name).clone()}
                        oninput={let n = name.clone(); Callback::from(move |e: InputEvent| {
                            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                            n.set(input.value());
                        })}
                    />
                </div>
                <div>
                    <label class="block text-sm text-gray-400 mb-1">{"Category"}</label>
                    <select
                        class="w-full px-3 py-2 bg-gray-700 rounded text-gray-100"
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
                    <label class="block text-sm text-gray-400 mb-1">{"Equipment"}</label>
                    <select
                        class="w-full px-3 py-2 bg-gray-700 rounded text-gray-100"
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
                <div>
                    <label class="block text-sm text-gray-400 mb-1">{"Muscle Groups (comma-separated)"}</label>
                    <input
                        type="text"
                        placeholder="e.g. Chest, Triceps"
                        class="w-full px-3 py-2 bg-gray-700 rounded text-gray-100"
                        value={(*muscles).clone()}
                        oninput={let m = muscles.clone(); Callback::from(move |e: InputEvent| {
                            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                            m.set(input.value());
                        })}
                    />
                </div>
                <div>
                    <label class="block text-sm text-gray-400 mb-1">{"Description"}</label>
                    <input
                        type="text"
                        class="w-full px-3 py-2 bg-gray-700 rounded text-gray-100"
                        value={(*description).clone()}
                        oninput={let d = description.clone(); Callback::from(move |e: InputEvent| {
                            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                            d.set(input.value());
                        })}
                    />
                </div>
                <div class="flex gap-2 pt-2">
                    <button
                        class="flex-1 py-2 bg-blue-600 rounded font-medium hover:bg-blue-700"
                        onclick={on_save}
                    >{"Save"}</button>
                    <button
                        class="flex-1 py-2 bg-gray-700 rounded font-medium hover:bg-gray-600"
                        onclick={let cb = props.on_cancel.clone(); Callback::from(move |_| cb.emit(()))}
                    >{"Cancel"}</button>
                </div>
            </div>
        </div>
    }
}
