use yew::prelude::*;
use std::collections::HashMap;
use chrono::NaiveDate;

use crate::components::charts::{BarChart, HorizontalBarChart, LineChart, StatCard};
use crate::data::default_exercises;
use crate::models::{Category, Exercise, Workout, WorkoutExercise};
use crate::storage;

// ── Helpers ─────────────────────────────────────────────────────────────────

fn all_exercises() -> Vec<Exercise> {
    let mut exs = default_exercises();
    exs.extend(storage::load_custom_exercises());
    exs
}

fn find_exercise_name(exercises: &[Exercise], id: &str) -> String {
    exercises
        .iter()
        .find(|e| e.id == id)
        .map(|e| e.name.clone())
        .unwrap_or_else(|| id.to_string())
}

fn find_exercise<'a>(exercises: &'a [Exercise], id: &str) -> Option<&'a Exercise> {
    exercises.iter().find(|e| e.id == id)
}

fn parse_date(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

fn workout_volume(w: &Workout) -> f64 {
    w.exercises
        .iter()
        .flat_map(|we| we.sets.iter())
        .filter(|s| s.completed)
        .map(|s| s.weight * s.reps as f64)
        .sum()
}

fn exercise_volume(we: &WorkoutExercise) -> f64 {
    we.sets
        .iter()
        .filter(|s| s.completed)
        .map(|s| s.weight * s.reps as f64)
        .sum()
}

fn exercise_max_weight(we: &WorkoutExercise) -> f64 {
    we.sets
        .iter()
        .filter(|s| s.completed && s.reps > 0)
        .map(|s| s.weight)
        .fold(0.0_f64, f64::max)
}

fn iso_week_label(d: NaiveDate) -> String {
    let iso = d.iso_week();
    format!("W{}", iso.week())
}

fn iso_week_key(d: NaiveDate) -> (i32, u32) {
    let iso = d.iso_week();
    (iso.year(), iso.week())
}

fn category_color(cat: &Category) -> &'static str {
    match cat {
        Category::Chest => "#ef4444",
        Category::Back => "#3b82f6",
        Category::Legs => "#22c55e",
        Category::Shoulders => "#f59e0b",
        Category::Arms => "#a855f7",
        Category::Core => "#ec4899",
        Category::Cardio => "#06b6d4",
    }
}

/// Build ordered list of last N weeks as (year, week) keys + labels.
fn last_n_weeks(workouts: &[Workout], n: usize) -> Vec<((i32, u32), String)> {
    // Find the latest date across all workouts
    let latest = workouts
        .iter()
        .filter_map(|w| parse_date(&w.date))
        .max();

    let latest = match latest {
        Some(d) => d,
        None => return vec![],
    };

    let mut weeks = Vec::with_capacity(n);
    for i in (0..n).rev() {
        let d = latest - chrono::Duration::weeks(i as i64);
        let key = iso_week_key(d);
        let label = iso_week_label(d);
        weeks.push((key, label));
    }
    weeks
}

// ── Streak calculation ──────────────────────────────────────────────────────

fn current_streak(workouts: &[Workout]) -> u32 {
    if workouts.is_empty() {
        return 0;
    }
    let mut dates: Vec<NaiveDate> = workouts
        .iter()
        .filter_map(|w| parse_date(&w.date))
        .collect();
    dates.sort();
    dates.dedup();
    if dates.is_empty() {
        return 0;
    }

    let today = chrono::Local::now().date_naive();
    // Start from today or the last workout date (whichever is earlier)
    let last = *dates.last().unwrap();
    // If last workout is more than 1 day ago, streak is 0
    if (today - last).num_days() > 1 {
        return 0;
    }

    let mut streak = 1u32;
    for i in (0..dates.len() - 1).rev() {
        if (dates[i + 1] - dates[i]).num_days() == 1 {
            streak += 1;
        } else {
            break;
        }
    }
    streak
}

// ── Personal Records ────────────────────────────────────────────────────────

struct PersonalRecord {
    exercise_name: String,
    max_weight: f64,
    date: String,
}

fn personal_records(workouts: &[Workout], exercises: &[Exercise]) -> Vec<PersonalRecord> {
    let mut best: HashMap<String, (f64, String)> = HashMap::new();

    for w in workouts {
        for we in &w.exercises {
            let max_w = exercise_max_weight(we);
            if max_w > 0.0 {
                let entry = best.entry(we.exercise_id.clone()).or_insert((0.0, String::new()));
                if max_w > entry.0 {
                    *entry = (max_w, w.date.clone());
                }
            }
        }
    }

    let mut records: Vec<PersonalRecord> = best
        .into_iter()
        .map(|(id, (weight, date))| PersonalRecord {
            exercise_name: find_exercise_name(exercises, &id),
            max_weight: weight,
            date,
        })
        .collect();

    records.sort_by(|a, b| b.date.cmp(&a.date).then(b.max_weight.partial_cmp(&a.max_weight).unwrap_or(std::cmp::Ordering::Equal)));
    records.truncate(10);
    records
}

