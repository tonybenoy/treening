use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Category {
    Chest,
    Back,
    Legs,
    Shoulders,
    Arms,
    Core,
    Cardio,
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Category::Chest => write!(f, "Chest"),
            Category::Back => write!(f, "Back"),
            Category::Legs => write!(f, "Legs"),
            Category::Shoulders => write!(f, "Shoulders"),
            Category::Arms => write!(f, "Arms"),
            Category::Core => write!(f, "Core"),
            Category::Cardio => write!(f, "Cardio"),
        }
    }
}

impl Category {
    pub fn all() -> Vec<Category> {
        vec![
            Category::Chest,
            Category::Back,
            Category::Legs,
            Category::Shoulders,
            Category::Arms,
            Category::Core,
            Category::Cardio,
        ]
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Equipment {
    Barbell,
    Dumbbell,
    Machine,
    Cable,
    Bodyweight,
    Kettlebell,
    Band,
    Other,
}

impl fmt::Display for Equipment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Equipment::Barbell => write!(f, "Barbell"),
            Equipment::Dumbbell => write!(f, "Dumbbell"),
            Equipment::Machine => write!(f, "Machine"),
            Equipment::Cable => write!(f, "Cable"),
            Equipment::Bodyweight => write!(f, "Bodyweight"),
            Equipment::Kettlebell => write!(f, "Kettlebell"),
            Equipment::Band => write!(f, "Band"),
            Equipment::Other => write!(f, "Other"),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum ExerciseTrackingType {
    #[default]
    Strength,   // Weight + Reps (Default)
    Cardio,     // Distance + Duration
    Duration,   // Duration only (e.g. Plank)
    Bodyweight, // Reps only
}


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Exercise {
    pub id: String,
    pub name: String,
    pub category: Category,
    pub equipment: Equipment,
    pub muscle_groups: Vec<String>,
    pub description: String,
    pub is_custom: bool,
    pub image: Option<String>,
    #[serde(default)]
    pub tracking_type: ExerciseTrackingType,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorkoutSet {
    #[serde(default)]
    pub weight: f64,
    #[serde(default)]
    pub reps: u32,
    #[serde(default)]
    pub distance: Option<f64>,
    #[serde(default)]
    pub duration_secs: Option<u32>,
    pub completed: bool,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorkoutExercise {
    pub exercise_id: String,
    pub sets: Vec<WorkoutSet>,
    pub notes: String,
    #[serde(default)]
    pub superset_group: Option<u32>,
    #[serde(default)]
    pub rest_seconds_override: Option<u32>,
}

impl WorkoutExercise {
    pub fn volume(&self) -> f64 {
        self.sets.iter()
            .filter(|s| s.completed)
            .map(|s| {
                if let Some(dist) = s.distance {
                    dist * 10.0 // Arbitrary cardio weight: 1km = 10kg volume for ranking
                } else if let Some(secs) = s.duration_secs {
                    secs as f64 / 6.0 // Arbitrary duration weight: 1min = 10kg volume
                } else {
                    s.weight * s.reps as f64
                }
            })
            .sum()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Workout {
    pub id: String,
    pub date: String,
    pub name: String,
    pub exercises: Vec<WorkoutExercise>,
    pub duration_mins: u32,
}

impl Workout {
    pub fn total_volume(&self) -> f64 {
        self.exercises.iter().map(|e| e.volume()).sum()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Routine {
    pub id: String,
    pub name: String,
    pub exercise_ids: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BodyMetric {
    pub id: String,
    pub date: String,
    pub weight: Option<f64>,
    pub body_fat: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TrustedDevice {
    pub peer_id: String,
    pub name: String,
    pub last_synced: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AppData {
    pub workouts: Vec<Workout>,
    pub routines: Vec<Routine>,
    pub custom_exercises: Vec<Exercise>,
    pub friends: Vec<Friend>,
    pub body_metrics: Vec<BodyMetric>,
    pub user_config: Option<UserConfig>,
    #[serde(default)]
    pub trusted_devices: Vec<TrustedDevice>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Friend {
    pub id: String, // PeerID
    pub name: String,
    pub last_stats: Option<FriendStats>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FriendStats {
    pub workouts_this_week: u32,
    pub total_volume_kg: f64,
    pub last_active: String,
    #[serde(default)]
    pub body_weight: Option<f64>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub enum Theme {
    #[default]
    Dark,
    Light,
    System,
}


#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UserConfig {
    pub nickname: String,
    pub peer_id: String,
    #[serde(default = "default_social_enabled")]
    pub social_enabled: bool,
    #[serde(default)]
    pub theme: Theme,
    #[serde(default)]
    pub height: Option<f64>,
    #[serde(default)]
    pub birth_date: Option<String>,
    #[serde(default)]
    pub gender: Option<String>,
    #[serde(default = "default_rest_seconds")]
    pub rest_seconds: u32,
    #[serde(default = "default_bar_weight")]
    pub bar_weight: f64,
}

fn default_rest_seconds() -> u32 {
    90
}

fn default_bar_weight() -> f64 {
    20.0
}

fn default_social_enabled() -> bool {
    true
}


