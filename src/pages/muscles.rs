use std::collections::HashMap;

use chrono::NaiveDate;
use yew::prelude::*;

use crate::data::default_exercises;
use crate::models::{Exercise, Workout};
use crate::muscle_data::{
    self, effective_sets_for_exercise, CORE_MUSCLES, LEG_MUSCLES, PULL_MUSCLES, PUSH_MUSCLES,
    TRACKED_MUSCLES,
};
use crate::storage;

// ── Helpers ──────────────────────────────────────────────────────────────────

fn all_exercises() -> Vec<Exercise> {
    let mut exs = default_exercises();
    exs.extend(storage::load_custom_exercises());
    exs
}

fn find_exercise<'a>(exercises: &'a [Exercise], id: &str) -> Option<&'a Exercise> {
    exercises.iter().find(|e| e.id == id)
}

fn parse_date(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

fn today() -> NaiveDate {
    chrono::Local::now().date_naive()
}

/// Epley formula for estimated 1RM.
fn estimate_1rm(weight: f64, reps: u32) -> f64 {
    if reps <= 1 {
        weight
    } else {
        weight * (1.0 + reps as f64 / 30.0)
    }
}

/// Get thresholds: user overrides merged with defaults.
fn get_thresholds() -> HashMap<String, (f64, f64)> {
    let defaults = muscle_data::default_thresholds();
    let config = storage::load_user_config();
    let mut result: HashMap<String, (f64, f64)> =
        defaults.iter().map(|(k, v)| (k.to_string(), *v)).collect();
    if let Some(custom) = config.muscle_thresholds {
        for (k, v) in custom {
            result.insert(k, v);
        }
    }
    result
}

/// Compute effective sets per muscle over a date range from workouts.
fn compute_muscle_sets(
    workouts: &[Workout],
    exercises: &[Exercise],
    from: NaiveDate,
    to: NaiveDate,
) -> HashMap<String, f64> {
    let mut sets: HashMap<String, f64> = HashMap::new();

    for w in workouts {
        if let Some(d) = parse_date(&w.date) {
            if d >= from && d <= to {
                for we in &w.exercises {
                    let completed = we.sets.iter().filter(|s| s.completed).count();
                    if completed == 0 {
                        continue;
                    }

                    let custom_mg = find_exercise(exercises, &we.exercise_id)
                        .filter(|e| e.is_custom)
                        .map(|e| e.muscle_groups.as_slice());

                    let eff = effective_sets_for_exercise(&we.exercise_id, completed, custom_mg);
                    for (muscle, val) in eff {
                        *sets.entry(muscle.to_string()).or_default() += val;
                    }
                }
            }
        }
    }
    sets
}

/// Compute per-session muscle sets for session volume warnings.
fn session_muscle_sets(workout: &Workout, exercises: &[Exercise]) -> HashMap<String, f64> {
    let mut sets: HashMap<String, f64> = HashMap::new();
    for we in &workout.exercises {
        let completed = we.sets.iter().filter(|s| s.completed).count();
        if completed == 0 {
            continue;
        }
        let custom_mg = find_exercise(exercises, &we.exercise_id)
            .filter(|e| e.is_custom)
            .map(|e| e.muscle_groups.as_slice());
        let eff = effective_sets_for_exercise(&we.exercise_id, completed, custom_mg);
        for (muscle, val) in eff {
            *sets.entry(muscle.to_string()).or_default() += val;
        }
    }
    sets
}

// ── Color helpers ────────────────────────────────────────────────────────────

fn volume_bar_color(sets: f64, mev: f64, mrv: f64) -> &'static str {
    if sets <= 0.0 {
        "bg-gray-300 dark:bg-gray-600"
    } else if sets < mev {
        "bg-yellow-400 dark:bg-yellow-500"
    } else if sets <= mrv {
        "bg-green-500 dark:bg-green-400"
    } else {
        "bg-red-500 dark:bg-red-400"
    }
}

fn volume_text_color(sets: f64, mev: f64, mrv: f64) -> &'static str {
    if sets <= 0.0 {
        "text-gray-400"
    } else if sets < mev {
        "text-yellow-500"
    } else if sets <= mrv {
        "text-green-500"
    } else {
        "text-red-500"
    }
}

fn frequency_color(freq: f64) -> &'static str {
    if freq >= 2.0 {
        "text-green-500"
    } else if freq >= 1.0 {
        "text-yellow-500"
    } else {
        "text-red-500"
    }
}

fn recovery_color(hours: f64) -> &'static str {
    if hours < 24.0 {
        "text-red-500"
    } else if hours < 48.0 {
        "text-yellow-500"
    } else {
        "text-green-500"
    }
}

// ── Main Page ────────────────────────────────────────────────────────────────

