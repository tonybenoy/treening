use yew::prelude::*;
use crate::models::{Category, Exercise};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub exercises: Vec<Exercise>,
    pub on_select: Callback<Exercise>,
    #[prop_or_default]
    pub on_add: Option<Callback<Exercise>>,
    #[prop_or_default]
    pub show_add_button: bool,
}

#[function_component(ExerciseList)]
pub fn exercise_list(props: &Props) -> Html {
    let search = use_state(String::new);
    let category_filter = use_state(|| None::<Category>);

    let filtered: Vec<&Exercise> = props.exercises.iter().filter(|e| {
        let search_match = if search.is_empty() {
            true
        } else {
            let s = search.to_lowercase();
            e.name.to_lowercase().contains(&s)
                || e.muscle_groups.iter().any(|m| m.to_lowercase().contains(&s))
                || e.equipment.to_string().to_lowercase().contains(&s)
        };
        let cat_match = match &*category_filter {
            Some(cat) => e.category == *cat,
            None => true,
        };
        search_match && cat_match
    }).collect();

    let on_search = {
        let search = search.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            search.set(input.value());
        })
    };

    let categories = Category::all();

    html! {
        <div>
            <div class="px-4 pt-2 pb-2">
                <input
                    type="text"
                    placeholder="Search exercises..."
                    class="w-full px-4 py-2 bg-gray-800 border border-gray-600 rounded-lg text-gray-100 placeholder-gray-500 focus:outline-none focus:border-blue-500"
                    oninput={on_search}
                    value={(*search).clone()}
                />
            </div>
            <div class="px-4 pb-2 flex gap-2 overflow-x-auto">
                {
                    {
                        let cf = category_filter.clone();
                        html! {
                            <button
                                class={if cf.is_none() { "px-3 py-1 rounded-full text-sm bg-blue-600 text-white" } else { "px-3 py-1 rounded-full text-sm bg-gray-700 text-gray-300" }}
                                onclick={let cf = cf.clone(); Callback::from(move |_| cf.set(None))}
                            >{"All"}</button>
                        }
                    }
                }
                { for categories.iter().map(|cat| {
                    let cf = category_filter.clone();
                    let cat_clone = cat.clone();
                    let active = *cf == Some(cat.clone());
                    let label = cat.to_string();
                    html! {
                        <button
                            class={if active { "px-3 py-1 rounded-full text-sm bg-blue-600 text-white whitespace-nowrap" } else { "px-3 py-1 rounded-full text-sm bg-gray-700 text-gray-300 whitespace-nowrap" }}
                            onclick={Callback::from(move |_| cf.set(Some(cat_clone.clone())))}
                        >{label}</button>
                    }
                })}
            </div>
            <div class="px-4 space-y-2 pb-4">
                { for filtered.iter().map(|exercise| {
                    let ex = (*exercise).clone();
                    let on_select = props.on_select.clone();
                    let on_add = props.on_add.clone();
                    let show_add = props.show_add_button;
                    let ex2 = ex.clone();
                    let ex3 = ex.clone();
                    html! {
                        <div class="bg-gray-800 rounded-lg p-3 flex justify-between items-center">
                            <div class="flex-1 cursor-pointer flex items-center gap-3" onclick={Callback::from(move |_| on_select.emit(ex2.clone()))}>
                                { if let Some(ref img) = ex.image {
                                    html! {
                                        <img
                                            src={img.clone()}
                                            alt={ex.name.clone()}
                                            class="w-10 h-10 rounded bg-gray-700 p-0.5 flex-shrink-0"
                                        />
                                    }
                                } else {
                                    html! {
                                        <div class="w-10 h-10 rounded bg-gray-700 flex items-center justify-center flex-shrink-0 text-gray-500 text-xs">{"?"}</div>
                                    }
                                }}
                                <div>
                                    <div class="font-medium">{&ex.name}</div>
                                    <div class="text-sm text-gray-400">
                                        {ex.category.to_string()}{" · "}{ex.equipment.to_string()}
                                    </div>
                                </div>
                            </div>
                            { if show_add {
                                html! {
                                    <button
                                        class="ml-2 px-3 py-1 bg-blue-600 rounded text-sm hover:bg-blue-700"
                                        onclick={
                                            let on_add = on_add.clone();
                                            Callback::from(move |_| {
                                                if let Some(ref cb) = on_add {
                                                    cb.emit(ex3.clone());
                                                }
                                            })
                                        }
                                    >{"+ Add"}</button>
                                }
                            } else {
                                html! {}
                            }}
                        </div>
                    }
                })}
                { if filtered.is_empty() {
                    html! { <p class="text-gray-500 text-center py-8">{"No exercises found"}</p> }
                } else {
                    html! {}
                }}
            </div>
        </div>
    }
}
