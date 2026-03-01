use std::collections::HashMap;

pub struct MuscleContribution {
    pub muscle: &'static str,
    pub contribution: f64,
}

/// The 14 tracked muscle groups.
pub const TRACKED_MUSCLES: &[&str] = &[
    "Chest",
    "Lats",
    "Traps",
    "Front Delts",
    "Side Delts",
    "Rear Delts",
    "Biceps",
    "Triceps",
    "Forearms",
    "Quads",
    "Hamstrings",
    "Glutes",
    "Calves",
    "Abs",
];

/// Default MEV (minimum effective volume) and MRV (maximum recoverable volume)
/// per muscle group per week, based on RP Volume Landmarks (Israetel).
/// Returns (MEV, MRV) tuple.
pub fn default_thresholds() -> HashMap<&'static str, (f64, f64)> {
    let mut m = HashMap::new();
    m.insert("Chest", (6.0, 22.0));
    m.insert("Lats", (10.0, 25.0));
    m.insert("Traps", (0.0, 26.0));
    m.insert("Front Delts", (0.0, 12.0));
    m.insert("Side Delts", (8.0, 26.0));
    m.insert("Rear Delts", (8.0, 26.0));
    m.insert("Biceps", (8.0, 26.0));
    m.insert("Triceps", (6.0, 18.0));
    m.insert("Forearms", (2.0, 12.0));
    m.insert("Quads", (8.0, 20.0));
    m.insert("Hamstrings", (6.0, 20.0));
    m.insert("Glutes", (0.0, 16.0));
    m.insert("Calves", (8.0, 20.0));
    m.insert("Abs", (0.0, 25.0));
    m
}

/// Body region groupings for display.
pub const PUSH_MUSCLES: &[&str] = &["Chest", "Front Delts", "Side Delts", "Triceps"];
pub const PULL_MUSCLES: &[&str] = &["Lats", "Traps", "Rear Delts", "Biceps", "Forearms"];
pub const LEG_MUSCLES: &[&str] = &["Quads", "Hamstrings", "Glutes", "Calves"];
pub const CORE_MUSCLES: &[&str] = &["Abs"];

