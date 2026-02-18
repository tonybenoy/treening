use yew::prelude::*;
use gloo::storage::{LocalStorage, Storage};
use crate::components::routine_editor::RoutineEditor;
use crate::models::{Exercise, Routine};
use crate::storage;
use crate::data;
use yew_router::prelude::*;
use crate::Route;

#[function_component(RoutinesPage)]
pub fn routines_page() -> Html {
    let routines = use_state(storage::load_routines);
    let navigator = use_navigator().unwrap();

    let custom_exercises = storage::load_custom_exercises();
    let all_exercises: Vec<Exercise> = {
        let mut exs = data::default_exercises();
        exs.extend(custom_exercises);
        exs
    };

    let on_save_routine = {
        let routines = routines.clone();
        Callback::from(move |routine: Routine| {
            let mut rs = (*routines).clone();
            if let Some(pos) = rs.iter().position(|r| r.id == routine.id) {
                rs[pos] = routine;
            } else {
                rs.push(routine);
            }
            storage::save_routines(&rs);
            routines.set(rs);
        })
    };

    let on_delete_routine = {
        let routines = routines.clone();
        Callback::from(move |id: String| {
            let mut rs = (*routines).clone();
            rs.retain(|r| r.id != id);
            storage::save_routines(&rs);
            routines.set(rs);
        })
    };

    let on_start_from_routine = {
        let nav = navigator.clone();
        Callback::from(move |routine: Routine| {
            let _ = LocalStorage::set("treening_active_routine", routine.id);
            nav.push(&Route::Workout);
        })
    };

    html! {
        <div class="pb-20 transition-colors duration-200">
            <div class="px-4 pt-4 pb-2">
                <h1 class="text-2xl font-bold mb-1 text-gray-900 dark:text-gray-100">{"Routines"}</h1>
                <p class="text-gray-500 dark:text-gray-400 text-sm mb-3">{"Plan your workout routines and start sessions from them."}</p>
            </div>
            <RoutineEditor
                routines={(*routines).clone()}
                all_exercises={all_exercises}
                on_save={on_save_routine}
                on_delete={on_delete_routine}
                on_start_workout={on_start_from_routine}
            />
        </div>
    }
}