#[function_component(MusclesPage)]
pub fn muscles_page() -> Html {
    let workouts = use_memo((), |_| storage::load_workouts());
    let exercises = use_memo((), |_| all_exercises());
    let show_thresholds = use_state(|| false);

    html! {
        <div class="px-4 py-4 space-y-6">
            <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">{"Training Intelligence"}</h1>

            <SectionMuscleBalance workouts={(*workouts).clone()} exercises={(*exercises).clone()} show_modal={show_thresholds.clone()} />
            <SectionFrequency workouts={(*workouts).clone()} exercises={(*exercises).clone()} />
            <SectionOverload workouts={(*workouts).clone()} exercises={(*exercises).clone()} />
            <SectionRepRange workouts={(*workouts).clone()} />
            <SectionDeload workouts={(*workouts).clone()} />
            <SectionPushPull workouts={(*workouts).clone()} exercises={(*exercises).clone()} />
            <SectionSessionVolume workouts={(*workouts).clone()} exercises={(*exercises).clone()} />
            <SectionRecovery workouts={(*workouts).clone()} exercises={(*exercises).clone()} />

            <ThresholdModal visible={show_thresholds} />
        </div>
    }
}

// ── Section A: Muscle Balance ────────────────────────────────────────────────

#[derive(Properties, PartialEq)]
struct MuscleBalanceProps {
    workouts: Vec<Workout>,
    exercises: Vec<Exercise>,
    show_modal: UseStateHandle<bool>,
}

#[function_component(SectionMuscleBalance)]
fn section_muscle_balance(props: &MuscleBalanceProps) -> Html {
    let t = today();
    let from = t - chrono::Duration::days(7);
    let sets = compute_muscle_sets(&props.workouts, &props.exercises, from, t);
    let thresholds = get_thresholds();

    let max_mrv = thresholds
        .values()
        .map(|(_, mrv)| *mrv)
        .fold(0.0_f64, f64::max);
    let bar_max = if max_mrv > 0.0 { max_mrv } else { 30.0 };

    let render_group = |title: &str, muscles: &[&str]| {
        html! {
            <div class="space-y-2">
                <h4 class="text-xs font-bold text-gray-500 dark:text-gray-400 uppercase tracking-wider">{title}</h4>
                { for muscles.iter().map(|&muscle| {
                    let s = sets.get(muscle).copied().unwrap_or(0.0);
                    let (mev, mrv) = thresholds.get(muscle).copied().unwrap_or((0.0, 20.0));
                    let pct = (s / bar_max * 100.0).min(100.0);
                    let color = volume_bar_color(s, mev, mrv);
                    let text_color = volume_text_color(s, mev, mrv);
                    html! {
                        <div class="flex items-center gap-2">
                            <span class="w-24 text-xs text-gray-700 dark:text-gray-300 truncate">{muscle}</span>
                            <div class="flex-1 h-4 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden relative">
                                <div class={classes!("h-full", "rounded-full", "transition-all", color)}
                                     style={format!("width: {}%", pct)} />
                                // MEV marker
                                if mev > 0.0 {
                                    <div class="absolute top-0 bottom-0 w-px bg-gray-400 dark:bg-gray-500 opacity-60"
                                         style={format!("left: {}%", (mev / bar_max * 100.0).min(100.0))} />
                                }
                                // MRV marker
                                <div class="absolute top-0 bottom-0 w-px bg-gray-500 dark:bg-gray-400 opacity-60"
                                     style={format!("left: {}%", (mrv / bar_max * 100.0).min(100.0))} />
                            </div>
                            <span class={classes!("w-10", "text-right", "text-xs", "font-bold", text_color)}>
                                {format!("{:.0}", s)}
                            </span>
                        </div>
                    }
                })}
            </div>
        }
    };

    let toggle_modal = {
        let show = props.show_modal.clone();
        Callback::from(move |_: MouseEvent| show.set(true))
    };

    if props.workouts.is_empty() {
        return html! {
            <div class="bg-gray-100 dark:bg-gray-800 rounded-xl p-4 neu-flat transition-colors">
                <h3 class="text-sm font-bold text-gray-900 dark:text-gray-100 uppercase tracking-wider mb-3">{"Muscle Balance (7 days)"}</h3>
                <p class="text-sm text-gray-500 dark:text-gray-400 text-center py-4">{"Log workouts to see your muscle balance."}</p>
            </div>
        };
    }

    html! {
        <div class="bg-gray-100 dark:bg-gray-800 rounded-xl p-4 neu-flat transition-colors space-y-4">
            <div class="flex justify-between items-center">
                <h3 class="text-sm font-bold text-gray-900 dark:text-gray-100 uppercase tracking-wider">{"Muscle Balance (7 days)"}</h3>
                <button onclick={toggle_modal} class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors" title="Customize thresholds">
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                    </svg>
                </button>
            </div>
            <div class="flex gap-3 text-[10px] text-gray-500">
                <span class="flex items-center gap-1"><span class="w-3 h-2 rounded bg-yellow-400 inline-block" /> {"< MEV"}</span>
                <span class="flex items-center gap-1"><span class="w-3 h-2 rounded bg-green-500 inline-block" /> {"MEV-MRV"}</span>
                <span class="flex items-center gap-1"><span class="w-3 h-2 rounded bg-red-500 inline-block" /> {"> MRV"}</span>
            </div>
            {render_group("Push", PUSH_MUSCLES)}
            {render_group("Pull", PULL_MUSCLES)}
            {render_group("Legs", LEG_MUSCLES)}
            {render_group("Core", CORE_MUSCLES)}
        </div>
    }
}