/// Get muscle contributions for a built-in exercise by ID.
/// Returns empty vec for unknown exercises.
pub fn exercise_muscles(exercise_id: &str) -> Vec<MuscleContribution> {
    match exercise_id {
        // === CHEST ===
        "chest-01" => vec![
            // Barbell Bench Press
            MuscleContribution {
                muscle: "Chest",
                contribution: 1.0,
            },
            MuscleContribution {
                muscle: "Triceps",
                contribution: 0.5,
            },
            MuscleContribution {
                muscle: "Front Delts",
                contribution: 0.3,
            },
        ],
        "chest-02" => vec![
            // Incline Barbell Bench Press
            MuscleContribution {
                muscle: "Chest",
                contribution: 1.0,
            },
            MuscleContribution {
                muscle: "Triceps",
                contribution: 0.5,
            },
            MuscleContribution {
                muscle: "Front Delts",
                contribution: 0.4,
            },
        ],
        "chest-03" => vec![
            // Decline Barbell Bench Press
            MuscleContribution {
                muscle: "Chest",
                contribution: 1.0,
            },
            MuscleContribution {
                muscle: "Triceps",
                contribution: 0.5,
            },
        ],
        "chest-04" => vec![
            // Dumbbell Bench Press
            MuscleContribution {
                muscle: "Chest",
                contribution: 1.0,
            },
            MuscleContribution {
                muscle: "Triceps",
                contribution: 0.5,
            },
            MuscleContribution {
                muscle: "Front Delts",
                contribution: 0.3,
            },
        ],
        "chest-05" => vec![
            // Incline Dumbbell Press
            MuscleContribution {
                muscle: "Chest",
                contribution: 1.0,
            },
            MuscleContribution {
                muscle: "Triceps",
                contribution: 0.5,
            },
            MuscleContribution {
                muscle: "Front Delts",
                contribution: 0.4,
            },
        ],
        "chest-06" => vec![
            // Dumbbell Fly
            MuscleContribution {
                muscle: "Chest",
                contribution: 1.0,
            },
        ],
        "chest-07" => vec![
            // Cable Fly
            MuscleContribution {
                muscle: "Chest",
                contribution: 1.0,
            },
        ],
        "chest-08" => vec![
            // Pec Deck Machine
            MuscleContribution {
                muscle: "Chest",
                contribution: 1.0,
            },
        ],
        "chest-09" => vec![
            // Machine Chest Press
            MuscleContribution {
                muscle: "Chest",
                contribution: 1.0,
            },
            MuscleContribution {
                muscle: "Triceps",
                contribution: 0.5,
            },
            MuscleContribution {
                muscle: "Front Delts",
                contribution: 0.3,
            },
        ],
        "chest-10" => vec![
            // Push-ups
            MuscleContribution {
                muscle: "Chest",
                contribution: 1.0,
            },
            MuscleContribution {
                muscle: "Triceps",
                contribution: 0.5,
            },
            MuscleContribution {
                muscle: "Front Delts",
                contribution: 0.3,
            },
            MuscleContribution {
                muscle: "Abs",
                contribution: 0.25,
            },
        ],
        "chest-11" => vec![
            // Dips (Chest)
            MuscleContribution {
                muscle: "Chest",
                contribution: 1.0,
            },
            MuscleContribution {
                muscle: "Triceps",
                contribution: 0.5,
            },
            MuscleContribution {
                muscle: "Front Delts",
                contribution: 0.3,
            },
        ],
        "chest-12" => vec![
            // Cable Crossover
            MuscleContribution {
                muscle: "Chest",
                contribution: 1.0,
            },
        ],
        "chest-13" => vec![
            // Smith Machine Bench Press
            MuscleContribution {
                muscle: "Chest",
                contribution: 1.0,
            },
            MuscleContribution {
                muscle: "Triceps",
                contribution: 0.5,
            },
            MuscleContribution {
                muscle: "Front Delts",
                contribution: 0.3,
            },
        ],
        "chest-14" => vec![
            // Decline Dumbbell Press
            MuscleContribution {
                muscle: "Chest",
                contribution: 1.0,
            },
            MuscleContribution {
                muscle: "Triceps",
                contribution: 0.5,
            },
        ],

        // === BACK ===
        "back-01" => vec![
            // Lat Pulldown
            MuscleContribution {
                muscle: "Lats",
                contribution: 1.0,
            },
            MuscleContribution {
                muscle: "Biceps",
                contribution: 0.5,
            },
            MuscleContribution {
                muscle: "Rear Delts",
                contribution: 0.25,
            },
        ],
        "back-02" => vec![
            // Seated Cable Row
            MuscleContribution {
                muscle: "Lats",
                contribution: 0.7,
            },
            MuscleContribution {
                muscle: "Traps",
                contribution: 0.5,
            },
            MuscleContribution {
                muscle: "Biceps",
                contribution: 0.5,
            },
            MuscleContribution {
                muscle: "Rear Delts",
                contribution: 0.3,
            },
        ],
        "back-03" => vec![
            // Barbell Bent-over Row
            MuscleContribution {
                muscle: "Lats",
                contribution: 0.7,
            },
            MuscleContribution {
                muscle: "Traps",
                contribution: 0.5,
            },
            MuscleContribution {
                muscle: "Biceps",
                contribution: 0.5,
            },
            MuscleContribution {
                muscle: "Rear Delts",
                contribution: 0.3,
            },
        ],
        "back-04" => vec![
            // Dumbbell Row
            MuscleContribution {
                muscle: "Lats",
                contribution: 1.0,
            },
            MuscleContribution {
                muscle: "Traps",
                contribution: 0.3,
            },
            MuscleContribution {
                muscle: "Biceps",
                contribution: 0.5,
            },
        ],
        "back-05" => vec![
            // Deadlift
            MuscleContribution {
                muscle: "Hamstrings",
                contribution: 0.5,
            },
            MuscleContribution {
                muscle: "Glutes",
                contribution: 0.5,
            },
            MuscleContribution {
                muscle: "Traps",
                contribution: 0.5,
            },
        ],
        "back-06" => vec![
            // Romanian Deadlift
            MuscleContribution {
                muscle: "Hamstrings",
                contribution: 1.0,
            },
            MuscleContribution {
                muscle: "Glutes",
                contribution: 0.5,
            },
        ],
        "back-07" => vec![
            // Pull-ups
            MuscleContribution {
                muscle: "Lats",
                contribution: 1.0,
            },
            MuscleContribution {
                muscle: "Biceps",
                contribution: 0.5,
            },
            MuscleContribution {
                muscle: "Rear Delts",
                contribution: 0.25,
            },
        ],
        "back-08" => vec![
            // Chin-ups
            MuscleContribution {
                muscle: "Lats",
                contribution: 1.0,
            },
            MuscleContribution {
                muscle: "Biceps",
                contribution: 0.7,
            },
        ],
        "back-09" => vec![
            // T-Bar Row
            MuscleContribution {
                muscle: "Lats",
                contribution: 0.7,
            },
            MuscleContribution {
                muscle: "Traps",
                contribution: 0.5,
            },
            MuscleContribution {
                muscle: "Biceps",
                contribution: 0.5,
            },
        ],
        "back-10" => vec![
            // Cable Pullover
            MuscleContribution {
                muscle: "Lats",
                contribution: 1.0,
            },
        ],
        "back-11" => vec![
            // Machine Row
            MuscleContribution {
                muscle: "Lats",
                contribution: 0.7,
            },
            MuscleContribution {
                muscle: "Traps",
                contribution: 0.5,
            },
            MuscleContribution {
                muscle: "Biceps",
                contribution: 0.5,
            },
        ],
        "back-12" => vec![
            // Hyperextension
            MuscleContribution {
                muscle: "Glutes",
                contribution: 0.5,
            },
            MuscleContribution {
                muscle: "Hamstrings",
                contribution: 0.5,
            },
        ],
        "back-13" => vec![
            // Vertical Traction
            MuscleContribution {
                muscle: "Lats",
                contribution: 1.0,
            },
            MuscleContribution {
                muscle: "Biceps",
                contribution: 0.5,
            },
            MuscleContribution {
                muscle: "Rear Delts",
                contribution: 0.25,
            },
        ],
        "back-14" => vec![
            // Seated Back Extension
            MuscleContribution {
                muscle: "Glutes",
                contribution: 0.25,
            },
        ],
        "back-15" => vec![
            // Assisted Chin/Dip
            MuscleContribution {
                muscle: "Lats",
                contribution: 0.7,
            },
            MuscleContribution {
                muscle: "Biceps",
                contribution: 0.5,
            },
        ],
        "back-16" => vec![
            // Dumbbell Pullover
            MuscleContribution {
                muscle: "Lats",
                contribution: 0.7,
            },
            MuscleContribution {
                muscle: "Chest",
                contribution: 0.3,
            },
        ],
        "back-17" => vec![
            // Straight Arm Pulldown
            MuscleContribution {
                muscle: "Lats",
                contribution: 1.0,
            },
        ],

        // === LEGS ===
        "legs-01" => vec![
            // Barbell Squat
            MuscleContribution {
                muscle: "Quads",
                contribution: 1.0,
            },
            MuscleContribution {
                muscle: "Glutes",
                contribution: 0.5,
            },
            MuscleContribution {
                muscle: "Hamstrings",
                contribution: 0.25,
            },
            MuscleContribution {
                muscle: "Abs",
                contribution: 0.25,
            },
        ],
        "legs-02" => vec![
            // Front Squat
            MuscleContribution {
                muscle: "Quads",
                contribution: 1.0,
            },
            MuscleContribution {
                muscle: "Glutes",
                contribution: 0.5,
            },
            MuscleContribution {
                muscle: "Abs",
                contribution: 0.3,
            },
        ],
        "legs-03" => vec![
            // Leg Press
            MuscleContribution {
                muscle: "Quads",
                contribution: 1.0,
            },
            MuscleContribution {
                muscle: "Glutes",
                contribution: 0.5,
            },
            MuscleContribution {
                muscle: "Hamstrings",
                contribution: 0.25,
            },
        ],
        "legs-04" => vec![
            // Leg Extension
            MuscleContribution {
                muscle: "Quads",
                contribution: 1.0,
            },
        ],
        "legs-05" => vec![
            // Leg Curl (Lying)
            MuscleContribution {
                muscle: "Hamstrings",
                contribution: 1.0,
            },
        ],
        "legs-06" => vec![
            // Leg Curl (Seated)
            MuscleContribution {
                muscle: "Hamstrings",
                contribution: 1.0,
            },
        ],
        "legs-07" => vec![
            // Hack Squat
            MuscleContribution {
                muscle: "Quads",
                contribution: 1.0,
            },
            MuscleContribution {
                muscle: "Glutes",
                contribution: 0.5,
            },
        ],
        "legs-08" => vec![
            // Bulgarian Split Squat
            MuscleContribution {
                muscle: "Quads",
                contribution: 1.0,
            },
            MuscleContribution {
                muscle: "Glutes",
                contribution: 0.5,
            },
            MuscleContribution {
                muscle: "Hamstrings",
                contribution: 0.25,
            },
        ],
        "legs-09" => vec![
            // Walking Lunges
            MuscleContribution {
                muscle: "Quads",
                contribution: 1.0,
            },
            MuscleContribution {
                muscle: "Glutes",
                contribution: 0.5,
            },
            MuscleContribution {
                muscle: "Hamstrings",
                contribution: 0.25,
            },
        ],
        "legs-10" => vec![
            // Calf Raise (Standing)
            MuscleContribution {
                muscle: "Calves",
                contribution: 1.0,
            },
        ],
        "legs-11" => vec![
            // Calf Raise (Seated)
            MuscleContribution {
                muscle: "Calves",
                contribution: 1.0,
            },
        ],
        "legs-12" => vec![
            // Goblet Squat
            MuscleContribution {
                muscle: "Quads",
                contribution: 1.0,
            },
            MuscleContribution {
                muscle: "Glutes",
                contribution: 0.5,
            },
            MuscleContribution {
                muscle: "Abs",
                contribution: 0.25,
            },
        ],
        "legs-13" => vec![
            // Hip Thrust
            MuscleContribution {
                muscle: "Glutes",
                contribution: 1.0,
            },
            MuscleContribution {
                muscle: "Hamstrings",
                contribution: 0.3,
            },
        ],
        "legs-14" => vec![
            // Leg Press Calf Raise
            MuscleContribution {
                muscle: "Calves",
                contribution: 1.0,
            },
        ],
        "legs-15" => vec![
            // Smith Machine Squat
            MuscleContribution {
                muscle: "Quads",
                contribution: 1.0,
            },
            MuscleContribution {
                muscle: "Glutes",
                contribution: 0.5,
            },
        ],
        "legs-16" => vec![
            // Hip Abductor
            MuscleContribution {
                muscle: "Glutes",
                contribution: 0.5,
            },
        ],
        "legs-17" => vec![
            // Hip Adductor
            // Mostly inner thighs — not a tracked 14-group muscle, skip or minimal glutes
            vec![],
        ]
        .into_iter()
        .flatten()
        .collect(),
        "legs-18" => vec![
            // Multi Hip
            MuscleContribution {
                muscle: "Glutes",
                contribution: 0.5,
            },
        ],
        "legs-19" => vec![
            // Hip Thrust Machine
            MuscleContribution {
                muscle: "Glutes",
                contribution: 1.0,
            },
            MuscleContribution {
                muscle: "Hamstrings",
                contribution: 0.3,
            },
        ],
        "legs-20" => vec![
            // Sumo Deadlift
            MuscleContribution {
                muscle: "Quads",
                contribution: 0.5,
            },
            MuscleContribution {
                muscle: "Glutes",
                contribution: 0.7,
            },
            MuscleContribution {
                muscle: "Hamstrings",
                contribution: 0.5,
            },
        ],
        "legs-21" => vec![
            // Reverse Lunge
            MuscleContribution {
                muscle: "Quads",
                contribution: 1.0,
            },
            MuscleContribution {
                muscle: "Glutes",
                contribution: 0.5,
            },
            MuscleContribution {
                muscle: "Hamstrings",
                contribution: 0.25,
            },
        ],
        "legs-22" => vec![
            // Step-ups
            MuscleContribution {
                muscle: "Quads",
                contribution: 1.0,
            },
            MuscleContribution {
                muscle: "Glutes",
                contribution: 0.5,
            },
        ],
        "legs-23" => vec![
            // Nordic Hamstring Curl
            MuscleContribution {
                muscle: "Hamstrings",
                contribution: 1.0,
            },
        ],
        "legs-24" => vec![
            // Glute Kickback Machine
            MuscleContribution {
                muscle: "Glutes",
                contribution: 1.0,
            },
            MuscleContribution {
                muscle: "Hamstrings",
                contribution: 0.25,
            },
        ],
        "legs-25" => vec![
            // Pendulum Squat
            MuscleContribution {
                muscle: "Quads",
                contribution: 1.0,
            },
            MuscleContribution {
                muscle: "Glutes",
                contribution: 0.5,
            },
        ],
        "legs-26" => vec![
            // Belt Squat
            MuscleContribution {
                muscle: "Quads",
                contribution: 1.0,
            },
            MuscleContribution {
                muscle: "Glutes",
                contribution: 0.5,
            },
            MuscleContribution {
                muscle: "Hamstrings",
                contribution: 0.25,
            },
        ],

        // === SHOULDERS ===
        "shldr-01" => vec![
            // Overhead Press (Barbell)
            MuscleContribution {
                muscle: "Front Delts",
                contribution: 1.0,
            },
            MuscleContribution {
                muscle: "Side Delts",
                contribution: 0.5,
            },
            MuscleContribution {
                muscle: "Triceps",
                contribution: 0.5,
            },
        ],
        "shldr-02" => vec![
            // Dumbbell Shoulder Press
            MuscleContribution {
                muscle: "Front Delts",
                contribution: 1.0,
            },
            MuscleContribution {
                muscle: "Side Delts",
                contribution: 0.5,
            },
            MuscleContribution {
                muscle: "Triceps",
                contribution: 0.5,
            },
        ],
        "shldr-03" => vec![
            // Lateral Raise
            MuscleContribution {
                muscle: "Side Delts",
                contribution: 1.0,
            },
        ],
        "shldr-04" => vec![
            // Front Raise
            MuscleContribution {
                muscle: "Front Delts",
                contribution: 1.0,
            },
        ],
        "shldr-05" => vec![
            // Face Pull
            MuscleContribution {
                muscle: "Rear Delts",
                contribution: 1.0,
            },
            MuscleContribution {
                muscle: "Traps",
                contribution: 0.3,
            },
        ],
        "shldr-06" => vec![
            // Rear Delt Fly
            MuscleContribution {
                muscle: "Rear Delts",
                contribution: 1.0,
            },
        ],
        "shldr-07" => vec![
            // Machine Shoulder Press
            MuscleContribution {
                muscle: "Front Delts",
                contribution: 1.0,
            },
            MuscleContribution {
                muscle: "Side Delts",
                contribution: 0.5,
            },
            MuscleContribution {
                muscle: "Triceps",
                contribution: 0.5,
            },
        ],
        "shldr-08" => vec![
            // Cable Lateral Raise
            MuscleContribution {
                muscle: "Side Delts",
                contribution: 1.0,
            },
        ],
        "shldr-09" => vec![
            // Arnold Press
            MuscleContribution {
                muscle: "Front Delts",
                contribution: 1.0,
            },
            MuscleContribution {
                muscle: "Side Delts",
                contribution: 0.5,
            },
            MuscleContribution {
                muscle: "Triceps",
                contribution: 0.3,
            },
        ],
        "shldr-10" => vec![
            // Upright Row
            MuscleContribution {
                muscle: "Side Delts",
                contribution: 1.0,
            },
            MuscleContribution {
                muscle: "Traps",
                contribution: 0.5,
            },
        ],
        "shldr-11" => vec![
            // Reverse Pec Deck
            MuscleContribution {
                muscle: "Rear Delts",
                contribution: 1.0,
            },
        ],
        "shldr-12" => vec![
            // Shrugs (Barbell)
            MuscleContribution {
                muscle: "Traps",
                contribution: 1.0,
            },
        ],
        "shldr-13" => vec![
            // Shrugs (Dumbbell)
            MuscleContribution {
                muscle: "Traps",
                contribution: 1.0,
            },
        ],
        "shldr-14" => vec![
            // Machine Lateral Raise
            MuscleContribution {
                muscle: "Side Delts",
                contribution: 1.0,
            },
        ],
        "shldr-15" => vec![
            // Landmine Press
            MuscleContribution {
                muscle: "Front Delts",
                contribution: 1.0,
            },
            MuscleContribution {
                muscle: "Chest",
                contribution: 0.3,
            },
            MuscleContribution {
                muscle: "Triceps",
                contribution: 0.3,
            },
        ],

        // === ARMS ===
        "arms-01" => vec![
            // Barbell Curl
            MuscleContribution {
                muscle: "Biceps",
                contribution: 1.0,
            },
        ],
        "arms-02" => vec![
            // Dumbbell Curl
            MuscleContribution {
                muscle: "Biceps",
                contribution: 1.0,
            },
        ],
        "arms-03" => vec![
            // Hammer Curl
            MuscleContribution {
                muscle: "Biceps",
                contribution: 0.7,
            },
            MuscleContribution {
                muscle: "Forearms",
                contribution: 0.5,
            },
        ],
        "arms-04" => vec![
            // Preacher Curl
            MuscleContribution {
                muscle: "Biceps",
                contribution: 1.0,
            },
        ],
        "arms-05" => vec![
            // Cable Curl
            MuscleContribution {
                muscle: "Biceps",
                contribution: 1.0,
            },
        ],
        "arms-06" => vec![
            // Concentration Curl
            MuscleContribution {
                muscle: "Biceps",
                contribution: 1.0,
            },
        ],
        "arms-07" => vec![
            // Tricep Pushdown
            MuscleContribution {
                muscle: "Triceps",
                contribution: 1.0,
            },
        ],
        "arms-08" => vec![
            // Overhead Tricep Extension
            MuscleContribution {
                muscle: "Triceps",
                contribution: 1.0,
            },
        ],
        "arms-09" => vec![
            // Skull Crushers
            MuscleContribution {
                muscle: "Triceps",
                contribution: 1.0,
            },
        ],
        "arms-10" => vec![
            // Tricep Dips
            MuscleContribution {
                muscle: "Triceps",
                contribution: 1.0,
            },
            MuscleContribution {
                muscle: "Chest",
                contribution: 0.3,
            },
            MuscleContribution {
                muscle: "Front Delts",
                contribution: 0.25,
            },
        ],
        "arms-11" => vec![
            // Cable Overhead Extension
            MuscleContribution {
                muscle: "Triceps",
                contribution: 1.0,
            },
        ],
        "arms-12" => vec![
            // Close-Grip Bench Press
            MuscleContribution {
                muscle: "Triceps",
                contribution: 1.0,
            },
            MuscleContribution {
                muscle: "Chest",
                contribution: 0.5,
            },
        ],
        "arms-13" => vec![
            // Wrist Curl
            MuscleContribution {
                muscle: "Forearms",
                contribution: 1.0,
            },
        ],
        "arms-14" => vec![
            // Reverse Curl
            MuscleContribution {
                muscle: "Forearms",
                contribution: 0.7,
            },
            MuscleContribution {
                muscle: "Biceps",
                contribution: 0.5,
            },
        ],
        "arms-15" => vec![
            // Machine Bicep Curl
            MuscleContribution {
                muscle: "Biceps",
                contribution: 1.0,
            },
        ],
        "arms-16" => vec![
            // Machine Tricep Extension
            MuscleContribution {
                muscle: "Triceps",
                contribution: 1.0,
            },
        ],
        "arms-17" => vec![
            // Incline Dumbbell Curl
            MuscleContribution {
                muscle: "Biceps",
                contribution: 1.0,
            },
        ],
        "arms-18" => vec![
            // EZ Bar Curl
            MuscleContribution {
                muscle: "Biceps",
                contribution: 1.0,
            },
        ],
        "arms-19" => vec![
            // Tricep Kickback
            MuscleContribution {
                muscle: "Triceps",
                contribution: 1.0,
            },
        ],
        "arms-20" => vec![
            // Spider Curl
            MuscleContribution {
                muscle: "Biceps",
                contribution: 1.0,
            },
        ],

        // === CORE ===
        "core-01" => vec![
            // Plank
            MuscleContribution {
                muscle: "Abs",
                contribution: 1.0,
            },
        ],
        "core-02" => vec![
            // Crunches
            MuscleContribution {
                muscle: "Abs",
                contribution: 1.0,
            },
        ],
        "core-03" => vec![
            // Hanging Leg Raise
            MuscleContribution {
                muscle: "Abs",
                contribution: 1.0,
            },
        ],
        "core-04" => vec![
            // Cable Crunch
            MuscleContribution {
                muscle: "Abs",
                contribution: 1.0,
            },
        ],
        "core-05" => vec![
            // Russian Twist
            MuscleContribution {
                muscle: "Abs",
                contribution: 1.0,
            },
        ],
        "core-06" => vec![
            // Ab Wheel Rollout
            MuscleContribution {
                muscle: "Abs",
                contribution: 1.0,
            },
        ],
        "core-07" => vec![
            // Mountain Climbers
            MuscleContribution {
                muscle: "Abs",
                contribution: 0.5,
            },
        ],
        "core-08" => vec![
            // Side Plank
            MuscleContribution {
                muscle: "Abs",
                contribution: 0.7,
            },
        ],
        "core-09" => vec![
            // Bicycle Crunch
            MuscleContribution {
                muscle: "Abs",
                contribution: 1.0,
            },
        ],
        "core-10" => vec![
            // Dead Bug
            MuscleContribution {
                muscle: "Abs",
                contribution: 1.0,
            },
        ],
        "core-11" => vec![
            // Decline Sit-up
            MuscleContribution {
                muscle: "Abs",
                contribution: 1.0,
            },
        ],
        "core-12" => vec![
            // Abdominal Crunch Machine
            MuscleContribution {
                muscle: "Abs",
                contribution: 1.0,
            },
        ],
        "core-13" => vec![
            // Total Abdominal Machine
            MuscleContribution {
                muscle: "Abs",
                contribution: 1.0,
            },
        ],
        "core-14" => vec![
            // Rotary Torso Machine
            MuscleContribution {
                muscle: "Abs",
                contribution: 0.7,
            },
        ],
        "core-15" => vec![
            // Cable Woodchop
            MuscleContribution {
                muscle: "Abs",
                contribution: 0.7,
            },
        ],
        "core-16" => vec![
            // Pallof Press
            MuscleContribution {
                muscle: "Abs",
                contribution: 0.7,
            },
        ],
        "core-17" => vec![
            // Lying Leg Raise
            MuscleContribution {
                muscle: "Abs",
                contribution: 1.0,
            },
        ],
        "core-18" => vec![
            // Farmer's Walk
            MuscleContribution {
                muscle: "Traps",
                contribution: 0.5,
            },
            MuscleContribution {
                muscle: "Forearms",
                contribution: 0.5,
            },
            MuscleContribution {
                muscle: "Abs",
                contribution: 0.3,
            },
        ],

        // === CARDIO (minimal muscle contributions) ===
        "cardio-01" | "cardio-02" | "cardio-03" | "cardio-05" => vec![], // Treadmill, Elliptical, Bike, Stair Climber
        "cardio-04" => vec![
            // Rowing Machine
            MuscleContribution {
                muscle: "Lats",
                contribution: 0.3,
            },
            MuscleContribution {
                muscle: "Biceps",
                contribution: 0.25,
            },
        ],
        "cardio-06" | "cardio-07" | "cardio-08" => vec![], // Jump Rope, Battle Ropes, Burpees
        "cardio-09" => vec![],                             // Air Bike
        "cardio-10" => vec![
            // Ski Erg
            MuscleContribution {
                muscle: "Lats",
                contribution: 0.3,
            },
            MuscleContribution {
                muscle: "Abs",
                contribution: 0.25,
            },
        ],
        "cardio-11" => vec![
            // Kettlebell Swing
            MuscleContribution {
                muscle: "Glutes",
                contribution: 0.7,
            },
            MuscleContribution {
                muscle: "Hamstrings",
                contribution: 0.5,
            },
            MuscleContribution {
                muscle: "Abs",
                contribution: 0.25,
            },
        ],

        _ => vec![],
    }
}

