use crate::components::custom_exercise::CustomExerciseForm;
use crate::components::exercise_detail::ExerciseDetail;
use crate::components::exercise_list::ExerciseList;
use crate::components::share_modal::ShareModal;
use crate::data;
use crate::models::Exercise;
use crate::sharing::{self, ShareableData};
use crate::storage;
use yew::prelude::*;

#[function_component(ExercisesPage)]
pub fn exercises_page() -> Html {
    let custom_exercises = use_state(storage::load_custom_exercises);
    let selected = use_state(|| None::<Exercise>);
    let show_custom_form = use_state(|| false);
    let share_target = use_state(|| None::<(ShareableData, String)>);

    let all_exercises = {
        let mut exs = data::default_exercises();
        exs.extend((*custom_exercises).clone());
        exs
    };

    let on_select = {
        let selected = selected.clone();
        Callback::from(move |ex: Exercise| selected.set(Some(ex)))
    };

    let on_back = {
        let selected = selected.clone();
        Callback::from(move |_| selected.set(None))
    };

    let on_save_custom = {
        let custom = custom_exercises.clone();
        let show = show_custom_form.clone();
        Callback::from(move |ex: Exercise| {
            let mut exs = (*custom).clone();
            exs.push(ex);
            storage::save_custom_exercises(&exs);
            custom.set(exs);
            show.set(false);
        })
    };

    html! {
        <div class="pb-20">
            { if let Some(ex) = &*selected {
                let share_target_c = share_target.clone();
                let on_share = Callback::from(move |ex: Exercise| {
                    let mut ex_clean = ex.clone();
                    ex_clean.image = None;
                    let text = sharing::format_exercise_text(&ex_clean);
                    let data = ShareableData::Exercise { exercise: ex_clean };
                    share_target_c.set(Some((data, text)));
                });
                html! { <ExerciseDetail exercise={ex.clone()} on_back={on_back} on_share={on_share} /> }
            } else if *show_custom_form {
                html! {
                    <div class="px-4 py-4">
                        <CustomExerciseForm
                            on_save={on_save_custom}
                            on_cancel={let s = show_custom_form.clone(); Callback::from(move |_| s.set(false))}
                        />
                    </div>
                }
            } else {
                html! {
                    <>
                        <div class="px-4 pt-4 pb-2 flex justify-between items-center transition-colors duration-200">
                            <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">{"Exercises"}</h1>
                            <button
                                class="px-3 py-1.5 bg-blue-600 text-white rounded-lg text-sm font-bold hover:bg-blue-700 neu-btn transition-colors"
                                onclick={let s = show_custom_form.clone(); Callback::from(move |_| s.set(true))}
                            >{"+ Custom"}</button>
                        </div>
                        <ExerciseList
                            exercises={all_exercises}
                            on_select={on_select}
                            show_add_button={false}
                        />
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
