use chrono::{Datelike, NaiveDate};
use std::collections::HashMap;
use yew::prelude::*;

use crate::components::charts::{BarChart, HorizontalBarChart, LineChart, StatCard};
use crate::data::default_exercises;
use crate::models::{Category, Exercise, UnitSystem, Workout, WorkoutExercise};
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
    w.total_volume()
}

fn exercise_volume(we: &WorkoutExercise) -> f64 {
    we.volume()
}

fn exercise_max_weight(we: &WorkoutExercise) -> f64 {
    we.sets
        .iter()
        .filter(|s| s.completed && s.reps > 0)
        .map(|s| s.weight)
        .fold(0.0_f64, f64::max)
}

/// Epley formula for estimated 1RM
fn estimate_1rm(weight: f64, reps: u32) -> f64 {
    weight * (1.0 + reps as f64 / 30.0)
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
    let latest = workouts.iter().filter_map(|w| parse_date(&w.date)).max();

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

use crate::models::{best_streak, current_streak};

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
                let entry = best
                    .entry(we.exercise_id.clone())
                    .or_insert((0.0, String::new()));
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

    records.sort_by(|a, b| {
        b.date.cmp(&a.date).then(
            b.max_weight
                .partial_cmp(&a.max_weight)
                .unwrap_or(std::cmp::Ordering::Equal),
        )
    });
    records.truncate(10);
    records
}

// ── Volume per category per week ─────────────────────────────────────────────

fn volume_per_category_per_week(
    workouts: &[Workout],
    exercises: &[Exercise],
    weeks: &[((i32, u32), String)],
) -> Vec<(Category, Vec<(String, f64)>)> {
    let mut cat_week_vol: HashMap<String, HashMap<(i32, u32), f64>> = HashMap::new();

    for w in workouts {
        if let Some(d) = parse_date(&w.date) {
            let wk = iso_week_key(d);
            for we in &w.exercises {
                if let Some(ex) = find_exercise(exercises, &we.exercise_id) {
                    let vol = exercise_volume(we);
                    *cat_week_vol
                        .entry(ex.category.to_string())
                        .or_default()
                        .entry(wk)
                        .or_default() += vol;
                }
            }
        }
    }

    // Get top 4 categories by total volume
    let mut cat_totals: Vec<(String, f64)> = cat_week_vol
        .iter()
        .map(|(cat, weeks_map)| (cat.clone(), weeks_map.values().sum::<f64>()))
        .collect();
    cat_totals.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    cat_totals.truncate(4);

    cat_totals
        .into_iter()
        .filter_map(|(cat_name, _)| {
            let cat = Category::all()
                .into_iter()
                .find(|c| c.to_string() == cat_name)?;
            let week_data = cat_week_vol.get(&cat_name)?;
            let points: Vec<(String, f64)> = weeks
                .iter()
                .map(|(key, label)| (label.clone(), week_data.get(key).copied().unwrap_or(0.0)))
                .collect();
            Some((cat, points))
        })
        .collect()
}

// ── Days since last training per category ────────────────────────────────────

fn days_since_category(workouts: &[Workout], exercises: &[Exercise]) -> Vec<(Category, i64)> {
    let today = chrono::Local::now().date_naive();
    let mut last_trained: HashMap<String, NaiveDate> = HashMap::new();

    for w in workouts {
        if let Some(d) = parse_date(&w.date) {
            for we in &w.exercises {
                if let Some(ex) = find_exercise(exercises, &we.exercise_id) {
                    let cat = ex.category.to_string();
                    let entry = last_trained.entry(cat).or_insert(d);
                    if d > *entry {
                        *entry = d;
                    }
                }
            }
        }
    }

    Category::all()
        .into_iter()
        .filter_map(|cat| {
            last_trained
                .get(&cat.to_string())
                .map(|d| (cat, (today - *d).num_days()))
        })
        .collect()
}

// ── Milestone badges ─────────────────────────────────────────────────────────

const MILESTONES: &[(u32, &str)] = &[
    (1, "\u{1f3c6}"),   // 🏆
    (5, "\u{2b50}"),    // ⭐
    (10, "\u{1f4aa}"),  // 💪
    (25, "\u{1f525}"),  // 🔥
    (50, "\u{1f48e}"),  // 💎
    (100, "\u{1f451}"), // 👑
    (250, "\u{26a1}"),  // ⚡
    (500, "\u{1f680}"), // 🚀
];

// ── Analytics Page ──────────────────────────────────────────────────────────