// ── Section B: Training Frequency ────────────────────────────────────────────

#[derive(Properties, PartialEq)]
struct FrequencyProps {
    workouts: Vec<Workout>,
    exercises: Vec<Exercise>,
}

#[function_component(SectionFrequency)]
fn section_frequency(props: &FrequencyProps) -> Html {
    let t = today();
    let from = t - chrono::Duration::days(14);

    // Track which days each muscle was trained
    let mut muscle_days: HashMap<String, Vec<NaiveDate>> = HashMap::new();

    for w in &props.workouts {
        if let Some(d) = parse_date(&w.date) {
            if d >= from && d <= t {
                for we in &w.exercises {
                    let completed = we.sets.iter().filter(|s| s.completed).count();
                    if completed == 0 {
                        continue;
                    }
                    let custom_mg = find_exercise(&props.exercises, &we.exercise_id)
                        .filter(|e| e.is_custom)
                        .map(|e| e.muscle_groups.as_slice());
                    let eff = effective_sets_for_exercise(&we.exercise_id, completed, custom_mg);
                    for (muscle, _) in eff {
                        let days = muscle_days.entry(muscle.to_string()).or_default();
                        if !days.contains(&d) {
                            days.push(d);
                        }
                    }
                }
            }
        }
    }

    if props.workouts.is_empty() {
        return html! {};
    }

    html! {
        <div class="bg-gray-100 dark:bg-gray-800 rounded-xl p-4 neu-flat transition-colors space-y-3">
            <h3 class="text-sm font-bold text-gray-900 dark:text-gray-100 uppercase tracking-wider">{"Training Frequency (14 days)"}</h3>
            <p class="text-xs text-gray-500">{"Times per week — 2x/week per muscle is optimal for hypertrophy."}</p>
            <div class="grid grid-cols-2 gap-x-4 gap-y-1.5">
                { for TRACKED_MUSCLES.iter().map(|&muscle| {
                    let days = muscle_days.get(muscle).map(|d| d.len()).unwrap_or(0);
                    let freq = days as f64 / 2.0; // 14 days = 2 weeks
                    let color = frequency_color(freq);
                    html! {
                        <div class="flex justify-between items-center">
                            <span class="text-xs text-gray-700 dark:text-gray-300 truncate">{muscle}</span>
                            <span class={classes!("text-xs", "font-bold", color)}>
                                {format!("{:.1}x", freq)}
                            </span>
                        </div>
                    }
                })}
            </div>
        </div>
    }
}

// ── Section C: Progressive Overload ──────────────────────────────────────────

#[derive(Properties, PartialEq)]
struct OverloadProps {
    workouts: Vec<Workout>,
    exercises: Vec<Exercise>,
}

struct OverloadEntry {
    name: String,
    trend: OverloadTrend,
    recent_1rm: f64,
}

enum OverloadTrend {
    Progressing,
    Stagnant,
    Regressing,
}

