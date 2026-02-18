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

pub fn merge_all_data(json: &str) -> Result<(), String> {
    let incoming: AppData = serde_json::from_str(json).map_err(|e| e.to_string())?;
    
    // Merge Workouts (deduplicate by ID)
    let mut current_workouts = load_workouts();
    for incoming_w in incoming.workouts {
        if !current_workouts.iter().any(|w| w.id == incoming_w.id) {
            current_workouts.push(incoming_w);
        }
    }
    save_workouts(&current_workouts);

    // Merge Routines (deduplicate by ID)
    let mut current_routines = load_routines();
    for incoming_r in incoming.routines {
        if !current_routines.iter().any(|r| r.id == incoming_r.id) {
            current_routines.push(incoming_r);
        }
    }
    save_routines(&current_routines);

    // Merge Custom Exercises (deduplicate by ID)
    let mut current_custom = load_custom_exercises();
    for incoming_ex in incoming.custom_exercises {
        if !current_custom.iter().any(|e| e.id == incoming_ex.id) {
            current_custom.push(incoming_ex);
        }
    }
    save_custom_exercises(&current_custom);

    Ok(())
}
