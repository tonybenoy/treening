use yew::prelude::*;
use crate::components::history::HistoryList;
use crate::models::{Exercise, Workout};
use crate::storage;
use crate::data;

#[function_component(HistoryPage)]
pub fn history_page() -> Html {
    let workouts = use_state(|| storage::load_workouts());

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
        <div class="pb-20">
            <div class="px-4 pt-4 pb-2">
                <h1 class="text-2xl font-bold mb-1">{"Workout History"}</h1>
                <p class="text-gray-400 text-sm mb-3">{"Your past workouts."}</p>
            </div>
            <HistoryList
                workouts={(*workouts).clone()}
                all_exercises={all_exercises}
                on_delete={on_delete_workout}
                on_update={on_update_workout}
            />
        </div>
    }
}
