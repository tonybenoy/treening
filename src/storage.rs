use gloo::storage::{LocalStorage, Storage};
use crate::models::{AppData, Exercise, Friend, Routine, UserConfig, Workout, BodyMetric};
use crate::backup;
use std::cell::Cell;

const WORKOUTS_KEY: &str = "treening_workouts";
const ROUTINES_KEY: &str = "treening_routines";
const CUSTOM_EXERCISES_KEY: &str = "treening_custom_exercises";
const FRIENDS_KEY: &str = "treening_friends";
const BODY_METRICS_KEY: &str = "treening_body_metrics";
const USER_CONFIG_KEY: &str = "treening_user_config";

const BACKUP_DEBOUNCE_MS: f64 = 5000.0;

thread_local! {
    static SAVE_FAILED: Cell<bool> = Cell::new(false);
    static LAST_BACKUP_TIME: Cell<f64> = Cell::new(0.0);
}

pub fn has_save_failed() -> bool {
    SAVE_FAILED.with(|f| f.get())
}

pub fn clear_save_failed() {
    SAVE_FAILED.with(|f| f.set(false));
}

fn check_save_result<T>(result: Result<(), T>) {
    if result.is_err() {
        log::warn!("LocalStorage write failed — storage may be full");
        SAVE_FAILED.with(|f| f.set(true));
    }
}

fn trigger_backup_debounced() {
    let now = js_sys::Date::now();
    let should_backup = LAST_BACKUP_TIME.with(|t| {
        if now - t.get() > BACKUP_DEBOUNCE_MS {
            t.set(now);
            true
        } else {
            false
        }
    });
    if should_backup {
        let data = export_all_data();
        backup::save_backup(&data);
    }
}

pub fn load_workouts() -> Vec<Workout> {
    LocalStorage::get(WORKOUTS_KEY).unwrap_or_default()
}

pub fn save_workouts(workouts: &[Workout]) {
    check_save_result(LocalStorage::set(WORKOUTS_KEY, workouts));
    trigger_backup_debounced();
}

pub fn load_friends() -> Vec<Friend> {
    LocalStorage::get(FRIENDS_KEY).unwrap_or_default()
}

pub fn save_friends(friends: &[Friend]) {
    check_save_result(LocalStorage::set(FRIENDS_KEY, friends));
    trigger_backup_debounced();
}

pub fn load_body_metrics() -> Vec<BodyMetric> {
    LocalStorage::get(BODY_METRICS_KEY).unwrap_or_default()
}

pub fn save_body_metrics(metrics: &[BodyMetric]) {
    check_save_result(LocalStorage::set(BODY_METRICS_KEY, metrics));
    trigger_backup_debounced();
}

pub fn load_user_config() -> UserConfig {
    LocalStorage::get(USER_CONFIG_KEY).unwrap_or_else(|_| {
        let config = UserConfig {
            nickname: "Athlete".to_string(),
            peer_id: format!("tr-{}", uuid::Uuid::new_v4().to_string()[..8].to_string()),
            social_enabled: true,
            theme: crate::models::Theme::Dark,
            height: None,
            birth_date: None,
            gender: None,
        };
        let _ = LocalStorage::set(USER_CONFIG_KEY, &config);
        config
    })
}

pub fn save_user_config(config: &UserConfig) {
    check_save_result(LocalStorage::set(USER_CONFIG_KEY, config));
    trigger_backup_debounced();
}

pub fn load_routines() -> Vec<Routine> {
    LocalStorage::get(ROUTINES_KEY).unwrap_or_default()
}

pub fn save_routines(routines: &[Routine]) {
    check_save_result(LocalStorage::set(ROUTINES_KEY, routines));
    trigger_backup_debounced();
}

pub fn load_custom_exercises() -> Vec<Exercise> {
    LocalStorage::get(CUSTOM_EXERCISES_KEY).unwrap_or_default()
}

pub fn save_custom_exercises(exercises: &[Exercise]) {
    check_save_result(LocalStorage::set(CUSTOM_EXERCISES_KEY, exercises));
    trigger_backup_debounced();
}

pub fn export_all_data() -> String {
    let data = AppData {
        workouts: load_workouts(),
        routines: load_routines(),
        custom_exercises: load_custom_exercises(),
        friends: load_friends(),
        body_metrics: load_body_metrics(),
        user_config: Some(load_user_config()),
    };
    serde_json::to_string_pretty(&data).unwrap_or_default()
}

pub fn import_all_data(json: &str) -> Result<(), String> {
    let data: AppData = serde_json::from_str(json).map_err(|e| e.to_string())?;
    save_workouts(&data.workouts);
    save_routines(&data.routines);
    save_custom_exercises(&data.custom_exercises);
    save_friends(&data.friends);
    save_body_metrics(&data.body_metrics);
    if let Some(config) = data.user_config {
        save_user_config(&config);
    }
    Ok(())
}

/// Force an immediate backup to IndexedDB (ignoring debounce).
pub fn save_all_to_backup() {
    let data = export_all_data();
    backup::save_backup(&data);
    LAST_BACKUP_TIME.with(|t| t.set(js_sys::Date::now()));
}

/// Check if LocalStorage appears empty; if so, try restoring from IndexedDB backup.
pub fn try_restore_from_backup() {
    let has_workouts: bool = LocalStorage::get::<String>(WORKOUTS_KEY).is_ok();
    let has_routines: bool = LocalStorage::get::<String>(ROUTINES_KEY).is_ok();
    let has_config: bool = LocalStorage::get::<String>(USER_CONFIG_KEY).is_ok();

    if has_workouts || has_routines || has_config {
        // LocalStorage has data, no need to restore
        return;
    }

    log::info!("LocalStorage appears empty, checking IndexedDB backup...");
    backup::load_backup(|data| {
        if let Some(json) = data {
            if !json.is_empty() {
                log::info!("Restoring data from IndexedDB backup");
                match import_all_data(&json) {
                    Ok(()) => {
                        log::info!("Successfully restored from backup, reloading...");
                        let _ = gloo::utils::window().location().reload();
                    }
                    Err(e) => {
                        log::warn!("Failed to restore from backup: {}", e);
                    }
                }
            }
        }
    });
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

    // Merge Friends (deduplicate by ID)
    let mut current_friends = load_friends();
    for incoming_f in incoming.friends {
        if !current_friends.iter().any(|f| f.id == incoming_f.id) {
            current_friends.push(incoming_f);
        }
    }
    save_friends(&current_friends);

    // Merge Body Metrics (deduplicate by ID)
    let mut current_metrics = load_body_metrics();
    for incoming_m in incoming.body_metrics {
        if !current_metrics.iter().any(|m| m.id == incoming_m.id) {
            current_metrics.push(incoming_m);
        }
    }
    save_body_metrics(&current_metrics);

    // Merge User Config (keep local peer_id, but take incoming profile info if it's set)
    if let Some(incoming_config) = incoming.user_config {
        let mut local_config = load_user_config();
        if incoming_config.nickname != "Athlete" {
            local_config.nickname = incoming_config.nickname;
        }
        if incoming_config.height.is_some() {
            local_config.height = incoming_config.height;
        }
        if incoming_config.birth_date.is_some() {
            local_config.birth_date = incoming_config.birth_date;
        }
        if incoming_config.gender.is_some() {
            local_config.gender = incoming_config.gender;
        }
        save_user_config(&local_config);
    }

    Ok(())
}