// ── Analytics Page ──────────────────────────────────────────────────────────

#[function_component(AnalyticsPage)]
pub fn analytics_page() -> Html {
    let workouts = use_state(|| storage::load_workouts());
    let routines = use_state(|| storage::load_routines());
    let exercises = use_memo((), |_| all_exercises());
    let active_tab = use_state(|| 0u8); // 0 = Overview, 1 = Progress

    let tab_click = |tab: u8| {
        let active_tab = active_tab.clone();
        Callback::from(move |_: MouseEvent| active_tab.set(tab))
    };

    let tab_class = |tab: u8| -> &'static str {
        if *active_tab == tab {
            "flex-1 py-2 text-center text-sm font-semibold text-blue-400 border-b-2 border-blue-400"
        } else {
            "flex-1 py-2 text-center text-sm font-semibold text-gray-500 border-b-2 border-transparent hover:text-gray-300"
        }
    };

    html! {
        <div class="px-4 py-4 space-y-4">
            <h1 class="text-2xl font-bold">{"Analytics"}</h1>

            // Tab bar
            <div class="flex border-b border-gray-700">
                <button class={tab_class(0)} onclick={tab_click(0)}>{"Overview"}</button>
                <button class={tab_class(1)} onclick={tab_click(1)}>{"Progress"}</button>
            </div>

            if *active_tab == 0 {
                <OverviewTab workouts={(*workouts).clone()} exercises={(*exercises).clone()} />
            } else {
                <ProgressTab workouts={(*workouts).clone()} exercises={(*exercises).clone()}
                             routines={(*routines).clone()} />
            }
        </div>
    }
}

// ── Overview Tab ────────────────────────────────────────────────────────────

#[derive(Properties, PartialEq)]
struct OverviewProps {
    workouts: Vec<Workout>,
    exercises: Vec<Exercise>,
}

