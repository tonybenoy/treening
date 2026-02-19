use chrono::Datelike;
use std::collections::HashMap;
use yew::prelude::*;

// ── StatCard ────────────────────────────────────────────────────────────────

#[derive(Properties, PartialEq)]
pub struct StatCardProps {
    pub label: AttrValue,
    pub value: AttrValue,
    pub icon: AttrValue,
}

#[function_component(StatCard)]
pub fn stat_card(props: &StatCardProps) -> Html {
    html! {
        <div class="bg-gray-100 dark:bg-gray-800 rounded-xl p-4 flex flex-col items-center gap-1 neu-flat transition-colors">
            <span class="text-2xl">{&props.icon}</span>
            <span class="text-xl font-bold text-gray-900 dark:text-gray-100">{&props.value}</span>
            <span class="text-xs text-gray-500 dark:text-gray-400 text-center">{&props.label}</span>
        </div>
    }
}

// ── BarChart ─────────────────────────────────────────────────────────────────

#[derive(Properties, PartialEq)]
pub struct BarChartProps {
    pub data: Vec<(String, f64)>,
    #[prop_or(200)]
    pub height: u32,
    #[prop_or(AttrValue::from("#3b82f6"))]
    pub color: AttrValue,
    #[prop_or_default]
    pub title: AttrValue,
}

#[function_component(BarChart)]
pub fn bar_chart(props: &BarChartProps) -> Html {
    if props.data.is_empty() {
        return html! {
            <div class="text-gray-500 dark:text-gray-400 text-center py-8 text-sm transition-colors">{"No data yet"}</div>
        };
    }

    let max_val = props.data.iter().map(|(_, v)| *v).fold(0.0_f64, f64::max);
    let max_val = if max_val == 0.0 { 1.0 } else { max_val };

    let n = props.data.len();
    let chart_w: f64 = 300.0;
    let chart_h: f64 = props.height as f64;
    let padding_top: f64 = 20.0;
    let padding_bottom: f64 = 30.0;
    let padding_left: f64 = 10.0;
    let padding_right: f64 = 10.0;
    let draw_w = chart_w - padding_left - padding_right;
    let draw_h = chart_h - padding_top - padding_bottom;
    let bar_gap: f64 = 4.0;
    let bar_w = (draw_w - bar_gap * (n as f64 + 1.0)) / n as f64;
    let total_w = chart_w;
    let total_h = chart_h;
    let viewbox = format!("0 0 {} {}", total_w, total_h);

    html! {
        <div class="w-full">
            if !props.title.is_empty() {
                <h3 class="text-sm font-semibold text-gray-600 dark:text-gray-300 mb-2 transition-colors">{&props.title}</h3>
            }
            <svg viewBox={viewbox} class="w-full" preserveAspectRatio="xMidYMid meet">
                // baseline
                <line x1={format!("{}", padding_left)}
                      y1={format!("{}", padding_top + draw_h)}
                      x2={format!("{}", padding_left + draw_w)}
                      y2={format!("{}", padding_top + draw_h)}
                      stroke="currentColor" stroke-width="1" class="text-gray-300 dark:text-gray-600"/>
                { for props.data.iter().enumerate().map(|(i, (label, val))| {
                    let bar_h = (val / max_val) * draw_h;
                    let x = padding_left + bar_gap + (i as f64) * (bar_w + bar_gap);
                    let y = padding_top + draw_h - bar_h;
                    let label_x = x + bar_w / 2.0;
                    let label_y = padding_top + draw_h + 14.0;
                    let val_y = y - 4.0;
                    let val_text = if *val == val.floor() {
                        format!("{}", *val as i64)
                    } else {
                        format!("{:.0}", val)
                    };
                    html! {
                        <>
                            <rect x={format!("{}", x)} y={format!("{}", y)}
                                  width={format!("{}", bar_w)} height={format!("{}", bar_h)}
                                  fill={props.color.to_string()} rx="2"/>
                            <text x={format!("{}", label_x)} y={format!("{}", label_y)}
                                  text-anchor="middle" fill="currentColor" font-size="9" class="text-gray-500 dark:text-gray-400">{label}</text>
                            if *val > 0.0 {
                                <text x={format!("{}", label_x)} y={format!("{}", val_y)}
                                      text-anchor="middle" fill="currentColor" font-size="9" class="text-gray-700 dark:text-gray-300">{val_text}</text>
                            }
                        </>
                    }
                })}
            </svg>
        </div>
    }
}

// ── LineChart ────────────────────────────────────────────────────────────────

