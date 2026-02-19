use serde::{Deserialize, Serialize};
use crate::models::{Exercise, Routine, Workout};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ShareableData {
    Workout { workout: Workout, exercises: Vec<Exercise> },
    Routine { routine: Routine, exercises: Vec<Exercise> },
    Exercise { exercise: Exercise },
}

pub fn encode(data: &ShareableData) -> Result<String, String> {
    let json = serde_json::to_string(data).map_err(|e| e.to_string())?;
    let compressed = miniz_oxide::deflate::compress_to_vec(json.as_bytes(), 9);
    use base64::Engine;
    Ok(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&compressed))
}

pub fn decode(encoded: &str) -> Result<ShareableData, String> {
    use base64::Engine;
    let compressed = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(encoded)
        .map_err(|e| format!("Base64 decode error: {}", e))?;
    let json_bytes = miniz_oxide::inflate::decompress_to_vec(&compressed)
        .map_err(|e| format!("Decompress error: {:?}", e))?;
    let json = String::from_utf8(json_bytes).map_err(|e| e.to_string())?;
    serde_json::from_str(&json).map_err(|e| format!("JSON parse error: {}", e))
}

const MAX_URL_LEN: usize = 2000;

pub fn build_share_url(data: &ShareableData) -> Result<String, String> {
    let encoded = encode(data)?;
    let url = format!("https://treen.ing/#/shared?d={}", encoded);
    if url.len() > MAX_URL_LEN {
        Err("URL too large to share as link".to_string())
    } else {
        Ok(url)
    }
}

pub fn collect_workout_exercises(workout: &Workout, all_exercises: &[Exercise]) -> Vec<Exercise> {
    let ids: Vec<&str> = workout.exercises.iter().map(|we| we.exercise_id.as_str()).collect();
    all_exercises
        .iter()
        .filter(|e| ids.contains(&e.id.as_str()))
        .map(|e| {
            let mut ex = e.clone();
            ex.image = None;
            ex
        })
        .collect()
}

pub fn collect_routine_exercises(routine: &Routine, all_exercises: &[Exercise]) -> Vec<Exercise> {
    all_exercises
        .iter()
        .filter(|e| routine.exercise_ids.contains(&e.id))
        .map(|e| {
            let mut ex = e.clone();
            ex.image = None;
            ex
        })
        .collect()
}

pub fn format_workout_text(workout: &Workout, exercises: &[Exercise]) -> String {
    let units = crate::storage::load_user_config().unit_system;
    let wl = units.weight_label();
    let dl = units.distance_label();
    let mut lines = Vec::new();
    lines.push(format!("Workout: {}", workout.name));
    lines.push(format!("Date: {}", workout.date));
    if workout.duration_mins > 0 {
        lines.push(format!("Duration: {} min", workout.duration_mins));
    }
    lines.push(String::new());

    for we in &workout.exercises {
        let name = exercises
            .iter()
            .find(|e| e.id == we.exercise_id)
            .map(|e| e.name.as_str())
            .unwrap_or(&we.exercise_id);
        let superset_tag = if we.superset_group.is_some() { " [Superset]" } else { "" };
        lines.push(format!("  {}{}", name, superset_tag));
        for (i, s) in we.sets.iter().enumerate() {
            if s.completed {
                let mut set_line = if let Some(dist) = s.distance {
                    let dur = s.duration_secs.unwrap_or(0);
                    format!("    Set {}: {:.1}{} / {}:{:02}", i + 1, units.display_distance(dist), dl, dur / 60, dur % 60)
                } else if let Some(secs) = s.duration_secs {
                    format!("    Set {}: {}:{:02}", i + 1, secs / 60, secs % 60)
                } else {
                    format!("    Set {}: {:.1}{} x {}", i + 1, units.display_weight(s.weight), wl, s.reps)
                };
                if let Some(ref note) = s.note {
                    if !note.is_empty() {
                        set_line.push_str(&format!(" ({})", note));
                    }
                }
                lines.push(set_line);
            }
        }
        if !we.notes.is_empty() {
            lines.push(format!("    Note: {}", we.notes));
        }
    }

    let total_vol = workout.total_volume();
    if total_vol > 0.0 {
        lines.push(String::new());
        lines.push(format!("Total volume: {:.0} {}", units.display_weight(total_vol), wl));
    }
    lines.push(String::new());
    lines.push("Shared from Treening - https://treen.ing/".to_string());
    lines.join("\n")
}

pub fn format_routine_text(routine: &Routine, exercises: &[Exercise]) -> String {
    let mut lines = Vec::new();
    lines.push(format!("Routine: {}", routine.name));
    lines.push(String::new());
    for eid in &routine.exercise_ids {
        if let Some(ex) = exercises.iter().find(|e| &e.id == eid) {
            lines.push(format!("  {} ({}, {})", ex.name, ex.category, ex.equipment));
        }
    }
    lines.push(String::new());
    lines.push("Shared from Treening - https://treen.ing/".to_string());
    lines.join("\n")
}

pub fn format_exercise_text(exercise: &Exercise) -> String {
    let mut lines = Vec::new();
    lines.push(format!("Exercise: {}", exercise.name));
    lines.push(format!("Category: {}", exercise.category));
    lines.push(format!("Equipment: {}", exercise.equipment));
    if !exercise.muscle_groups.is_empty() {
        lines.push(format!("Muscles: {}", exercise.muscle_groups.join(", ")));
    }
    if !exercise.description.is_empty() {
        lines.push(String::new());
        lines.push(exercise.description.clone());
    }
    lines.push(String::new());
    lines.push("Shared from Treening - https://treen.ing/".to_string());
    lines.join("\n")
}