#[function_component(OverviewTab)]
fn overview_tab(props: &OverviewProps) -> Html {
    let workouts = &props.workouts;
    let exercises = &props.exercises;

    if workouts.is_empty() {
        return html! {
            <div class="text-center text-gray-500 py-12">
                <p class="text-4xl mb-4">{"📊"}</p>
                <p class="text-lg">{"No workouts yet"}</p>
                <p class="text-sm mt-1">{"Complete your first workout to see analytics here."}</p>
            </div>
        };
    }

    // ── Stats ───────────────────────────────────────────────────────────
    let total_workouts = workouts.len();
    let total_volume: f64 = workouts.iter().map(|w| workout_volume(w)).sum();
    let streak = current_streak(workouts);
    let avg_duration: u32 = if total_workouts > 0 {
        let total_dur: u32 = workouts.iter().map(|w| w.duration_mins).sum();
        total_dur / total_workouts as u32
    } else {
        0
    };

    let volume_display = if total_volume >= 1_000_000.0 {
        format!("{:.1}M", total_volume / 1_000_000.0)
    } else if total_volume >= 1000.0 {
        format!("{:.0}k", total_volume / 1000.0)
    } else {
        format!("{:.0}", total_volume)
    };

    // ── Workouts per week (bar chart) ───────────────────────────────────
    let weeks = last_n_weeks(workouts, 8);
    let mut week_counts: HashMap<(i32, u32), f64> = HashMap::new();
    for w in workouts {
        if let Some(d) = parse_date(&w.date) {
            *week_counts.entry(iso_week_key(d)).or_default() += 1.0;
        }
    }
    let workouts_per_week: Vec<(String, f64)> = weeks
        .iter()
        .map(|(key, label)| {
            (label.clone(), *week_counts.get(key).unwrap_or(&0.0))
        })
        .collect();

    // ── Volume per week (line chart) ────────────────────────────────────
    let mut week_volume: HashMap<(i32, u32), f64> = HashMap::new();
    for w in workouts {
        if let Some(d) = parse_date(&w.date) {
            *week_volume.entry(iso_week_key(d)).or_default() += workout_volume(w);
        }
    }
    let volume_per_week: Vec<(String, f64)> = weeks
        .iter()
        .map(|(key, label)| {
            (label.clone(), *week_volume.get(key).unwrap_or(&0.0))
        })
        .collect();

    // ── Muscle group distribution ───────────────────────────────────────
    let mut cat_counts: HashMap<String, f64> = HashMap::new();
    for w in workouts {
        for we in &w.exercises {
            if let Some(ex) = find_exercise(exercises, &we.exercise_id) {
                *cat_counts.entry(ex.category.to_string()).or_default() += 1.0;
            }
        }
    }
    let muscle_data: Vec<(String, f64, String)> = Category::all()
        .iter()
        .filter_map(|cat| {
            let count = cat_counts.get(&cat.to_string()).copied().unwrap_or(0.0);
            if count > 0.0 {
                Some((cat.to_string(), count, category_color(cat).to_string()))
            } else {
                None
            }
        })
        .collect();

    // ── Personal Records ────────────────────────────────────────────────
    let prs = personal_records(workouts, exercises);

    html! {
        <div class="space-y-6">
            // Stat cards
            <div class="grid grid-cols-2 gap-3">
                <StatCard label="Total Workouts" value={format!("{}", total_workouts)} icon="\u{1f3cb}" />
                <StatCard label="Total Volume (kg)" value={volume_display} icon="\u{1f4aa}" />
                <StatCard label="Day Streak" value={format!("{}", streak)} icon="\u{1f525}" />
                <StatCard label="Avg Duration" value={format!("{}m", avg_duration)} icon="\u{23f1}" />
            </div>

            // Workouts per week
            <div class="bg-gray-800 rounded-xl p-4">
                <BarChart data={workouts_per_week} title="Workouts Per Week" height={180} color="#3b82f6" />
            </div>

            // Volume over time
            <div class="bg-gray-800 rounded-xl p-4">
                <LineChart data={volume_per_week} title="Volume Per Week (kg)" height={180} color="#10b981" />
            </div>

            // Muscle group distribution
            <div class="bg-gray-800 rounded-xl p-4">
                <HorizontalBarChart data={muscle_data} title="Muscle Group Distribution" />
            </div>

            // Personal Records
            if !prs.is_empty() {
                <div class="bg-gray-800 rounded-xl p-4">
                    <h3 class="text-sm font-semibold text-gray-300 mb-3">{"Personal Records"}</h3>
                    <div class="space-y-2">
                        { for prs.iter().map(|pr| {
                            html! {
                                <div class="flex justify-between items-center text-sm">
                                    <span class="text-gray-300 truncate mr-2">{&pr.exercise_name}</span>
                                    <div class="flex items-center gap-2 flex-shrink-0">
                                        <span class="text-yellow-400 font-bold">{format!("{:.1} kg", pr.max_weight)}</span>
                                        <span class="text-gray-500 text-xs">{&pr.date}</span>
                                    </div>
                                </div>
                            }
                        })}
                    </div>
                </div>
            }
        </div>
    }
}

// ── Progress Tab ────────────────────────────────────────────────────────────

#[derive(Properties, PartialEq)]
struct ProgressProps {
    workouts: Vec<Workout>,
    exercises: Vec<Exercise>,
    routines: Vec<crate::models::Routine>,
}