#[function_component(SectionOverload)]
fn section_overload(props: &OverloadProps) -> Html {
    let t = today();
    let from = t - chrono::Duration::days(28);

    // Group workout exercises by exercise_id within the last 4 weeks
    let mut exercise_sessions: HashMap<String, Vec<(NaiveDate, f64)>> = HashMap::new();

    for w in &props.workouts {
        if let Some(d) = parse_date(&w.date) {
            if d >= from && d <= t {
                for we in &w.exercises {
                    let max_e1rm = we
                        .sets
                        .iter()
                        .filter(|s| s.completed && s.weight > 0.0 && s.reps > 0)
                        .map(|s| estimate_1rm(s.weight, s.reps))
                        .fold(0.0_f64, f64::max);
                    if max_e1rm > 0.0 {
                        exercise_sessions
                            .entry(we.exercise_id.clone())
                            .or_default()
                            .push((d, max_e1rm));
                    }
                }
            }
        }
    }

    let mut entries: Vec<OverloadEntry> = Vec::new();
    for (eid, mut sessions) in exercise_sessions {
        if sessions.len() < 2 {
            continue;
        }
        sessions.sort_by_key(|(d, _)| *d);
        let n = sessions.len();
        let midpoint = n / 2;
        let early_avg: f64 =
            sessions[..midpoint].iter().map(|(_, v)| v).sum::<f64>() / midpoint as f64;
        let late_avg: f64 =
            sessions[midpoint..].iter().map(|(_, v)| v).sum::<f64>() / (n - midpoint) as f64;
        let recent_1rm = sessions.last().map(|(_, v)| *v).unwrap_or(0.0);

        let diff_pct = (late_avg - early_avg) / early_avg * 100.0;
        let trend = if diff_pct > 2.0 {
            OverloadTrend::Progressing
        } else if diff_pct < -2.0 {
            OverloadTrend::Regressing
        } else {
            OverloadTrend::Stagnant
        };

        let name = find_exercise(&props.exercises, &eid)
            .map(|e| e.name.clone())
            .unwrap_or_else(|| eid.clone());
        entries.push(OverloadEntry {
            name,
            trend,
            recent_1rm,
        });
    }

    entries.sort_by(|a, b| a.name.cmp(&b.name));

    if entries.is_empty() {
        return html! {};
    }

    let units = storage::load_user_config().unit_system;

    html! {
        <div class="bg-gray-100 dark:bg-gray-800 rounded-xl p-4 neu-flat transition-colors space-y-3">
            <h3 class="text-sm font-bold text-gray-900 dark:text-gray-100 uppercase tracking-wider">{"Progressive Overload (4 weeks)"}</h3>
            <div class="space-y-2">
                { for entries.iter().map(|e| {
                    let (arrow, color, label) = match e.trend {
                        OverloadTrend::Progressing => ("\u{2191}", "text-green-500", "Progressing"),
                        OverloadTrend::Stagnant => ("\u{2192}", "text-yellow-500", "Stagnant"),
                        OverloadTrend::Regressing => ("\u{2193}", "text-red-500", "Regressing"),
                    };
                    html! {
                        <div class="flex justify-between items-center">
                            <span class="text-xs text-gray-700 dark:text-gray-300 truncate mr-2">{&e.name}</span>
                            <div class="flex items-center gap-2 flex-shrink-0">
                                <span class="text-xs text-gray-500">
                                    {format!("e1RM {:.0}{}", units.display_weight(e.recent_1rm), units.weight_label())}
                                </span>
                                <span class={classes!("text-sm", "font-bold", color)} title={label}>
                                    {arrow}
                                </span>
                            </div>
                        </div>
                    }
                })}
            </div>
            { if entries.iter().any(|e| matches!(e.trend, OverloadTrend::Stagnant)) {
                html! {
                    <p class="text-xs text-yellow-500 bg-yellow-500/10 rounded-lg p-2 mt-2">
                        {"Stagnant exercises: Try adding reps, increasing weight, or changing rep range."}
                    </p>
                }
            } else {
                html! {}
            }}
        </div>
    }
}

// ── Section D: Rep Range Distribution ────────────────────────────────────────

#[derive(Properties, PartialEq)]
struct RepRangeProps {
    workouts: Vec<Workout>,
}