#[function_component(AnalyticsPage)]
pub fn analytics_page() -> Html {
    let workouts = use_state(storage::load_workouts);
    let routines = use_state(storage::load_routines);
    let exercises = use_memo((), |_| all_exercises());
    let units = use_memo((), |_| storage::load_user_config().unit_system);
    let active_tab = use_state(|| 0u8);

    let tab_click = |tab: u8| {
        let active_tab = active_tab.clone();
        Callback::from(move |_: MouseEvent| active_tab.set(tab))
    };

    let tab_class = |tab: u8| -> String {
        if *active_tab == tab {
            "flex-1 py-2 text-center text-sm font-bold text-blue-600 dark:text-blue-400 border-b-2 border-blue-600 dark:border-blue-400 transition-colors".to_string()
        } else {
            "flex-1 py-2 text-center text-sm font-semibold text-gray-500 dark:text-gray-500 border-b-2 border-transparent hover:text-gray-700 dark:hover:text-gray-300 transition-colors".to_string()
        }
    };

    html! {
        <div class="px-4 py-4 space-y-4">
            <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">{"Analytics"}</h1>

            <div class="flex border-b border-gray-200 dark:border-gray-700">
                <button class={tab_class(0)} onclick={tab_click(0)}>{"Overview"}</button>
                <button class={tab_class(1)} onclick={tab_click(1)}>{"Progress"}</button>
                <button class={tab_class(2)} onclick={tab_click(2)}>{"Body"}</button>
            </div>

            { match *active_tab {
                0 => html! { <OverviewTab workouts={(*workouts).clone()} exercises={(*exercises).clone()} units={(*units).clone()} /> },
                1 => html! { <ProgressTab workouts={(*workouts).clone()} exercises={(*exercises).clone()} routines={(*routines).clone()} units={(*units).clone()} /> },
                _ => html! { <BodyTab /> },
            }}
        </div>
    }
}

// ── Overview Tab ────────────────────────────────────────────────────────────

#[derive(Properties, PartialEq)]
struct OverviewProps {
    workouts: Vec<Workout>,
    exercises: Vec<Exercise>,
    #[prop_or_default]
    units: UnitSystem,
}

