use crate::components::workout_log::compute_plates;
use crate::storage;
use yew::prelude::*;

#[function_component(PlateCalculatorPage)]
pub fn plate_calculator_page() -> Html {
    let config = storage::load_user_config();
    let unit_system = config.unit_system.clone();

    let target_weight = use_state(|| 100.0_f64);
    let bar_weight = use_state(|| config.bar_weight);
    let custom_bar = use_state(|| false);

    let plates = compute_plates(*target_weight, *bar_weight);
    let wl = unit_system.weight_label();

    let on_target_change = {
        let target_weight = target_weight.clone();
        let unit_sys = unit_system.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            if let Ok(val) = input.value().parse::<f64>() {
                target_weight.set(unit_sys.to_kg(val));
            }
        })
    };

    let bar_options: Vec<f64> = vec![20.0, 15.0, 10.0];

    html! {
        <div class="px-4 py-4 space-y-6">
            <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">{"Plate Calculator"}</h1>

            // Target weight input
            <div class="bg-gray-100 dark:bg-gray-800 rounded-2xl p-5 neu-flat transition-colors space-y-4">
                <div>
                    <label class="text-xs text-gray-500 uppercase font-bold block mb-1">{format!("Target Weight ({})", wl)}</label>
                    <input
                        type="text"
                        inputmode="decimal"
                        class="w-full px-4 py-3 bg-white dark:bg-gray-700 rounded-xl text-2xl font-bold text-center text-gray-900 dark:text-gray-100 outline-none neu-pressed transition-colors"
                        value={format!("{:.1}", unit_system.display_weight(*target_weight))}
                        oninput={on_target_change}
                    />
                </div>

                // Bar weight selector
                <div>
                    <label class="text-xs text-gray-500 uppercase font-bold block mb-2">{format!("Bar Weight ({})", wl)}</label>
                    <div class="flex gap-2 flex-wrap">
                        { for bar_options.iter().map(|&bw| {
                            let bar_weight = bar_weight.clone();
                            let custom_bar = custom_bar.clone();
                            let is_selected = !*custom_bar && *bar_weight == bw;
                            let display_bw = unit_system.display_weight(bw);
                            html! {
                                <button
                                    class={classes!(
                                        "px-4", "py-2", "rounded-xl", "text-sm", "font-bold", "transition-all",
                                        if is_selected {
                                            "bg-blue-600 text-white neu-btn"
                                        } else {
                                            "bg-gray-200 dark:bg-gray-700 text-gray-600 dark:text-gray-400 hover:bg-gray-300 dark:hover:bg-gray-600"
                                        }
                                    )}
                                    onclick={Callback::from(move |_| {
                                        bar_weight.set(bw);
                                        custom_bar.set(false);
                                    })}
                                >
                                    {format!("{:.0}{}", display_bw, if bw == 20.0 { " (Olympic)" } else if bw == 15.0 { " (Women)" } else { " (EZ/Short)" })}
                                </button>
                            }
                        })}
                        <button
                            class={classes!(
                                "px-4", "py-2", "rounded-xl", "text-sm", "font-bold", "transition-all",
                                if *custom_bar {
                                    "bg-blue-600 text-white neu-btn"
                                } else {
                                    "bg-gray-200 dark:bg-gray-700 text-gray-600 dark:text-gray-400 hover:bg-gray-300 dark:hover:bg-gray-600"
                                }
                            )}
                            onclick={{
                                let custom_bar = custom_bar.clone();
                                Callback::from(move |_| custom_bar.set(true))
                            }}
                        >
                            {"Custom"}
                        </button>
                    </div>
                    { if *custom_bar {
                        let bar_weight = bar_weight.clone();
                        let unit_sys = unit_system.clone();
                        html! {
                            <input
                                type="text"
                                inputmode="decimal"
                                class="mt-2 w-full px-3 py-2 bg-white dark:bg-gray-700 rounded-xl text-sm text-center text-gray-900 dark:text-gray-100 outline-none neu-pressed transition-colors"
                                placeholder="Custom bar weight"
                                value={format!("{:.1}", unit_sys.display_weight(*bar_weight))}
                                oninput={Callback::from(move |e: InputEvent| {
                                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                    if let Ok(val) = input.value().parse::<f64>() {
                                        bar_weight.set(unit_sys.to_kg(val));
                                    }
                                })}
                            />
                        }
                    } else {
                        html! {}
                    }}
                </div>
            </div>

            // Plate breakdown
            <div class="bg-gray-100 dark:bg-gray-800 rounded-2xl p-5 neu-flat transition-colors space-y-4">
                <h2 class="text-sm font-bold text-gray-500 uppercase tracking-wider">{"Plates Per Side"}</h2>

                { if *target_weight <= *bar_weight {
                    html! {
                        <div class="text-center py-6 text-gray-500 dark:text-gray-400">
                            <div class="text-3xl mb-2">{"🏋️"}</div>
                            <p class="font-medium">{"Bar only — no plates needed"}</p>
                        </div>
                    }
                } else if plates.is_empty() {
                    html! {
                        <div class="text-center py-6 text-gray-500 dark:text-gray-400">
                            <p class="font-medium">{"Bar only"}</p>
                        </div>
                    }
                } else {
                    let per_side_weight = (*target_weight - *bar_weight) / 2.0;
                    html! {
                        <>
                        <div class="text-xs text-gray-500 dark:text-gray-400 mb-3">
                            {format!("{:.1}{} per side", unit_system.display_weight(per_side_weight), wl)}
                        </div>
                        <div class="space-y-2">
                            { for plates.iter().map(|(plate_w, count)| {
                                let plate_display = unit_system.display_weight(*plate_w);
                                // Visual plate sizing: heavier plates are wider
                                let width_pct = (plate_display / unit_system.display_weight(25.0) * 100.0).clamp(30.0, 100.0);
                                html! {
                                    <div class="flex items-center gap-3">
                                        <div
                                            class="h-10 rounded-lg bg-blue-600 flex items-center justify-center text-white font-bold text-sm"
                                            style={format!("width: {}%", width_pct)}
                                        >
                                            {format!("{:.1}{}", plate_display, wl)}
                                        </div>
                                        <span class="text-sm font-bold text-gray-600 dark:text-gray-400">{format!("x{}", count)}</span>
                                    </div>
                                }
                            })}
                        </div>
                        </>
                    }
                }}

                // Summary
                { if *target_weight > *bar_weight {
                    let remainder = (*target_weight - *bar_weight) - plates.iter().map(|(w, c)| w * *c as f64 * 2.0).sum::<f64>();
                    html! {
                        <div class="border-t border-gray-200 dark:border-gray-700 pt-3 mt-3 text-xs text-gray-500 dark:text-gray-400 space-y-1">
                            <div class="flex justify-between">
                                <span>{"Bar"}</span>
                                <span class="font-mono">{format!("{:.1}{}", unit_system.display_weight(*bar_weight), wl)}</span>
                            </div>
                            <div class="flex justify-between">
                                <span>{"Plates (both sides)"}</span>
                                <span class="font-mono">{format!("{:.1}{}", unit_system.display_weight(plates.iter().map(|(w, c)| w * *c as f64 * 2.0).sum::<f64>()), wl)}</span>
                            </div>
                            <div class="flex justify-between font-bold text-gray-700 dark:text-gray-300">
                                <span>{"Total"}</span>
                                <span class="font-mono">{format!("{:.1}{}", unit_system.display_weight(*bar_weight + plates.iter().map(|(w, c)| w * *c as f64 * 2.0).sum::<f64>()), wl)}</span>
                            </div>
                            { if remainder.abs() > 0.01 {
                                html! {
                                    <div class="text-yellow-500 font-bold mt-1">
                                        {format!("Note: {:.1}{} cannot be made with standard plates", unit_system.display_weight(remainder), wl)}
                                    </div>
                                }
                            } else {
                                html! {}
                            }}
                        </div>
                    }
                } else {
                    html! {}
                }}
            </div>
        </div>
    }
}