#[function_component(SectionRepRange)]
fn section_rep_range(props: &RepRangeProps) -> Html {
    let t = today();
    let from = t - chrono::Duration::days(28);

    let mut strength = 0u32; // 1-5
    let mut hypertrophy = 0u32; // 6-12
    let mut endurance = 0u32; // 13+

    for w in &props.workouts {
        if let Some(d) = parse_date(&w.date) {
            if d >= from && d <= t {
                for we in &w.exercises {
                    for s in &we.sets {
                        if s.completed && s.reps > 0 {
                            match s.reps {
                                1..=5 => strength += 1,
                                6..=12 => hypertrophy += 1,
                                _ => endurance += 1,
                            }
                        }
                    }
                }
            }
        }
    }

    let total = strength + hypertrophy + endurance;
    if total == 0 {
        return html! {};
    }

    let pct = |v: u32| (v as f64 / total as f64 * 100.0) as u32;
    let s_pct = pct(strength);
    let h_pct = pct(hypertrophy);
    let e_pct = pct(endurance);

    let one_range_warning = (s_pct == 100) || (h_pct == 100) || (e_pct == 100);

    html! {
        <div class="bg-gray-100 dark:bg-gray-800 rounded-xl p-4 neu-flat transition-colors space-y-3">
            <h3 class="text-sm font-bold text-gray-900 dark:text-gray-100 uppercase tracking-wider">{"Rep Range Distribution (4 weeks)"}</h3>
            // Stacked bar
            <div class="h-6 rounded-full overflow-hidden flex bg-gray-200 dark:bg-gray-700">
                if s_pct > 0 {
                    <div class="bg-red-400 dark:bg-red-500 flex items-center justify-center text-[10px] font-bold text-white"
                         style={format!("width: {}%", s_pct)}>
                        { if s_pct >= 10 { format!("{}%", s_pct) } else { String::new() } }
                    </div>
                }
                if h_pct > 0 {
                    <div class="bg-green-400 dark:bg-green-500 flex items-center justify-center text-[10px] font-bold text-white"
                         style={format!("width: {}%", h_pct)}>
                        { if h_pct >= 10 { format!("{}%", h_pct) } else { String::new() } }
                    </div>
                }
                if e_pct > 0 {
                    <div class="bg-blue-400 dark:bg-blue-500 flex items-center justify-center text-[10px] font-bold text-white"
                         style={format!("width: {}%", e_pct)}>
                        { if e_pct >= 10 { format!("{}%", e_pct) } else { String::new() } }
                    </div>
                }
            </div>
            <div class="flex justify-between text-[10px] text-gray-500">
                <span class="flex items-center gap-1"><span class="w-2 h-2 rounded bg-red-400 inline-block" /> {format!("Strength 1-5 ({}%)", s_pct)}</span>
                <span class="flex items-center gap-1"><span class="w-2 h-2 rounded bg-green-400 inline-block" /> {format!("Hypertrophy 6-12 ({}%)", h_pct)}</span>
                <span class="flex items-center gap-1"><span class="w-2 h-2 rounded bg-blue-400 inline-block" /> {format!("Endurance 13+ ({}%)", e_pct)}</span>
            </div>
            { if one_range_warning {
                html! {
                    <p class="text-xs text-yellow-500 bg-yellow-500/10 rounded-lg p-2">
                        {"All your sets are in one rep range. Consider varying rep ranges for well-rounded development."}
                    </p>
                }
            } else {
                html! {}
            }}
        </div>
    }
}

// ── Section E: Deload Recommendation ─────────────────────────────────────────

#[derive(Properties, PartialEq)]
struct DeloadProps {
    workouts: Vec<Workout>,
}

#[function_component(SectionDeload)]
fn section_deload(props: &DeloadProps) -> Html {
    let t = today();
    // Look at last 6 weeks
    let mut weekly_volume: Vec<(String, f64)> = Vec::new();

    for i in (0..6).rev() {
        let week_end = t - chrono::Duration::weeks(i);
        let week_start = week_end - chrono::Duration::days(6);
        let label = format!("W{}", 6 - i);
        let vol: f64 = props
            .workouts
            .iter()
            .filter_map(|w| {
                let d = parse_date(&w.date)?;
                if d >= week_start && d <= week_end {
                    Some(
                        w.exercises
                            .iter()
                            .map(|e| e.sets.iter().filter(|s| s.completed).count() as f64)
                            .sum::<f64>(),
                    )
                } else {
                    None
                }
            })
            .sum();
        weekly_volume.push((label, vol));
    }

    // Check for 4+ consecutive weeks of increasing volume
    let volumes: Vec<f64> = weekly_volume.iter().map(|(_, v)| *v).collect();
    let mut increasing_streak = 0u32;
    for i in 1..volumes.len() {
        if volumes[i] > volumes[i - 1] && volumes[i - 1] > 0.0 {
            increasing_streak += 1;
        } else {
            increasing_streak = 0;
        }
    }

    let total_sets: f64 = volumes.iter().sum();
    if total_sets == 0.0 {
        return html! {};
    }

    let should_deload = increasing_streak >= 4;

    html! {
        <div class="bg-gray-100 dark:bg-gray-800 rounded-xl p-4 neu-flat transition-colors space-y-3">
            <h3 class="text-sm font-bold text-gray-900 dark:text-gray-100 uppercase tracking-wider">{"Deload Check (6 weeks)"}</h3>
            <div class="flex items-end gap-1 h-16">
                { for weekly_volume.iter().map(|(label, vol)| {
                    let max_vol = volumes.iter().cloned().fold(0.0_f64, f64::max);
                    let pct = if max_vol > 0.0 { vol / max_vol * 100.0 } else { 0.0 };
                    html! {
                        <div class="flex-1 flex flex-col items-center gap-0.5">
                            <div class={classes!(
                                "w-full", "rounded-t", "transition-all",
                                if should_deload { "bg-red-400 dark:bg-red-500" } else { "bg-blue-400 dark:bg-blue-500" }
                            )} style={format!("height: {}%", pct.max(4.0))} />
                            <span class="text-[9px] text-gray-500">{label}</span>
                            <span class="text-[9px] text-gray-400">{format!("{:.0}", vol)}</span>
                        </div>
                    }
                })}
            </div>
            { if should_deload {
                html! {
                    <p class="text-xs text-red-500 bg-red-500/10 rounded-lg p-2 font-medium">
                        {"Volume has increased for 4+ weeks. Consider a deload week — reduce volume by 40-50%."}
                    </p>
                }
            } else {
                html! {
                    <p class="text-xs text-gray-500">{"Volume trend looks manageable. No deload needed yet."}</p>
                }
            }}
        </div>
    }
}

