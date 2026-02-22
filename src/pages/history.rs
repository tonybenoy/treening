use crate::components::history::HistoryList;
use crate::data;
use crate::models::{Exercise, Workout};
use crate::storage;
use yew::prelude::*;

#[function_component(HistoryPage)]
pub fn history_page() -> Html {
    let workouts = use_state(storage::load_workouts);

    let custom_exercises = storage::load_custom_exercises();
    let all_exercises: Vec<Exercise> = {
        let mut exs = data::default_exercises();
        exs.extend(custom_exercises);
        exs
    };

    let on_delete_workout = {
        let workouts = workouts.clone();
        Callback::from(move |id: String| {
            let mut ws = (*workouts).clone();
            ws.retain(|w| w.id != id);
            storage::save_workouts(&ws);
            workouts.set(ws);
        })
    };

    let on_update_workout = {
        let workouts = workouts.clone();
        Callback::from(move |updated: Workout| {
            let mut ws = (*workouts).clone();
            if let Some(pos) = ws.iter().position(|w| w.id == updated.id) {
                ws[pos] = updated;
            }
            storage::save_workouts(&ws);
            workouts.set(ws);
        })
    };

    html! {
        <div class="pb-20 transition-colors duration-200">
            <div class="px-4 pt-4 pb-2">
                <h1 class="text-2xl font-bold mb-1 text-gray-900 dark:text-gray-100">{"Workout History"}</h1>
                <p class="text-gray-500 dark:text-gray-400 text-sm mb-3">{"Your past workouts."}</p>
            </div>
            { if workouts.is_empty() {
                html! {
                    <div class="mx-4 text-center py-12 bg-gray-50 dark:bg-gray-800/20 rounded-2xl border border-dashed border-gray-200 dark:border-gray-700 transition-colors">
                        <p class="text-4xl mb-4">{"📋"}</p>
                        <p class="text-lg font-bold text-gray-900 dark:text-gray-100">{"No workouts yet!"}</p>
                        <p class="text-sm text-gray-500 dark:text-gray-400 mt-1 px-4">{"Tap \"Start New Workout\" on the home page to log your first session."}</p>
                    </div>
                }
            } else {
                html! {
                    <HistoryList
                        workouts={(*workouts).clone()}
                        all_exercises={all_exercises}
                        on_delete={on_delete_workout}
                        on_update={on_update_workout}
                    />
                }
            }}
        </div>
    }
}