#[derive(Properties, PartialEq)]
pub struct LineChartProps {
    pub data: Vec<(String, f64)>,
    #[prop_or(200)]
    pub height: u32,
    #[prop_or(AttrValue::from("#10b981"))]
    pub color: AttrValue,
    #[prop_or_default]
    pub title: AttrValue,
}

#[function_component(LineChart)]
pub fn line_chart(props: &LineChartProps) -> Html {
    if props.data.is_empty() {
        return html! {
            <div class="text-gray-500 dark:text-gray-400 text-center py-8 text-sm transition-colors">{"No data yet"}</div>
        };
    }

    let max_val = props.data.iter().map(|(_, v)| *v).fold(0.0_f64, f64::max);
    let min_val = props.data.iter().map(|(_, v)| *v).fold(f64::MAX, f64::min);
    let range = if (max_val - min_val).abs() < 0.001 {
        1.0
    } else {
        max_val - min_val
    };

    let chart_w: f64 = 300.0;
    let chart_h: f64 = props.height as f64;
    let padding_top: f64 = 20.0;
    let padding_bottom: f64 = 30.0;
    let padding_left: f64 = 10.0;
    let padding_right: f64 = 10.0;
    let draw_w = chart_w - padding_left - padding_right;
    let draw_h = chart_h - padding_top - padding_bottom;
    let viewbox = format!("0 0 {} {}", chart_w, chart_h);

    let n = props.data.len();
    let step_x = if n > 1 { draw_w / (n - 1) as f64 } else { 0.0 };

    let points: Vec<(f64, f64)> = props
        .data
        .iter()
        .enumerate()
        .map(|(i, (_, val))| {
            let x = padding_left + i as f64 * step_x;
            let y = padding_top + draw_h - ((val - min_val) / range) * draw_h;
            (x, y)
        })
        .collect();

    let polyline_points: String = points
        .iter()
        .map(|(x, y)| format!("{},{}", x, y))
        .collect::<Vec<_>>()
        .join(" ");

    html! {
        <div class="w-full">
            if !props.title.is_empty() {
                <h3 class="text-sm font-semibold text-gray-600 dark:text-gray-300 mb-2 transition-colors">{&props.title}</h3>
            }
            <svg viewBox={viewbox} class="w-full" preserveAspectRatio="xMidYMid meet">
                // baseline
                <line x1={format!("{}", padding_left)}
                      y1={format!("{}", padding_top + draw_h)}
                      x2={format!("{}", padding_left + draw_w)}
                      y2={format!("{}", padding_top + draw_h)}
                      stroke="currentColor" stroke-width="1" class="text-gray-300 dark:text-gray-600"/>
                // line
                <polyline points={polyline_points} fill="none"
                          stroke={props.color.to_string()} stroke-width="2"
                          stroke-linejoin="round" stroke-linecap="round"/>
                // dots + labels
                { for points.iter().enumerate().map(|(i, (x, y))| {
                    let label = &props.data[i].0;
                    let val = props.data[i].1;
                    let label_y = padding_top + draw_h + 14.0;
                    let val_text = if val >= 1000.0 {
                        format!("{:.0}k", val / 1000.0)
                    } else if val == val.floor() {
                        format!("{}", val as i64)
                    } else {
                        format!("{:.1}", val)
                    };
                    html! {
                        <>
                            <circle cx={format!("{}", x)} cy={format!("{}", y)} r="3"
                                    fill={props.color.to_string()}/>
                            <text x={format!("{}", x)} y={format!("{}", label_y)}
                                  text-anchor="middle" fill="currentColor" font-size="9" class="text-gray-500 dark:text-gray-400">{label}</text>
                            <text x={format!("{}", x)} y={format!("{}", y - 6.0)}
                                  text-anchor="middle" fill="currentColor" font-size="8" class="text-gray-700 dark:text-gray-300">{val_text}</text>
                        </>
                    }
                })}
            </svg>
        </div>
    }
}

// ── HorizontalBarChart ──────────────────────────────────────────────────────

#[derive(Properties, PartialEq)]
pub struct HorizontalBarChartProps {
    /// (label, value, color)
    pub data: Vec<(String, f64, String)>,
    #[prop_or(AttrValue::from("Muscle Group Distribution"))]
    pub title: AttrValue,
}