// ── Section F: Push/Pull Balance ─────────────────────────────────────────────

#[derive(Properties, PartialEq)]
struct PushPullProps {
    workouts: Vec<Workout>,
    exercises: Vec<Exercise>,
}

#[function_component(SectionPushPull)]
fn section_push_pull(props: &PushPullProps) -> Html {
    let t = today();
    let from = t - chrono::Duration::days(7);
    let sets = compute_muscle_sets(&props.workouts, &props.exercises, from, t);

    let push_total: f64 = PUSH_MUSCLES
        .iter()
        .map(|m| sets.get(*m).copied().unwrap_or(0.0))
        .sum();
    let pull_total: f64 = PULL_MUSCLES
        .iter()
        .map(|m| sets.get(*m).copied().unwrap_or(0.0))
        .sum();

    if push_total == 0.0 && pull_total == 0.0 {
        return html! {};
    }

    let ratio = if pull_total > 0.0 {
        push_total / pull_total
    } else {
        f64::INFINITY
    };

    let (ratio_text, ratio_color) = if ratio.is_infinite() || ratio.is_nan() {
        ("N/A".to_string(), "text-gray-500")
    } else if (0.8..=1.2).contains(&ratio) {
        (format!("{:.1}:1", ratio), "text-green-500")
    } else if !(0.6..=1.5).contains(&ratio) {
        (format!("{:.1}:1", ratio), "text-red-500")
    } else {
        (format!("{:.1}:1", ratio), "text-yellow-500")
    };

    let total = push_total + pull_total;
    let push_pct = if total > 0.0 {
        (push_total / total * 100.0) as u32
    } else {
        50
    };
    let pull_pct = 100 - push_pct;

    html! {
        <div class="bg-gray-100 dark:bg-gray-800 rounded-xl p-4 neu-flat transition-colors space-y-3">
            <h3 class="text-sm font-bold text-gray-900 dark:text-gray-100 uppercase tracking-wider">{"Push/Pull Balance (7 days)"}</h3>
            <div class="flex justify-between items-center">
                <span class="text-xs text-gray-500">{format!("Push: {:.0} sets", push_total)}</span>
                <span class={classes!("text-lg", "font-bold", ratio_color)}>{ratio_text}</span>
                <span class="text-xs text-gray-500">{format!("Pull: {:.0} sets", pull_total)}</span>
            </div>
            <div class="h-4 rounded-full overflow-hidden flex bg-gray-200 dark:bg-gray-700">
                <div class="bg-blue-400 dark:bg-blue-500 h-full transition-all" style={format!("width: {}%", push_pct)} />
                <div class="bg-purple-400 dark:bg-purple-500 h-full transition-all" style={format!("width: {}%", pull_pct)} />
            </div>
            <div class="flex justify-between text-[10px] text-gray-500">
                <span>{"Push"}</span>
                <span>{"Pull"}</span>
            </div>
            { if ratio > 1.5 {
                html! {
                    <p class="text-xs text-yellow-500 bg-yellow-500/10 rounded-lg p-2">
                        {"Push-dominant imbalance. Add more pulling exercises (rows, face pulls) to protect shoulder health."}
                    </p>
                }
            } else if ratio < 0.6 && !ratio.is_nan() {
                html! {
                    <p class="text-xs text-yellow-500 bg-yellow-500/10 rounded-lg p-2">
                        {"Pull-dominant imbalance. Consider adding more pushing exercises."}
                    </p>
                }
            } else {
                html! {}
            }}
        </div>
    }
}

// ── Section G: Session Volume Check ──────────────────────────────────────────

#[derive(Properties, PartialEq)]
struct SessionVolumeProps {
    workouts: Vec<Workout>,
    exercises: Vec<Exercise>,
}

