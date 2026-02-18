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
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorkoutSet {
    pub weight: f64,
    pub reps: u32,
    pub completed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorkoutExercise {
    pub exercise_id: String,
    pub sets: Vec<WorkoutSet>,
    pub notes: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Workout {
    pub id: String,
    pub date: String,
    pub name: String,
    pub exercises: Vec<WorkoutExercise>,
    pub duration_mins: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Routine {
    pub id: String,
    pub name: String,
    pub exercise_ids: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppData {
    pub workouts: Vec<Workout>,
    pub routines: Vec<Routine>,
    pub custom_exercises: Vec<Exercise>,
    pub friends: Vec<Friend>,
    pub user_config: Option<UserConfig>,
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
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Theme {
    Dark,
    Light,
    System,
}

impl Default for Theme {
    fn default() -> Self {
        Self::Dark
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UserConfig {
    pub nickname: String,
    pub peer_id: String,
    #[serde(default = "default_social_enabled")]
    pub social_enabled: bool,
    #[serde(default)]
    pub theme: Theme,
}

fn default_social_enabled() -> bool {
    true
}

impl Default for AppData {
    fn default() -> Self {
        Self {
            workouts: Vec::new(),
            routines: Vec::new(),
            custom_exercises: Vec::new(),
            friends: Vec::new(),
            user_config: None,
        }
    }
}
