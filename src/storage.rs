use gloo::storage::{LocalStorage, Storage};
use crate::models::{AppData, Exercise, Routine, Workout};

const WORKOUTS_KEY: &str = "treening_workouts";
const ROUTINES_KEY: &str = "treening_routines";
const CUSTOM_EXERCISES_KEY: &str = "treening_custom_exercises";

pub fn load_workouts() -> Vec<Workout> {
    LocalStorage::get(WORKOUTS_KEY).unwrap_or_default()
}

pub fn save_workouts(workouts: &[Workout]) {
    let _ = LocalStorage::set(WORKOUTS_KEY, workouts);
}

pub fn load_routines() -> Vec<Routine> {
    LocalStorage::get(ROUTINES_KEY).unwrap_or_default()
}

pub fn save_routines(routines: &[Routine]) {
    let _ = LocalStorage::set(ROUTINES_KEY, routines);
}

pub fn load_custom_exercises() -> Vec<Exercise> {
    LocalStorage::get(CUSTOM_EXERCISES_KEY).unwrap_or_default()
}

pub fn save_custom_exercises(exercises: &[Exercise]) {
    let _ = LocalStorage::set(CUSTOM_EXERCISES_KEY, exercises);
}

pub fn export_all_data() -> String {
    let data = AppData {
        workouts: load_workouts(),
        routines: load_routines(),
        custom_exercises: load_custom_exercises(),
    };
    serde_json::to_string_pretty(&data).unwrap_or_default()
}

pub fn import_all_data(json: &str) -> Result<(), String> {
    let data: AppData = serde_json::from_str(json).map_err(|e| e.to_string())?;
    save_workouts(&data.workouts);
    save_routines(&data.routines);
    save_custom_exercises(&data.custom_exercises);
    Ok(())
}