#[function_component(SectionSessionVolume)]
fn section_session_volume(props: &SessionVolumeProps) -> Html {
    let t = today();
    let from = t - chrono::Duration::days(14);

    let mut warnings: Vec<(String, String, f64)> = Vec::new(); // (muscle, date, sets)

    for w in &props.workouts {
        if let Some(d) = parse_date(&w.date) {
            if d >= from && d <= t {
                let session_sets = session_muscle_sets(w, &props.exercises);
                for (muscle, sets) in session_sets {
                    if sets > 10.0 {
                        warnings.push((muscle, w.date.clone(), sets));
                    }
                }
            }
        }
    }

    if warnings.is_empty() {
        return html! {};
    }

    warnings.sort_by(|a, b| b.1.cmp(&a.1));
    warnings.truncate(5);

    html! {
        <div class="bg-gray-100 dark:bg-gray-800 rounded-xl p-4 neu-flat transition-colors space-y-3">
            <h3 class="text-sm font-bold text-gray-900 dark:text-gray-100 uppercase tracking-wider">{"Session Volume Warnings"}</h3>
            <p class="text-xs text-gray-500">{"Research suggests >10 direct sets per muscle per session has diminishing returns."}</p>
            <div class="space-y-2">
                { for warnings.iter().map(|(muscle, date, sets)| {
                    html! {
                        <div class="text-xs bg-yellow-500/10 text-yellow-600 dark:text-yellow-400 rounded-lg p-2">
                            {format!("{:.0} sets of {} on {}", sets, muscle, date)}
                        </div>
                    }
                })}
            </div>
        </div>
    }
}

// ── Section H: Recovery Tracking ─────────────────────────────────────────────

#[derive(Properties, PartialEq)]
struct RecoveryProps {
    workouts: Vec<Workout>,
    exercises: Vec<Exercise>,
}

#[function_component(SectionRecovery)]
fn section_recovery(props: &RecoveryProps) -> Html {
    let t = today();
    // Find last trained date per muscle
    let mut last_trained: HashMap<String, NaiveDate> = HashMap::new();

    for w in &props.workouts {
        if let Some(d) = parse_date(&w.date) {
            for we in &w.exercises {
                let completed = we.sets.iter().filter(|s| s.completed).count();
                if completed == 0 {
                    continue;
                }
                let custom_mg = find_exercise(&props.exercises, &we.exercise_id)
                    .filter(|e| e.is_custom)
                    .map(|e| e.muscle_groups.as_slice());
                let eff = effective_sets_for_exercise(&we.exercise_id, completed, custom_mg);
                for (muscle, _) in eff {
                    let entry = last_trained
                        .entry(muscle.to_string())
                        .or_insert(NaiveDate::from_ymd_opt(2000, 1, 1).unwrap());
                    if d > *entry {
                        *entry = d;
                    }
                }
            }
        }
    }

    if last_trained.is_empty() {
        return html! {};
    }

    // Sort by most recently trained
    let mut entries: Vec<(&str, f64)> = TRACKED_MUSCLES
        .iter()
        .filter_map(|&muscle| {
            let d = last_trained.get(muscle)?;
            let days_ago = (t - *d).num_days() as f64;
            let hours_approx = days_ago * 24.0;
            Some((muscle, hours_approx))
        })
        .collect();
    entries.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    html! {
        <div class="bg-gray-100 dark:bg-gray-800 rounded-xl p-4 neu-flat transition-colors space-y-3">
            <h3 class="text-sm font-bold text-gray-900 dark:text-gray-100 uppercase tracking-wider">{"Recovery Status"}</h3>
            <p class="text-xs text-gray-500">{"Optimal: 48-72h between sessions for the same muscle."}</p>
            <div class="grid grid-cols-2 gap-x-4 gap-y-1.5">
                { for entries.iter().map(|(muscle, hours)| {
                    let color = recovery_color(*hours);
                    let display = if *hours < 24.0 {
                        format!("{:.0}h ago", hours)
                    } else {
                        let days = (*hours / 24.0).floor();
                        format!("{:.0}d ago", days)
                    };
                    html! {
                        <div class="flex justify-between items-center">
                            <span class="text-xs text-gray-700 dark:text-gray-300 truncate">{muscle}</span>
                            <span class={classes!("text-xs", "font-bold", color)}>{display}</span>
                        </div>
                    }
                })}
            </div>
        </div>
    }
}

// ── Threshold Customization Modal ────────────────────────────────────────────

#[derive(Properties, PartialEq)]
struct ThresholdModalProps {
    visible: UseStateHandle<bool>,
}

