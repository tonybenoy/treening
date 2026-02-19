use crate::models::{Category, Exercise};
use yew::prelude::*;

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

    let filtered: Vec<&Exercise> = props
        .exercises
        .iter()
        .filter(|e| {
            let search_match = if search.is_empty() {
                true
            } else {
                let s = search.to_lowercase();
                e.name.to_lowercase().contains(&s)
                    || e.muscle_groups
                        .iter()
                        .any(|m| m.to_lowercase().contains(&s))
                    || e.equipment.to_string().to_lowercase().contains(&s)
            };
            let cat_match = match &*category_filter {
                Some(cat) => e.category == *cat,
                None => true,
            };
            search_match && cat_match
        })
        .collect();

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
                    class="w-full px-4 py-2 bg-gray-100 dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg text-gray-900 dark:text-gray-100 placeholder-gray-500 focus:outline-none focus:border-blue-500 transition-colors shadow-sm"
                    oninput={on_search}
                    value={(*search).clone()}
                />
            </div>
            <div class="px-4 pb-2 flex gap-2 overflow-x-auto scrollbar-hide">
                {
                    {
                        let cf = category_filter.clone();
                        html! {
                            <button
                                class={if cf.is_none() {
                                    "px-3 py-1 rounded-full text-sm bg-blue-600 text-white font-medium shadow-sm transition-colors"
                                } else {
                                    "px-3 py-1 rounded-full text-sm bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300 border border-gray-200 dark:border-transparent hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
                                }}
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
                            class={if active {
                                "px-3 py-1 rounded-full text-sm bg-blue-600 text-white whitespace-nowrap font-medium shadow-sm transition-colors"
                            } else {
                                "px-3 py-1 rounded-full text-sm bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300 border border-gray-200 dark:border-transparent whitespace-nowrap hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
                            }}
                            onclick={Callback::from(move |_| cf.set(Some(cat_clone.clone())))}
                        >{label}</button>
                    }
                })}
            </div>
            <div class="px-4 space-y-2 pb-4">
                { for filtered.iter().enumerate().map(|(i, exercise)| {
                    let ex = (*exercise).clone();
                    let on_select = props.on_select.clone();
                    let on_add = props.on_add.clone();
                    let show_add = props.show_add_button;
                    let ex2 = ex.clone();
                    let ex3 = ex.clone();
                    let delay = format!("animation-delay: {}ms", i.min(10) * 30);
                    html! {
                        <div class="bg-gray-100 dark:bg-gray-800 rounded-lg p-3 flex justify-between items-center border border-gray-200 dark:border-transparent transition-colors shadow-sm list-item-enter" style={delay}>
                            <div class="flex-1 cursor-pointer flex items-center gap-3" onclick={Callback::from(move |_| on_select.emit(ex2.clone()))}>
                                { if let Some(ref img) = ex.image {
                                    html! {
                                        <img
                                            src={img.clone()}
                                            alt={ex.name.clone()}
                                            class="w-10 h-10 rounded bg-white dark:bg-gray-700 border border-gray-200 dark:border-transparent p-0.5 flex-shrink-0 transition-colors"
                                        />
                                    }
                                } else {
                                    html! {
                                        <div class="w-10 h-10 rounded bg-white dark:bg-gray-700 border border-gray-200 dark:border-transparent flex items-center justify-center flex-shrink-0 text-gray-400 dark:text-gray-500 text-xs transition-colors">{"?"}</div>
                                    }
                                }}
                                <div>
                                    <div class="font-medium text-gray-900 dark:text-gray-100">{&ex.name}</div>
                                    <div class="text-sm text-gray-500 dark:text-gray-400">
                                        {ex.category.to_string()}{" · "}{ex.equipment.to_string()}
                                    </div>
                                </div>
                            </div>
                            { if show_add {
                                html! {
                                    <button
                                        class="ml-2 px-3 py-1 bg-blue-600 text-white rounded text-sm font-bold hover:bg-blue-700 shadow-sm transition-colors"
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
                    html! { <p class="text-gray-500 dark:text-gray-400 text-center py-12 bg-gray-50 dark:bg-gray-800/20 rounded-2xl border border-dashed border-gray-200 dark:border-gray-700 transition-colors">{"No exercises found"}</p> }
                } else {
                    html! {}
                }}
            </div>
        </div>
    }
}