#[function_component(OverviewTab)]
fn overview_tab(props: &OverviewProps) -> Html {
    let workouts = &props.workouts;
    let exercises = &props.exercises;
    let show_volume_cats = use_state(|| false);

    if workouts.is_empty() {
        return html! {
            <div class="text-center py-12 bg-gray-50 dark:bg-gray-800/20 rounded-2xl border border-dashed border-gray-200 dark:border-gray-700 transition-colors">
                <p class="text-4xl mb-4">{"📊"}</p>
                <p class="text-lg font-bold text-gray-900 dark:text-gray-100">{"No workouts yet"}</p>
                <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">{"Complete your first workout to see analytics here."}</p>
            </div>
        };
    }

    // ── Stats
    let total_workouts = workouts.len();
    let total_volume: f64 = workouts.iter().map(workout_volume).sum();
    let streak = current_streak(workouts);
    let best = best_streak(workouts);
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

    // ── Workouts per week (bar chart)
    let weeks = last_n_weeks(workouts, 8);
    let mut week_counts: HashMap<(i32, u32), f64> = HashMap::new();
    for w in workouts {
        if let Some(d) = parse_date(&w.date) {
            *week_counts.entry(iso_week_key(d)).or_default() += 1.0;
        }
    }
    let workouts_per_week: Vec<(String, f64)> = weeks
        .iter()
        .map(|(key, label)| (label.clone(), *week_counts.get(key).unwrap_or(&0.0)))
        .collect();

    // ── Volume per week (line chart)
    let mut week_volume: HashMap<(i32, u32), f64> = HashMap::new();
    for w in workouts {
        if let Some(d) = parse_date(&w.date) {
            *week_volume.entry(iso_week_key(d)).or_default() += workout_volume(w);
        }
    }
    let volume_per_week: Vec<(String, f64)> = weeks
        .iter()
        .map(|(key, label)| (label.clone(), *week_volume.get(key).unwrap_or(&0.0)))
        .collect();

    // ── Muscle group distribution
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

    // ── Personal Records
    let prs = personal_records(workouts, exercises);

    // ── Milestone badges
    let total = total_workouts as u32;
    let next_milestone = MILESTONES.iter().find(|(threshold, _)| *threshold > total);

    // ── Volume per category per week
    let vol_cat_data = volume_per_category_per_week(workouts, exercises, &weeks);

    // ── Training frequency warnings
    let days_since = days_since_category(workouts, exercises);
    let stale_cats: Vec<&(Category, i64)> =
        days_since.iter().filter(|(_, days)| *days > 7).collect();

    html! {
        <div class="space-y-6">
            // Stat cards
            <div class="grid grid-cols-2 gap-3">
                <StatCard label="Total Workouts" value={format!("{}", total_workouts)} icon="\u{1f3cb}" />
                <StatCard label={format!("Total Volume ({})", props.units.weight_label())} value={volume_display} icon="\u{1f4aa}" />
                <StatCard label="Day Streak" value={format!("{}", streak)} icon="\u{1f525}" />
                <StatCard label="Best Streak" value={format!("{}d", best)} icon="\u{1f3c6}" />
                <StatCard label="Avg Duration" value={format!("{}m", avg_duration)} icon="\u{23f1}" />
            </div>

            // Milestone badges
            <div class="bg-gray-100 dark:bg-gray-800 rounded-xl p-4 neu-flat transition-colors">
                <h3 class="text-sm font-bold text-gray-900 dark:text-gray-100 mb-3 uppercase tracking-wider">{"Milestones"}</h3>
                <div class="flex gap-2 overflow-x-auto pb-1">
                    { for MILESTONES.iter().map(|(threshold, emoji)| {
                        let achieved = total >= *threshold;
                        html! {
                            <div class={classes!(
                                "flex-shrink-0", "flex", "flex-col", "items-center", "gap-1", "px-3", "py-2", "rounded-lg", "border",
                                if achieved {
                                    "bg-yellow-500/10 border-yellow-500/30 text-yellow-500"
                                } else {
                                    "bg-gray-200 dark:bg-gray-700 border-transparent text-gray-400 dark:text-gray-600 opacity-50"
                                }
                            )}>
                                <span class="text-lg">{emoji}</span>
                                <span class="text-[10px] font-bold">{threshold}</span>
                            </div>
                        }
                    })}
                </div>
                { if let Some((next_threshold, _)) = next_milestone {
                    let remaining = next_threshold - total;
                    html! {
                        <p class="text-xs text-gray-500 mt-2">
                            {format!("Next: {} workouts ({} to go)", next_threshold, remaining)}
                        </p>
                    }
                } else {
                    html! { <p class="text-xs text-yellow-500 mt-2 font-bold">{"All milestones achieved!"}</p> }
                }}
            </div>

            // Training frequency warnings
            { if !stale_cats.is_empty() {
                html! {
                    <div class="flex flex-wrap gap-2">
                        { for stale_cats.iter().map(|(cat, days)| {
                            let is_critical = *days >= 14;
                            html! {
                                <span class={classes!(
                                    "text-xs", "font-bold", "px-2", "py-1", "rounded-full",
                                    if is_critical {
                                        "bg-red-500/20 text-red-400"
                                    } else {
                                        "bg-yellow-500/20 text-yellow-500"
                                    }
                                )}>
                                    {format!("{}: {}d ago", cat, days)}
                                </span>
                            }
                        })}
                    </div>
                }
            } else { html! {} }}

            // Workouts per week
            <div class="bg-gray-100 dark:bg-gray-800 rounded-xl p-4 neu-flat transition-colors">
                <BarChart data={workouts_per_week} title="Workouts Per Week" height={180} color="#3b82f6" />
            </div>

            // Volume over time
            <div class="bg-gray-100 dark:bg-gray-800 rounded-xl p-4 neu-flat transition-colors">
                <LineChart data={volume_per_week} title={format!("Volume Per Week ({})", props.units.weight_label())} height={180} color="#10b981" />
            </div>

            // Volume per muscle group over time (collapsible)
            { if !vol_cat_data.is_empty() {
                let show = *show_volume_cats;
                let toggle = show_volume_cats.clone();
                html! {
                    <div class="bg-gray-100 dark:bg-gray-800 rounded-xl p-4 neu-flat transition-colors">
                        <button
                            class="w-full flex justify-between items-center text-sm font-bold text-gray-900 dark:text-gray-100 uppercase tracking-wider"
                            onclick={Callback::from(move |_| toggle.set(!show))}
                        >
                            {"Volume Per Muscle Group"}
                            <span class={classes!("text-gray-400", "transition-transform", if show { "rotate-180" } else { "" })}>{"\u{25be}"}</span>
                        </button>
                        { if show {
                            html! {
                                <div class="mt-4 space-y-4">
                                    { for vol_cat_data.iter().map(|(cat, data)| {
                                        let color = category_color(cat).to_string();
                                        html! {
                                            <LineChart data={data.clone()} title={cat.to_string()} height={140} color={color} />
                                        }
                                    })}
                                </div>
                            }
                        } else { html! {} }}
                    </div>
                }
            } else { html! {} }}

            // Muscle group distribution
            <div class="bg-gray-100 dark:bg-gray-800 rounded-xl p-4 neu-flat transition-colors">
                <HorizontalBarChart data={muscle_data} title="Muscle Group Distribution" />
            </div>

            // Personal Records
            if !prs.is_empty() {
                <div class="bg-gray-100 dark:bg-gray-800 rounded-xl p-4 neu-flat transition-colors">
                    <h3 class="text-sm font-bold text-gray-900 dark:text-gray-100 mb-3 uppercase tracking-wider">{"Personal Records"}</h3>
                    <div class="space-y-2">
                        { for prs.iter().map(|pr| {
                            html! {
                                <div class="flex justify-between items-center text-sm">
                                    <span class="text-gray-700 dark:text-gray-300 truncate mr-2">{&pr.exercise_name}</span>
                                    <div class="flex items-center gap-2 flex-shrink-0">
                                        <span class="text-yellow-600 dark:text-yellow-400 font-bold">{format!("{:.1} {}", props.units.display_weight(pr.max_weight), props.units.weight_label())}</span>
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
    #[prop_or_default]
    units: UnitSystem,
}

#[function_component(ProgressTab)]
fn progress_tab(props: &ProgressProps) -> Html {
    let workouts = &props.workouts;
    let exercises = &props.exercises;
    let routines = &props.routines;

    let selected_exercise = use_state(String::new);

    if workouts.is_empty() {
        return html! {
            <div class="text-center py-12 bg-gray-50 dark:bg-gray-800/20 rounded-2xl border border-dashed border-gray-200 dark:border-gray-700 transition-colors">
                <p class="text-4xl mb-4">{"📈"}</p>
                <p class="text-lg font-bold text-gray-900 dark:text-gray-100">{"No workouts yet"}</p>
                <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">{"Complete workouts to track your progress."}</p>
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
    let (weight_data, volume_data, e1rm_data) = if !selected_exercise.is_empty() {
        let mut weight_points: Vec<(String, f64)> = Vec::new();
        let mut volume_points: Vec<(String, f64)> = Vec::new();
        let mut e1rm_points: Vec<(String, f64)> = Vec::new();

        let mut relevant: Vec<&Workout> = workouts
            .iter()
            .filter(|w| {
                w.exercises
                    .iter()
                    .any(|we| we.exercise_id == *selected_exercise)
            })
            .collect();
        relevant.sort_by(|a, b| a.date.cmp(&b.date));

        for w in &relevant {
            for we in &w.exercises {
                if we.exercise_id == *selected_exercise {
                    let max_w = exercise_max_weight(we);
                    let vol = exercise_volume(we);
                    let label = if w.date.len() >= 10 {
                        format!("{}/{}", &w.date[5..7], &w.date[8..10])
                    } else {
                        w.date.clone()
                    };

                    // Compute max est. 1RM for this session
                    let max_e1rm = we
                        .sets
                        .iter()
                        .filter(|s| s.completed && s.weight > 0.0 && s.reps > 0)
                        .map(|s| estimate_1rm(s.weight, s.reps))
                        .fold(0.0_f64, f64::max);

                    weight_points.push((label.clone(), max_w));
                    volume_points.push((label.clone(), vol));
                    if max_e1rm > 0.0 {
                        e1rm_points.push((label, max_e1rm));
                    }
                }
            }
        }

        // Limit to last 12 sessions
        let n = weight_points.len();
        if n > 12 {
            weight_points = weight_points[n - 12..].to_vec();
            volume_points = volume_points[n - 12..].to_vec();
        }
        let n2 = e1rm_points.len();
        if n2 > 12 {
            e1rm_points = e1rm_points[n2 - 12..].to_vec();
        }

        (weight_points, volume_points, e1rm_points)
    } else {
        (vec![], vec![], vec![])
    };

    // Routine tracking
    let routine_stats: Vec<Html> = routines
        .iter()
        .map(|routine| {
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

                    let wl = props.units.weight_label();
                    let d_last = props.units.display_weight(last);
                    let d_diff = props.units.display_weight(diff);
                    Some(html! {
                        <div class="flex justify-between text-xs">
                            <span class="text-gray-400 truncate mr-2">{name}</span>
                            <span class={color}>
                                {format!("{:.1}{} ", d_last, wl)}{arrow}
                                if diff.abs() > 0.0 {
                                    <span class="text-gray-500">{format!(" ({:+.1})", d_diff)}</span>
                                }
                            </span>
                        </div>
                    })
                })
                .collect();

            html! {
                <div class="bg-gray-100 dark:bg-gray-800 rounded-xl p-4 neu-flat transition-colors">
                    <div class="flex justify-between items-start mb-2">
                        <h4 class="font-bold text-gray-900 dark:text-gray-100">{&routine.name}</h4>
                        <span class="text-xs text-gray-500 font-medium">{format!("{} sessions", count)}</span>
                    </div>
                    <div class="text-xs text-gray-500 dark:text-gray-500 mb-3 font-mono">
                        {"Last: "}{last_date}
                    </div>
                    if !exercise_trends.is_empty() {
                        <div class="space-y-1.5 border-t border-gray-200 dark:border-gray-700 pt-3">
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
            <div class="bg-gray-100 dark:bg-gray-800 rounded-xl p-4 space-y-4 neu-flat transition-colors">
                <h3 class="text-sm font-bold text-gray-900 dark:text-gray-100 uppercase tracking-wider">{"Exercise Progress"}</h3>
                <select class="w-full bg-white dark:bg-gray-700 text-gray-900 dark:text-white rounded-lg px-3 py-2 text-sm outline-none neu-pressed transition-colors"
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
                    <LineChart data={weight_data} title={format!("Max Weight Per Session ({})", props.units.weight_label())} height={180} color="#f59e0b" />
                }

                if !volume_data.is_empty() {
                    <LineChart data={volume_data} title={format!("Volume Per Session ({})", props.units.weight_label())} height={180} color="#8b5cf6" />
                }

                if !e1rm_data.is_empty() {
                    <LineChart data={e1rm_data} title={format!("Est. 1RM Per Session ({})", props.units.weight_label())} height={180} color="#ec4899" />
                }
            </div>

            // Routine tracking
            if !routine_stats.is_empty() {
                <div class="space-y-3">
                    <h3 class="text-sm font-bold text-gray-900 dark:text-gray-100 px-1 uppercase tracking-wider">{"Routine Tracking"}</h3>
                    { for routine_stats }
                </div>
            }
        </div>
    }
}

// ── Body Tab ────────────────────────────────────────────────────────────────

#[function_component(BodyTab)]
fn body_tab() -> Html {
    let metrics = storage::load_body_metrics();
    let config = storage::load_user_config();
    let units = &config.unit_system;

    if metrics.is_empty() {
        return html! {
            <div class="text-center py-12 bg-gray-50 dark:bg-gray-800/20 rounded-2xl border border-dashed border-gray-200 dark:border-gray-700 transition-colors">
                <p class="text-4xl mb-4">{"⚖️"}</p>
                <p class="text-lg font-bold text-gray-900 dark:text-gray-100">{"No body metrics yet"}</p>
                <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">{"Log your weight in Settings to see trends here."}</p>
            </div>
        };
    }

    let mut weight_data: Vec<(String, f64)> = metrics
        .iter()
        .filter_map(|m| m.weight.map(|w| (m.date[5..].to_string(), w)))
        .collect();
    weight_data.sort_by(|a, b| a.0.cmp(&b.0));

    let mut fat_data: Vec<(String, f64)> = metrics
        .iter()
        .filter_map(|m| m.body_fat.map(|f| (m.date[5..].to_string(), f)))
        .collect();
    fat_data.sort_by(|a, b| a.0.cmp(&b.0));

    let latest_weight = weight_data.last().map(|d| d.1);
    let bmi = if let (Some(w), Some(h)) = (latest_weight, config.height) {
        let h_m = h / 100.0;
        Some(w / (h_m * h_m))
    } else {
        None
    };

    html! {
        <div class="space-y-6">
            <div class="grid grid-cols-2 gap-3">
                <StatCard label="Latest Weight" value={latest_weight.map(|w| format!("{:.1}{}", units.display_weight(w), units.weight_label())).unwrap_or_else(|| "--".to_string())} icon="\u{2696}" />
                <StatCard label="BMI" value={bmi.map(|b| format!("{:.1}", b)).unwrap_or_else(|| "--".to_string())} icon="\u{1f4cf}" />
            </div>

            if !weight_data.is_empty() {
                <div class="bg-gray-100 dark:bg-gray-800 rounded-xl p-4 neu-flat transition-colors">
                    <LineChart data={weight_data} title={format!("Weight Progress ({})", units.weight_label())} height={180} color="#3b82f6" />
                </div>
            }

            if !fat_data.is_empty() {
                <div class="bg-gray-100 dark:bg-gray-800 rounded-xl p-4 neu-flat transition-colors">
                    <LineChart data={fat_data} title="Body Fat %" height={180} color="#ef4444" />
                </div>
            }
        </div>
    }
}
