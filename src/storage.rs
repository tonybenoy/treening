use crate::backup;
use crate::models::{
    AppData, BodyMetric, Exercise, Friend, Routine, TrustedDevice, UserConfig, Workout,
};
use gloo::storage::{LocalStorage, Storage};
use std::cell::Cell;

const WORKOUTS_KEY: &str = "treening_workouts";
const ROUTINES_KEY: &str = "treening_routines";
const CUSTOM_EXERCISES_KEY: &str = "treening_custom_exercises";
const FRIENDS_KEY: &str = "treening_friends";
const BODY_METRICS_KEY: &str = "treening_body_metrics";
const USER_CONFIG_KEY: &str = "treening_user_config";
const TRUSTED_DEVICES_KEY: &str = "treening_trusted_devices";

const BACKUP_DEBOUNCE_MS: f64 = 5000.0;

thread_local! {
    static SAVE_FAILED: Cell<bool> = const { Cell::new(false) };
    static LAST_BACKUP_TIME: Cell<f64> = const { Cell::new(0.0) };
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
            peer_id: format!("tr-{}", &uuid::Uuid::new_v4().to_string()[..8]),
            social_enabled: true,
            theme: crate::models::Theme::Dark,
            height: None,
            birth_date: None,
            gender: None,
            rest_seconds: 90,
            bar_weight: 20.0,
            unit_system: crate::models::UnitSystem::Metric,
            ai_enabled: false,
            ai_model: crate::models::AiModel::default(),
            muscle_thresholds: None,
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

pub fn load_trusted_devices() -> Vec<TrustedDevice> {
    LocalStorage::get(TRUSTED_DEVICES_KEY).unwrap_or_default()
}

pub fn save_trusted_devices(devices: &[TrustedDevice]) {
    check_save_result(LocalStorage::set(TRUSTED_DEVICES_KEY, devices));
    trigger_backup_debounced();
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

pub fn export_csv() -> String {
    let workouts = load_workouts();
    let mut lines = Vec::new();
    lines.push("date,workout_name,duration_mins,exercise,set_number,weight_kg,reps,distance_km,duration_secs,completed,note".to_string());

    let exercises = {
        let mut exs = crate::data::default_exercises();
        exs.extend(load_custom_exercises());
        exs
    };

    for w in &workouts {
        for we in &w.exercises {
            let ex_name = exercises
                .iter()
                .find(|e| e.id == we.exercise_id)
                .map(|e| e.name.clone())
                .unwrap_or_else(|| we.exercise_id.clone());
            for (i, s) in we.sets.iter().enumerate() {
                let dist = s.distance.map(|d| format!("{}", d)).unwrap_or_default();
                let dur = s
                    .duration_secs
                    .map(|d| format!("{}", d))
                    .unwrap_or_default();
                let note = s.note.as_deref().unwrap_or("");
                lines.push(format!(
                    "{},{},{},{},{},{},{},{},{},{},{}",
                    csv_escape(&w.date),
                    csv_escape(&w.name),
                    w.duration_mins,
                    csv_escape(&ex_name),
                    i + 1,
                    s.weight,
                    s.reps,
                    dist,
                    dur,
                    s.completed,
                    csv_escape(note),
                ));
            }
        }
    }
    lines.join("\n")
}

pub fn export_all_data() -> String {
    let data = AppData {
        workouts: load_workouts(),
        routines: load_routines(),
        custom_exercises: load_custom_exercises(),
        friends: load_friends(),
        body_metrics: load_body_metrics(),
        user_config: Some(load_user_config()),
        trusted_devices: load_trusted_devices(),
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
    save_trusted_devices(&data.trusted_devices);
    if let Some(config) = data.user_config {
        save_user_config(&config);
    }
    Ok(())
}

/// Check if LocalStorage appears empty; if so, try restoring from IndexedDB backup.
pub fn try_restore_from_backup() {
    // Use raw get_item to check key existence — LocalStorage::get::<String> fails
    // for JSON objects/arrays, which would incorrectly trigger restore every time.
    let storage = gloo::utils::window().local_storage().ok().flatten();
    let has_data = storage
        .map(|s| {
            s.get_item(WORKOUTS_KEY).ok().flatten().is_some()
                || s.get_item(ROUTINES_KEY).ok().flatten().is_some()
                || s.get_item(USER_CONFIG_KEY).ok().flatten().is_some()
        })
        .unwrap_or(false);

    if has_data {
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

    // Merge Trusted Devices (deduplicate by peer_id)
    let mut current_devices = load_trusted_devices();
    for incoming_d in incoming.trusted_devices {
        if !current_devices
            .iter()
            .any(|d| d.peer_id == incoming_d.peer_id)
        {
            current_devices.push(incoming_d);
        }
    }
    save_trusted_devices(&current_devices);

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