/// Parse muscle contributions from custom exercise muscle_groups strings.
/// Supports `:primary` (1.0), `:secondary` (0.5), `:tertiary` (0.25) suffixes.
/// No suffix defaults to primary (1.0) for backward compatibility.
pub fn parse_custom_muscles(muscle_groups: &[String]) -> Vec<MuscleContribution> {
    // We need to return owned data but MuscleContribution uses &'static str.
    // Instead, we'll return a Vec of (String, f64) and convert.
    // Actually, since MuscleContribution uses &'static str, for custom exercises
    // we'll match against known muscle names and use their static references.
    let mut result = Vec::new();
    for mg in muscle_groups {
        let (name, contribution) = if let Some(base) = mg.strip_suffix(":secondary") {
            (base.trim(), 0.5)
        } else if let Some(base) = mg.strip_suffix(":tertiary") {
            (base.trim(), 0.25)
        } else if let Some(base) = mg.strip_suffix(":primary") {
            (base.trim(), 1.0)
        } else {
            (mg.trim(), 1.0)
        };

        // Match against tracked muscles
        if let Some(&muscle) = TRACKED_MUSCLES
            .iter()
            .find(|&&m| m.eq_ignore_ascii_case(name))
        {
            result.push(MuscleContribution {
                muscle,
                contribution,
            });
        }
    }
    result
}

/// Compute effective sets per muscle group from completed sets in a workout exercise.
/// Returns a map of muscle name -> effective sets.
pub fn effective_sets_for_exercise(
    exercise_id: &str,
    completed_sets: usize,
    custom_muscle_groups: Option<&[String]>,
) -> HashMap<&'static str, f64> {
    let mut result = HashMap::new();

    let contributions = if let Some(mg) = custom_muscle_groups {
        // Custom exercise: try parsed muscles first, fall back to built-in
        let parsed = parse_custom_muscles(mg);
        if parsed.is_empty() {
            exercise_muscles(exercise_id)
        } else {
            parsed
        }
    } else {
        exercise_muscles(exercise_id)
    };

    for mc in &contributions {
        *result.entry(mc.muscle).or_insert(0.0) += completed_sets as f64 * mc.contribution;
    }
    result
}
