use yew::prelude::*;
use crate::models::Exercise;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub exercise: Exercise,
    pub on_back: Callback<()>,
}

#[function_component(ExerciseDetail)]
pub fn exercise_detail(props: &Props) -> Html {
    let ex = &props.exercise;
    let on_back = props.on_back.clone();

    html! {
        <div class="px-4 py-4">
            <button
                class="text-blue-400 mb-4 flex items-center gap-1"
                onclick={Callback::from(move |_| on_back.emit(()))}
            >
                {"\u{2190} Back"}
            </button>
            <h2 class="text-2xl font-bold mb-2">{&ex.name}</h2>
            <div class="flex gap-2 mb-4">
                <span class="px-2 py-1 bg-blue-900 rounded text-sm">{ex.category.to_string()}</span>
                <span class="px-2 py-1 bg-gray-700 rounded text-sm">{ex.equipment.to_string()}</span>
                { if ex.is_custom {
                    html! { <span class="px-2 py-1 bg-amber-900 rounded text-sm">{"Custom"}</span> }
                } else {
                    html! {}
                }}
            </div>
            { if let Some(ref img) = ex.image {
                html! {
                    <div class="mb-4 flex justify-center">
                        <img
                            src={img.clone()}
                            alt={ex.name.clone()}
                            class="w-full max-w-sm rounded-lg bg-gray-800 p-2"
                        />
                    </div>
                }
            } else {
                html! {}
            }}
            <div class="mb-4">
                <h3 class="text-lg font-semibold mb-1">{"Muscles Worked"}</h3>
                <div class="flex flex-wrap gap-2">
                    { for ex.muscle_groups.iter().map(|m| html! {
                        <span class="px-2 py-1 bg-gray-800 rounded text-sm text-gray-300">{m}</span>
                    })}
                </div>
            </div>
            <div>
                <h3 class="text-lg font-semibold mb-1">{"How to Perform"}</h3>
                <p class="text-gray-300 leading-relaxed">{&ex.description}</p>
            </div>
        </div>
    }
}
