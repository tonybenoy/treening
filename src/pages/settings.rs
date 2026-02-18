use yew::prelude::*;
use yew_router::prelude::*;
use crate::components::settings::SettingsPanel;
use crate::components::sync::SyncPanel;
use crate::components::custom_exercise::CustomExerciseForm;
use crate::models::Exercise;
use crate::storage;
use crate::Route;

#[function_component(SettingsPage)]
pub fn settings_page() -> Html {
    let custom_exercises = use_state(|| storage::load_custom_exercises());
    let show_custom_form = use_state(|| false);
    let reload_trigger = use_state(|| 0u32);

    let on_import_complete = {
        let r = reload_trigger.clone();
        let ce = custom_exercises.clone();
        Callback::from(move |_| {
            ce.set(storage::load_custom_exercises());
            r.set(*r + 1);
        })
    };

    let on_save_custom = {
        let ce = custom_exercises.clone();
        let show = show_custom_form.clone();
        Callback::from(move |ex: Exercise| {
            let mut exs = (*ce).clone();
            exs.push(ex);
            storage::save_custom_exercises(&exs);
            ce.set(exs);
            show.set(false);
        })
    };

    let on_delete_custom = {
        let ce = custom_exercises.clone();
        Callback::from(move |id: String| {
            let mut exs = (*ce).clone();
            exs.retain(|e| e.id != id);
            storage::save_custom_exercises(&exs);
            ce.set(exs);
        })
    };

    html! {
        <div class="px-4 py-4 pb-20 space-y-6">
            <h1 class="text-2xl font-bold">{"Settings"}</h1>

            <SyncPanel />

            <SettingsPanel on_import_complete={on_import_complete} />

            <div class="bg-gray-800 rounded-lg p-4">
                <h3 class="font-semibold mb-3">{"App Info"}</h3>
                <Link<Route> to={Route::Faq} classes="flex items-center justify-between py-2 text-blue-400 hover:text-blue-300">
                    <span>{"Frequently Asked Questions"}</span>
                    <span>{"→"}</span>
                </Link<Route>>
            </div>

            <div>
                <div class="flex justify-between items-center mb-3">
                    <h2 class="text-lg font-semibold">{"Custom Exercises"}</h2>
                    <button
                        class="px-3 py-1.5 bg-blue-600 rounded text-sm hover:bg-blue-700"
                        onclick={let s = show_custom_form.clone(); Callback::from(move |_| s.set(true))}
                    >{"+ Add"}</button>
                </div>

                { if *show_custom_form {
                    html! {
                        <CustomExerciseForm
                            on_save={on_save_custom}
                            on_cancel={let s = show_custom_form.clone(); Callback::from(move |_| s.set(false))}
                        />
                    }
                } else { html! {} }}

                <div class="space-y-2">
                    { for custom_exercises.iter().map(|ex| {
                        let on_delete = on_delete_custom.clone();
                        let eid = ex.id.clone();
                        html! {
                            <div class="bg-gray-800 rounded-lg p-3 flex justify-between items-center">
                                <div>
                                    <div class="font-medium">{&ex.name}</div>
                                    <div class="text-sm text-gray-400">
                                        {ex.category.to_string()}{" · "}{ex.equipment.to_string()}
                                    </div>
                                </div>
                                <button
                                    class="text-red-400 text-sm hover:text-red-300"
                                    onclick={Callback::from(move |_| on_delete.emit(eid.clone()))}
                                >{"Delete"}</button>
                            </div>
                        }
                    })}
                    { if custom_exercises.is_empty() && !*show_custom_form {
                        html! { <p class="text-gray-500 text-sm">{"No custom exercises yet."}</p> }
                    } else { html! {} }}
                </div>
            </div>
        </div>
    }
}