#[function_component(ProgressTab)]
fn progress_tab(props: &ProgressProps) -> Html {
    let workouts = &props.workouts;
    let exercises = &props.exercises;
    let routines = &props.routines;

    let selected_exercise = use_state(|| String::new());

    if workouts.is_empty() {
        return html! {
            <div class="text-center text-gray-500 py-12">
                <p class="text-4xl mb-4">{"📈"}</p>
                <p class="text-lg">{"No workouts yet"}</p>
                <p class="text-sm mt-1">{"Complete workouts to track your progress."}</p>
            </div>
        };
    }

    // Build list of exercises that appear in workouts
    let mut seen_ids: Vec<String> = Vec::new();
    for w in workouts {
        for we in &w.exercises {
            if !seen_ids.contains(&we.exercise_id) {
                seen_ids.push(we.exercise_id.clone());
            }
        }
    }

    // Auto-select first if none selected
    if selected_exercise.is_empty() && !seen_ids.is_empty() {
        selected_exercise.set(seen_ids[0].clone());
    }

    let on_select = {
        let selected_exercise = selected_exercise.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlSelectElement = e.target_unchecked_into();
            selected_exercise.set(input.value());
        })
    };

    // Build data for selected exercise
    let (weight_data, volume_data) = if !selected_exercise.is_empty() {
        let mut weight_points: Vec<(String, f64)> = Vec::new();
        let mut volume_points: Vec<(String, f64)> = Vec::new();

        let mut relevant: Vec<&Workout> = workouts
            .iter()
            .filter(|w| w.exercises.iter().any(|we| we.exercise_id == *selected_exercise))
            .collect();
        relevant.sort_by(|a, b| a.date.cmp(&b.date));

        for w in &relevant {
            for we in &w.exercises {
                if we.exercise_id == *selected_exercise {
                    let max_w = exercise_max_weight(we);
                    let vol = exercise_volume(we);
                    // Use short date label (MM/DD)
                    let label = if w.date.len() >= 10 {
                        format!("{}/{}", &w.date[5..7], &w.date[8..10])
                    } else {
                        w.date.clone()
                    };
                    weight_points.push((label.clone(), max_w));
                    volume_points.push((label, vol));
                }
            }
        }

        // Limit to last 12 sessions
        let n = weight_points.len();
        if n > 12 {
            weight_points = weight_points[n - 12..].to_vec();
            volume_points = volume_points[n - 12..].to_vec();
        }

        (weight_points, volume_points)
    } else {
        (vec![], vec![])
    };

    // Routine tracking
    let routine_stats: Vec<Html> = routines
        .iter()
        .map(|routine| {
            // Count workouts that contain at least one exercise from this routine
            let matching_workouts: Vec<&Workout> = workouts
                .iter()
                .filter(|w| {
                    w.exercises
                        .iter()
                        .any(|we| routine.exercise_ids.contains(&we.exercise_id))
                })
                .collect();

            let count = matching_workouts.len();
            let last_date = matching_workouts
                .iter()
                .map(|w| w.date.as_str())
                .max()
                .unwrap_or("-");

            // Per-exercise weight trend (latest vs first)
            let exercise_trends: Vec<Html> = routine
                .exercise_ids
                .iter()
                .filter_map(|eid| {
                    let name = find_exercise_name(exercises, eid);
                    let mut sessions: Vec<(&str, f64)> = Vec::new();
                    let mut sorted_workouts: Vec<&Workout> = workouts.iter().collect();
                    sorted_workouts.sort_by(|a, b| a.date.cmp(&b.date));

                    for w in &sorted_workouts {
                        for we in &w.exercises {
                            if we.exercise_id == *eid {
                                let max_w = exercise_max_weight(we);
                                if max_w > 0.0 {
                                    sessions.push((&w.date, max_w));
                                }
                            }
                        }
                    }

                    if sessions.is_empty() {
                        return None;
                    }

                    let first = sessions.first().unwrap().1;
                    let last = sessions.last().unwrap().1;
                    let diff = last - first;
                    let (arrow, color) = if diff > 0.0 {
                        ("\u{2191}", "text-green-400")
                    } else if diff < 0.0 {
                        ("\u{2193}", "text-red-400")
                    } else {
                        ("\u{2192}", "text-gray-400")
                    };

                    Some(html! {
                        <div class="flex justify-between text-xs">
                            <span class="text-gray-400 truncate mr-2">{name}</span>
                            <span class={color}>
                                {format!("{:.1}kg ", last)}{arrow}
                                if diff.abs() > 0.0 {
                                    <span class="text-gray-500">{format!(" ({:+.1})", diff)}</span>
                                }
                            </span>
                        </div>
                    })
                })
                .collect();

            html! {
                <div class="bg-gray-800 rounded-xl p-4">
                    <div class="flex justify-between items-start mb-2">
                        <h4 class="font-medium text-gray-200">{&routine.name}</h4>
                        <span class="text-xs text-gray-500">{format!("{} sessions", count)}</span>
                    </div>
                    <div class="text-xs text-gray-500 mb-3">
                        {"Last: "}{last_date}
                    </div>
                    if !exercise_trends.is_empty() {
                        <div class="space-y-1">
                            { for exercise_trends }
                        </div>
                    }
                </div>
            }
        })
        .collect();

    html! {
        <div class="space-y-6">
            // Exercise progress selector
            <div class="bg-gray-800 rounded-xl p-4 space-y-4">
                <h3 class="text-sm font-semibold text-gray-300">{"Exercise Progress"}</h3>
                <select class="w-full bg-gray-700 text-white rounded-lg px-3 py-2 text-sm"
                        onchange={on_select}>
                    { for seen_ids.iter().map(|id| {
                        let name = find_exercise_name(exercises, id);
                        let selected = *selected_exercise == *id;
                        html! {
                            <option value={id.clone()} selected={selected}>{name}</option>
                        }
                    })}
                </select>

                if !weight_data.is_empty() {
                    <LineChart data={weight_data} title="Max Weight Per Session (kg)" height={180} color="#f59e0b" />
                }

                if !volume_data.is_empty() {
                    <LineChart data={volume_data} title="Volume Per Session (kg)" height={180} color="#8b5cf6" />
                }
            </div>

            // Routine tracking
            if !routine_stats.is_empty() {
                <div class="space-y-3">
                    <h3 class="text-sm font-semibold text-gray-300">{"Routine Tracking"}</h3>
                    { for routine_stats }
                </div>
            }
        </div>
    }
}