#[function_component(ThresholdModal)]
fn threshold_modal(props: &ThresholdModalProps) -> Html {
    let thresholds = use_state(get_thresholds);

    if !*props.visible {
        return html! {};
    }

    let on_close = {
        let visible = props.visible.clone();
        Callback::from(move |_: MouseEvent| visible.set(false))
    };

    let on_save = {
        let visible = props.visible.clone();
        let thresholds = thresholds.clone();
        Callback::from(move |_: MouseEvent| {
            let mut config = storage::load_user_config();
            config.muscle_thresholds = Some((*thresholds).clone());
            storage::save_user_config(&config);
            visible.set(false);
        })
    };

    let on_reset = {
        let thresholds = thresholds.clone();
        Callback::from(move |_: MouseEvent| {
            let defaults: HashMap<String, (f64, f64)> = muscle_data::default_thresholds()
                .iter()
                .map(|(k, v)| (k.to_string(), *v))
                .collect();
            thresholds.set(defaults);
        })
    };

    html! {
        <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 px-4" onclick={on_close.clone()}>
            <div class="bg-white dark:bg-gray-800 rounded-2xl p-4 w-full max-w-md max-h-[80vh] overflow-y-auto shadow-xl"
                 onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}>
                <div class="flex justify-between items-center mb-4">
                    <h3 class="text-lg font-bold text-gray-900 dark:text-gray-100">{"Volume Thresholds"}</h3>
                    <button onclick={on_close} class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 text-xl">{"\u{00d7}"}</button>
                </div>
                <p class="text-xs text-gray-500 mb-4">{"MEV = Minimum Effective Volume, MRV = Maximum Recoverable Volume (sets/week)"}</p>
                <div class="space-y-3">
                    { for TRACKED_MUSCLES.iter().map(|&muscle| {
                        let (mev, mrv) = thresholds.get(muscle).copied().unwrap_or((0.0, 20.0));
                        let thresholds_mev = thresholds.clone();
                        let thresholds_mrv = thresholds.clone();
                        let muscle_str = muscle.to_string();
                        let muscle_str2 = muscle.to_string();
                        html! {
                            <div class="flex items-center gap-2">
                                <span class="w-24 text-xs text-gray-700 dark:text-gray-300 truncate">{muscle}</span>
                                <div class="flex-1 flex items-center gap-1">
                                    <input type="number" value={format!("{:.0}", mev)}
                                        class="w-14 px-1.5 py-1 text-xs bg-gray-100 dark:bg-gray-700 rounded text-center text-gray-900 dark:text-gray-100 outline-none"
                                        onchange={Callback::from(move |e: Event| {
                                            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                            let val: f64 = input.value().parse().unwrap_or(0.0);
                                            let mut t = (*thresholds_mev).clone();
                                            let entry = t.entry(muscle_str.clone()).or_insert((0.0, 20.0));
                                            entry.0 = val;
                                            thresholds_mev.set(t);
                                        })}
                                    />
                                    <span class="text-xs text-gray-400">{"-"}</span>
                                    <input type="number" value={format!("{:.0}", mrv)}
                                        class="w-14 px-1.5 py-1 text-xs bg-gray-100 dark:bg-gray-700 rounded text-center text-gray-900 dark:text-gray-100 outline-none"
                                        onchange={Callback::from(move |e: Event| {
                                            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                            let val: f64 = input.value().parse().unwrap_or(20.0);
                                            let mut t = (*thresholds_mrv).clone();
                                            let entry = t.entry(muscle_str2.clone()).or_insert((0.0, 20.0));
                                            entry.1 = val;
                                            thresholds_mrv.set(t);
                                        })}
                                    />
                                </div>
                            </div>
                        }
                    })}
                </div>
                <div class="flex gap-2 mt-4">
                    <button onclick={on_save}
                        class="flex-1 py-2.5 bg-blue-600 text-white rounded-lg font-bold text-sm hover:bg-blue-700 transition-colors neu-btn">
                        {"Save"}
                    </button>
                    <button onclick={on_reset}
                        class="py-2.5 px-4 bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded-lg font-bold text-sm hover:bg-gray-300 dark:hover:bg-gray-600 transition-colors neu-btn">
                        {"Reset"}
                    </button>
                </div>
            </div>
        </div>
    }
}

// ── Quick Summary (for Home page card) ───────────────────────────────────────

pub fn muscle_balance_summary(workouts: &[Workout], exercises: &[Exercise]) -> (u32, u32) {
    let t = today();
    let from = t - chrono::Duration::days(7);
    let sets = compute_muscle_sets(workouts, exercises, from, t);
    let thresholds = get_thresholds();

    let mut undertrained = 0u32;
    let mut overtrained = 0u32;

    for &muscle in TRACKED_MUSCLES {
        let s = sets.get(muscle).copied().unwrap_or(0.0);
        let (mev, mrv) = thresholds.get(muscle).copied().unwrap_or((0.0, 20.0));
        if mev > 0.0 && s < mev && s > 0.0 {
            undertrained += 1;
        } else if s > mrv {
            overtrained += 1;
        }
    }

    (undertrained, overtrained)
}