#[function_component(HorizontalBarChart)]
pub fn horizontal_bar_chart(props: &HorizontalBarChartProps) -> Html {
    if props.data.is_empty() {
        return html! {
            <div class="text-gray-500 dark:text-gray-400 text-center py-8 text-sm transition-colors">{"No data yet"}</div>
        };
    }

    let max_val = props
        .data
        .iter()
        .map(|(_, v, _)| *v)
        .fold(0.0_f64, f64::max);
    let max_val = if max_val == 0.0 { 1.0 } else { max_val };

    html! {
        <div class="w-full">
            if !props.title.is_empty() {
                <h3 class="text-sm font-semibold text-gray-600 dark:text-gray-300 mb-2 transition-colors">{&props.title}</h3>
            }
            <div class="space-y-2">
                { for props.data.iter().map(|(label, val, color)| {
                    let pct = (val / max_val) * 100.0;
                    let width_style = format!("width: {}%", pct);
                    let count = if *val == val.floor() {
                        format!("{}", *val as i64)
                    } else {
                        format!("{:.0}", val)
                    };
                    html! {
                        <div class="flex items-center gap-2">
                            <span class="text-xs text-gray-500 dark:text-gray-400 w-20 text-right transition-colors">{label}</span>
                            <div class="flex-1 bg-gray-200 dark:bg-gray-700 rounded h-5 overflow-hidden transition-colors">
                                <div class="h-full rounded flex items-center pl-1"
                                     style={format!("{}; background-color: {}", width_style, color)}>
                                    if pct > 15.0 {
                                        <span class="text-xs text-white font-medium">{&count}</span>
                                    }
                                </div>
                            </div>
                            if pct <= 15.0 {
                                <span class="text-xs text-gray-600 dark:text-gray-400 transition-colors">{&count}</span>
                            }
                        </div>
                    }
                })}
            </div>
        </div>
    }
}

// ── CalendarHeatmap ─────────────────────────────────────────────────────────

#[derive(Properties, PartialEq)]
pub struct CalendarHeatmapProps {
    /// Map of "YYYY-MM-DD" -> workout count for that day
    pub data: HashMap<String, u32>,
}

#[function_component(CalendarHeatmap)]
pub fn calendar_heatmap(props: &CalendarHeatmapProps) -> Html {
    let is_dark = gloo::utils::document()
        .document_element()
        .and_then(|el| el.get_attribute("class"))
        .map(|c| c.contains("dark"))
        .unwrap_or(false);
    let today = chrono::Local::now().date_naive();
    // 20 columns (weeks) x 7 rows (days, Mon=0..Sun=6)
    let cols = 20u32;
    let rows = 7u32;
    let cell = 14.0_f64;
    let gap = 2.0_f64;
    let padding_left = 24.0_f64;
    let padding_top = 4.0_f64;
    let total_w = padding_left + cols as f64 * (cell + gap);
    let total_h = padding_top + rows as f64 * (cell + gap) + 2.0;
    let viewbox = format!("0 0 {} {}", total_w, total_h);

    // Start date: go back (cols * 7 - 1) days from today, align to Monday
    let total_days = cols * rows;
    let start = today - chrono::Duration::days(total_days as i64 - 1);
    // Align to Monday (weekday 0 = Monday in chrono)
    let weekday = start.weekday().num_days_from_monday();
    let start = start - chrono::Duration::days(weekday as i64);

    let day_labels = ["M", "", "W", "", "F", "", ""];

    html! {
        <div class="w-full">
            <h3 class="text-sm font-semibold text-gray-600 dark:text-gray-300 mb-2 transition-colors">{"Workout Calendar"}</h3>
            <svg viewBox={viewbox} class="w-full" preserveAspectRatio="xMidYMid meet">
                // Day labels
                { for (0..rows).map(|row| {
                    let label = day_labels[row as usize];
                    if label.is_empty() {
                        return html! {};
                    }
                    let y = padding_top + row as f64 * (cell + gap) + cell * 0.75;
                    html! {
                        <text x="0" y={format!("{}", y)} fill="currentColor" font-size="8" class="text-gray-500 dark:text-gray-500">{label}</text>
                    }
                })}
                // Cells
                { for (0..cols).flat_map(|col| {
                    (0..rows).map(move |row| {
                        let day_offset = col * rows + row;
                        let date = start + chrono::Duration::days(day_offset as i64);
                        let date_str = date.format("%Y-%m-%d").to_string();
                        let count = props.data.get(&date_str).copied().unwrap_or(0);
                        let x = padding_left + col as f64 * (cell + gap);
                        let y = padding_top + row as f64 * (cell + gap);

                        let fill = if date > today {
                            "transparent"
                        } else {
                            match count {
                                0 => {
                                    if is_dark {
                                        "#374151" // gray-700
                                    } else {
                                        "#e5e7eb" // gray-200
                                    }
                                }
                                1 => "#166534",
                                _ => "#22c55e",
                            }
                        };

                        html! {
                            <rect
                                x={format!("{}", x)} y={format!("{}", y)}
                                width={format!("{}", cell)} height={format!("{}", cell)}
                                rx="2" fill={fill}
                            />
                        }
                    })
                })}
            </svg>
        </div>
    }
}
