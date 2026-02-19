use crate::storage;
use yew::prelude::*;

struct Badge {
    name: &'static str,
    emoji: &'static str,
    description: &'static str,
    earned: bool,
}

#[function_component(AchievementBadges)]
pub fn achievement_badges() -> Html {
    let workouts = storage::load_workouts();

    if workouts.is_empty() {
        return html! {};
    }

    let total_workouts = workouts.len();
    let total_sets: usize = workouts
        .iter()
        .map(|w| w.exercises.iter().map(|e| e.sets.len()).sum::<usize>())
        .sum();
    let max_weight: f64 = workouts
        .iter()
        .flat_map(|w| w.exercises.iter())
        .flat_map(|e| e.sets.iter())
        .map(|s| s.weight)
        .fold(0.0_f64, f64::max);

    let consecutive_streak = {
        let mut dates: Vec<chrono::NaiveDate> = workouts
            .iter()
            .filter_map(|w| chrono::NaiveDate::parse_from_str(&w.date, "%Y-%m-%d").ok())
            .collect();
        dates.sort();
        dates.dedup();

        let mut max_streak = 1u32;
        let mut current = 1u32;
        for i in 1..dates.len() {
            if (dates[i] - dates[i - 1]).num_days() == 1 {
                current += 1;
                if current > max_streak {
                    max_streak = current;
                }
            } else {
                current = 1;
            }
        }
        if dates.len() <= 1 {
            dates.len() as u32
        } else {
            max_streak
        }
    };

    let badges = vec![
        Badge {
            name: "Wooden Spoon",
            emoji: "🥄",
            description: "Complete 1st workout",
            earned: total_workouts >= 1,
        },
        Badge {
            name: "Cutting Board",
            emoji: "🔪",
            description: "Complete 10 workouts",
            earned: total_workouts >= 10,
        },
        Badge {
            name: "Snuff Box",
            emoji: "📦",
            description: "Log 100 total sets",
            earned: total_sets >= 100,
        },
        Badge {
            name: "Turned Bowl",
            emoji: "🥣",
            description: "Complete 50 workouts",
            earned: total_workouts >= 50,
        },
        Badge {
            name: "Mallet Head",
            emoji: "🔨",
            description: "Lift 100kg or more",
            earned: max_weight >= 100.0,
        },
        Badge {
            name: "Lignum Vitae",
            emoji: "🪵",
            description: "30-day streak",
            earned: consecutive_streak >= 30,
        },
        Badge {
            name: "Master Carver",
            emoji: "👑",
            description: "Complete 100 workouts",
            earned: total_workouts >= 100,
        },
    ];

    html! {
        <div class="space-y-3">
            <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100 px-1">{"Achievements"}</h2>
            <div class="grid grid-cols-4 gap-2">
                { for badges.iter().map(|b| {
                    let opacity = if b.earned { "" } else { "opacity-30 grayscale" };
                    html! {
                        <div class={format!("flex flex-col items-center p-2 bg-gray-100 dark:bg-gray-800/50 rounded-xl border border-gray-200 dark:border-gray-700/50 text-center transition-colors {}", opacity)}>
                            <span class="text-2xl">{b.emoji}</span>
                            <span class="text-[9px] font-bold text-gray-800 dark:text-gray-200 mt-1 leading-tight">{b.name}</span>
                            <span class="text-[8px] text-gray-500 dark:text-gray-500 leading-tight">{b.description}</span>
                        </div>
                    }
                })}
            </div>
        </div>
    }
}
