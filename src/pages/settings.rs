use crate::components::custom_exercise::CustomExerciseForm;
use crate::components::settings::SettingsPanel;
use crate::components::sync::SyncPanel;
use crate::models::{BodyMetric, Exercise, UnitSystem};
use crate::storage;
use crate::Route;
use wasm_bindgen::prelude::*;
use yew::prelude::*;
use yew_router::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = canInstallApp)]
    fn can_install_app() -> bool;

    #[wasm_bindgen(js_name = isAppStandalone)]
    fn is_app_standalone() -> bool;

    #[wasm_bindgen(js_name = triggerInstallPrompt)]
    fn trigger_install_prompt() -> js_sys::Promise;

    #[wasm_bindgen(js_name = isIOS)]
    fn is_ios() -> bool;

    #[wasm_bindgen(js_name = getAppVersion)]
    fn get_app_version() -> js_sys::Promise;

    #[wasm_bindgen(js_name = getBuildDate)]
    fn get_build_date() -> js_sys::Promise;
}

#[function_component(InstallButton)]
fn install_button() -> Html {
    let can_install = use_state(can_install_app);
    let is_standalone = is_app_standalone();

    // Re-check periodically in case beforeinstallprompt fires after mount
    {
        let can_install = can_install.clone();
        use_effect_with((), move |_| {
            let cb = Closure::<dyn Fn()>::new(move || {
                can_install.set(can_install_app());
            });
            let window = web_sys::window().unwrap();
            let id = window
                .set_interval_with_callback_and_timeout_and_arguments_0(
                    cb.as_ref().unchecked_ref(),
                    1000,
                )
                .unwrap();
            cb.forget();
            move || {
                web_sys::window().unwrap().clear_interval_with_handle(id);
            }
        });
    }

    if is_standalone {
        return html! {};
    }

    // iOS doesn't support beforeinstallprompt — show manual instructions
    if is_ios() && !*can_install {
        return html! {
            <div class="w-full py-3 px-4 bg-blue-50 dark:bg-blue-900/30 rounded-xl text-sm neu-flat">
                <div class="font-bold mb-1 text-blue-900 dark:text-blue-100">{"Install Treening App"}</div>
                <div class="text-xs text-blue-700 dark:text-blue-300">
                    {"Tap the "}
                    <span class="inline-block align-middle">
                        <svg class="inline w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
                        </svg>
                    </span>
                    {" Share button at the bottom of Safari, then tap \"Add to Home Screen\""}
                </div>
            </div>
        };
    }

    let onclick = {
        Callback::from(move |_| {
            if can_install_app() {
                let _ = trigger_install_prompt();
            }
        })
    };

    html! {
        <button
            onclick={onclick}
            disabled={!*can_install}
            class={if *can_install {
                "w-full py-3 bg-gradient-to-r from-blue-600 to-blue-500 text-white rounded-xl font-bold text-sm neu-btn hover:from-blue-700 hover:to-blue-600 transition-all"
            } else {
                "w-full py-3 bg-gray-300 dark:bg-gray-700 text-gray-500 dark:text-gray-400 rounded-xl font-bold text-sm cursor-not-allowed"
            }}
        >
            { if *can_install { "Install Treening App" } else { "Open in browser to install" } }
        </button>
    }
}

#[function_component(ProfileSection)]
fn profile_section() -> Html {
    let config = use_state(storage::load_user_config);
    let nickname = use_state(|| config.nickname.clone());
    let height = use_state(|| {
        config
            .height
            .map(|h| config.unit_system.display_height(h).to_string())
            .unwrap_or_default()
    });
    let birth_date = use_state(|| config.birth_date.clone().unwrap_or_default());
    let gender = use_state(|| config.gender.clone().unwrap_or_default());
    let rest_seconds = use_state(|| config.rest_seconds.to_string());
    let unit_system = use_state(|| config.unit_system.clone());
    let bar_weight = use_state(|| {
        let bw = config.unit_system.display_weight(config.bar_weight);
        format!("{:.1}", bw)
    });

    let has_changes = {
        let c = &*config;
        *nickname != c.nickname
            || *height
                != c.height
                    .map(|h| c.unit_system.display_height(h).to_string())
                    .unwrap_or_default()
            || *birth_date != c.birth_date.clone().unwrap_or_default()
            || *gender != c.gender.clone().unwrap_or_default()
            || *rest_seconds != c.rest_seconds.to_string()
            || *unit_system != c.unit_system
            || *bar_weight != format!("{:.1}", c.unit_system.display_weight(c.bar_weight))
    };

    let on_save = {
        let config_state = config.clone();
        let nickname = nickname.clone();
        let height = height.clone();
        let birth_date = birth_date.clone();
        let gender = gender.clone();
        let rest_seconds = rest_seconds.clone();
        let bar_weight = bar_weight.clone();
        let unit_system = unit_system.clone();
        Callback::from(move |_| {
            let mut new_config = (*config_state).clone();
            new_config.nickname = (*nickname).clone();
            new_config.unit_system = (*unit_system).clone();
            new_config.height = height.parse::<f64>().ok().map(|h| unit_system.to_cm(h));
            new_config.birth_date = Some((*birth_date).clone()).filter(|s| !s.is_empty());
            new_config.gender = Some((*gender).clone()).filter(|s| !s.is_empty());
            new_config.rest_seconds = rest_seconds.parse::<u32>().unwrap_or(90);
            new_config.bar_weight = unit_system.to_kg(bar_weight.parse::<f64>().unwrap_or(20.0));
            storage::save_user_config(&new_config);
            config_state.set(new_config);
        })
    };

    html! {
        <div class="bg-gray-100 dark:bg-gray-800 rounded-2xl p-4 space-y-4 neu-flat transition-colors">
            <h2 class="text-lg font-bold text-gray-900 dark:text-gray-100">{"Personal Profile"}</h2>
            <div class="grid grid-cols-2 gap-4">
                <div class="col-span-2">
                    <label class="block text-[10px] uppercase font-bold text-gray-500 mb-1">{"Nickname"}</label>
                    <input
                        type="text"
                        class="w-full bg-white dark:bg-gray-700 rounded-lg px-3 py-2 text-sm text-gray-900 dark:text-white outline-none neu-pressed"
                        value={(*nickname).clone()}
                        oninput={let n = nickname.clone(); Callback::from(move |e: InputEvent| n.set(e.target_unchecked_into::<web_sys::HtmlInputElement>().value()))}
                    />
                </div>
                <div class="col-span-2">
                    <label class="block text-[10px] uppercase font-bold text-gray-500 mb-1">{"Units"}</label>
                    <select
                        class="w-full bg-white dark:bg-gray-700 rounded-lg px-3 py-2 text-sm text-gray-900 dark:text-white outline-none neu-pressed"
                        onchange={let u = unit_system.clone(); Callback::from(move |e: Event| {
                            let val = e.target_unchecked_into::<web_sys::HtmlSelectElement>().value();
                            u.set(if val == "Imperial" { UnitSystem::Imperial } else { UnitSystem::Metric });
                        })}
                    >
                        <option value="Metric" selected={*unit_system == UnitSystem::Metric}>{"Metric (kg, km, cm)"}</option>
                        <option value="Imperial" selected={*unit_system == UnitSystem::Imperial}>{"Imperial (lbs, mi, in)"}</option>
                    </select>
                </div>
                <div>
                    <label class="block text-[10px] uppercase font-bold text-gray-500 mb-1">{format!("Height ({})", unit_system.height_label())}</label>
                    <input
                        type="number" autocomplete="off"
                        class="w-full bg-white dark:bg-gray-700 rounded-lg px-3 py-2 text-sm text-gray-900 dark:text-white outline-none neu-pressed"
                        value={(*height).clone()}
                        onchange={let h = height.clone(); Callback::from(move |e: Event| h.set(e.target_unchecked_into::<web_sys::HtmlInputElement>().value()))}
                    />
                </div>
                <div>
                    <label class="block text-[10px] uppercase font-bold text-gray-500 mb-1">{"Gender"}</label>
                    <select
                        class="w-full bg-white dark:bg-gray-700 rounded-lg px-3 py-2 text-sm text-gray-900 dark:text-white outline-none neu-pressed"
                        onchange={let g = gender.clone(); Callback::from(move |e: Event| g.set(e.target_unchecked_into::<web_sys::HtmlSelectElement>().value()))}
                    >
                        <option value="" selected={gender.is_empty()}>{"Select..."}</option>
                        <option value="Male" selected={*gender == "Male"}>{"Male"}</option>
                        <option value="Female" selected={*gender == "Female"}>{"Female"}</option>
                        <option value="Other" selected={*gender == "Other"}>{"Other"}</option>
                    </select>
                </div>
                <div class="col-span-2">
                    <label class="block text-[10px] uppercase font-bold text-gray-500 mb-1">{"Birth Date"}</label>
                    <input
                        type="date"
                        class="w-full bg-white dark:bg-gray-700 rounded-lg px-3 py-2 text-sm text-gray-900 dark:text-white outline-none neu-pressed"
                        value={(*birth_date).clone()}
                        oninput={let b = birth_date.clone(); Callback::from(move |e: InputEvent| b.set(e.target_unchecked_into::<web_sys::HtmlInputElement>().value()))}
                    />
                </div>
                <div>
                    <label class="block text-[10px] uppercase font-bold text-gray-500 mb-1">{"Rest Timer (sec)"}</label>
                    <input
                        type="number" autocomplete="off"
                        class="w-full bg-white dark:bg-gray-700 rounded-lg px-3 py-2 text-sm text-gray-900 dark:text-white outline-none neu-pressed"
                        value={(*rest_seconds).clone()}
                        onchange={let r = rest_seconds.clone(); Callback::from(move |e: Event| r.set(e.target_unchecked_into::<web_sys::HtmlInputElement>().value()))}
                    />
                    <p class="text-[10px] text-gray-400 mt-0.5">{"Countdown after completing a set"}</p>
                </div>
                <div>
                    <label class="block text-[10px] uppercase font-bold text-gray-500 mb-1">{format!("Bar Weight ({})", unit_system.weight_label())}</label>
                    <input
                        type="number" step="0.5" autocomplete="off"
                        class="w-full bg-white dark:bg-gray-700 rounded-lg px-3 py-2 text-sm text-gray-900 dark:text-white outline-none neu-pressed"
                        value={(*bar_weight).clone()}
                        onchange={let bw = bar_weight.clone(); Callback::from(move |e: Event| bw.set(e.target_unchecked_into::<web_sys::HtmlInputElement>().value()))}
                    />
                    <p class="text-[10px] text-gray-400 mt-0.5">{"Empty barbell weight, used for warm-ups and plate calculator"}</p>
                </div>
            </div>
            <button
                onclick={on_save}
                disabled={!has_changes}
                class={if has_changes {
                    "w-full py-2.5 bg-blue-600 text-white rounded-lg font-bold text-sm hover:bg-blue-700 neu-btn transition-colors"
                } else {
                    "w-full py-2.5 bg-gray-300 dark:bg-gray-700 text-gray-500 dark:text-gray-400 rounded-lg font-bold text-sm cursor-not-allowed transition-colors"
                }}
            >{"Update Profile"}</button>
        </div>
    }
}

#[function_component(BodyMetricsSection)]
fn body_metrics_section() -> Html {
    let metrics = use_state(storage::load_body_metrics);
    let weight = use_state(String::new);
    let body_fat = use_state(String::new);
    let show_form = use_state(|| false);
    let units = storage::load_user_config().unit_system;

    let on_add = {
        let metrics_state = metrics.clone();
        let weight = weight.clone();
        let body_fat = body_fat.clone();
        let show = show_form.clone();
        let units = units.clone();
        Callback::from(move |_| {
            if weight.is_empty() {
                return;
            }
            let mut new_metrics = (*metrics_state).clone();
            new_metrics.push(BodyMetric {
                id: uuid::Uuid::new_v4().to_string(),
                date: chrono::Local::now().format("%Y-%m-%d").to_string(),
                weight: weight.parse::<f64>().ok().map(|w| units.to_kg(w)),
                body_fat: body_fat.parse().ok(),
            });
            storage::save_body_metrics(&new_metrics);
            metrics_state.set(new_metrics);
            weight.set(String::new());
            body_fat.set(String::new());
            show.set(false);
        })
    };

    let on_delete = {
        let metrics_state = metrics.clone();
        Callback::from(move |id: String| {
            let mut new_metrics = (*metrics_state).clone();
            new_metrics.retain(|m| m.id != id);
            storage::save_body_metrics(&new_metrics);
            metrics_state.set(new_metrics);
        })
    };

    let mut sorted_metrics = (*metrics).clone();
    sorted_metrics.sort_by(|a, b| b.date.cmp(&a.date));

    html! {
        <div class="space-y-4">
            <div class="flex justify-between items-center px-1 transition-colors">
                <h2 class="text-lg font-bold text-gray-900 dark:text-gray-100">{"Body Progress"}</h2>
                <button
                    onclick={let s = show_form.clone(); Callback::from(move |_| s.set(!*s))}
                    class="text-xs font-bold text-blue-600 dark:text-blue-400 hover:underline"
                >{if *show_form { "Cancel" } else { "+ Log Weight" }}</button>
            </div>

            { if *show_form {
                html! {
                    <div class="bg-gray-100 dark:bg-gray-800 rounded-xl p-4 border border-blue-500/30 space-y-3 neu-flat transition-colors">
                        <div class="grid grid-cols-2 gap-3">
                            <div>
                                <label class="block text-[10px] uppercase font-bold text-gray-500 mb-1">{format!("Weight ({})", units.weight_label())}</label>
                                <input
                                    type="number" step="0.1"
                                    class="w-full bg-white dark:bg-gray-700 rounded-lg px-3 py-2 text-sm text-gray-900 dark:text-white outline-none neu-pressed"
                                    value={(*weight).clone()}
                                    onchange={let w = weight.clone(); Callback::from(move |e: Event| w.set(e.target_unchecked_into::<web_sys::HtmlInputElement>().value()))}
                                />
                            </div>
                            <div>
                                <label class="block text-[10px] uppercase font-bold text-gray-500 mb-1">{"Body Fat %"}</label>
                                <input
                                    type="number" step="0.1"
                                    class="w-full bg-white dark:bg-gray-700 rounded-lg px-3 py-2 text-sm text-gray-900 dark:text-white outline-none neu-pressed"
                                    value={(*body_fat).clone()}
                                    onchange={let bf = body_fat.clone(); Callback::from(move |e: Event| bf.set(e.target_unchecked_into::<web_sys::HtmlInputElement>().value()))}
                                />
                            </div>
                        </div>
                        <button
                            onclick={on_add}
                            class="w-full py-2 bg-blue-600 text-white rounded-lg font-bold text-sm neu-btn hover:bg-blue-700 transition-colors"
                        >{"Save Measurement"}</button>
                    </div>
                }
            } else { html! {} }}

            <div class="space-y-2">
                { for sorted_metrics.iter().take(3).map(|m| {
                    let id = m.id.clone();
                    let on_del = on_delete.clone();
                    html! {
                        <div class="bg-gray-100 dark:bg-gray-800 rounded-xl p-3 flex justify-between items-center neu-flat transition-colors">
                            <div>
                                <div class="text-sm font-bold text-gray-900 dark:text-gray-100">
                                    {m.weight.map(|w| format!("{:.1} {}", units.display_weight(w), units.weight_label())).unwrap_or_else(|| "--".to_string())}
                                    {m.body_fat.map(|bf| format!(" • {}% fat", bf)).unwrap_or_default()}
                                </div>
                                <div class="text-[10px] text-gray-500 dark:text-gray-500 font-mono uppercase tracking-wider">{&m.date}</div>
                            </div>
                            <button
                                onclick={Callback::from(move |_| on_del.emit(id.clone()))}
                                class="text-gray-400 hover:text-red-500 p-1 transition-colors"
                            >{"\u{1f5d1}"}</button>
                        </div>
                    }
                })}
                { if sorted_metrics.is_empty() && !*show_form {
                    html! { <p class="text-center py-8 text-gray-500 text-xs italic bg-gray-50 dark:bg-gray-800/20 rounded-xl border border-dashed border-gray-200 dark:border-gray-700 transition-colors">{"No measurements yet."}</p> }
                } else { html! {} }}
            </div>
        </div>
    }
}

#[function_component(SettingsPage)]
pub fn settings_page() -> Html {
    let custom_exercises = use_state(storage::load_custom_exercises);
    let show_custom_form = use_state(|| false);
    let reload_trigger = use_state(|| 0u32);

    let on_import_complete = {
        let r = reload_trigger.clone();
        let ce = custom_exercises.clone();
        Callback::from(move |_| {
            ce.set(storage::load_custom_exercises());
            r.set(*r + 1);
        })
    };

    let on_save_custom = {
        let ce = custom_exercises.clone();
        let show = show_custom_form.clone();
        Callback::from(move |ex: Exercise| {
            let mut exs = (*ce).clone();
            exs.push(ex);
            storage::save_custom_exercises(&exs);
            ce.set(exs);
            show.set(false);
        })
    };

    let on_delete_custom = {
        let ce = custom_exercises.clone();
        Callback::from(move |id: String| {
            let mut exs = (*ce).clone();
            exs.retain(|e| e.id != id);
            storage::save_custom_exercises(&exs);
            ce.set(exs);
        })
    };

    let app_version = use_state(|| String::from("..."));
    let build_date = use_state(String::new);
    {
        let app_version = app_version.clone();
        let build_date = build_date.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let promise = get_app_version();
                if let Ok(val) = wasm_bindgen_futures::JsFuture::from(promise).await {
                    if let Some(s) = val.as_string() {
                        app_version.set(s);
                    }
                }
                let promise = get_build_date();
                if let Ok(val) = wasm_bindgen_futures::JsFuture::from(promise).await {
                    if let Some(s) = val.as_string() {
                        if !s.is_empty() && !s.contains("__") {
                            build_date.set(s);
                        }
                    }
                }
            });
        });
    }

    html! {
        <div class="px-4 py-4 pb-20 space-y-8 transition-colors duration-200">
            <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">{"Settings"}</h1>

            <InstallButton />

            <ProfileSection />

            <BodyMetricsSection />

            <SyncPanel />

            <SettingsPanel on_import_complete={on_import_complete} />

            <div class="bg-gray-100 dark:bg-gray-800 rounded-xl p-4 neu-flat transition-colors">
                <h3 class="font-semibold mb-3 text-gray-900 dark:text-gray-100">{"App Info"}</h3>
                <Link<Route> to={Route::Faq} classes="flex items-center justify-between py-2 text-blue-600 dark:text-blue-400 hover:text-blue-500 dark:hover:text-blue-300">
                    <span>{"Frequently Asked Questions"}</span>
                    <span>{"→"}</span>
                </Link<Route>>
                <div class="pt-2 border-t border-gray-200 dark:border-gray-700 mt-2 space-y-1">
                    <div class="flex items-center gap-1">
                        <span class="text-xs text-gray-500 dark:text-gray-400">{"Version: "}</span>
                        <a href={format!("https://github.com/tonybenoy/treening/commit/{}", *app_version)}
                           target="_blank" rel="noopener noreferrer"
                           class="text-xs text-blue-600 dark:text-blue-400 hover:underline font-mono">
                            {(*app_version).clone()}
                        </a>
                        { if !build_date.is_empty() {
                            html! { <span class="text-xs text-gray-500 dark:text-gray-400">{" · "}{(*build_date).clone()}</span> }
                        } else { html! {} }}
                    </div>
                    <a href="https://github.com/tonybenoy/treening/releases"
                       target="_blank" rel="noopener noreferrer"
                       class="text-xs text-blue-600 dark:text-blue-400 hover:underline block">
                        {"View Release Notes →"}
                    </a>
                </div>
            </div>

            <div>
                <div class="flex justify-between items-center mb-3 px-1">
                    <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">{"Custom Exercises"}</h2>
                    <button
                        class="px-3 py-1.5 bg-blue-600 text-white rounded-lg text-sm hover:bg-blue-700 neu-btn transition-colors"
                        onclick={let s = show_custom_form.clone(); Callback::from(move |_| s.set(true))}
                    >{"+ Add"}</button>
                </div>

                { if *show_custom_form {
                    html! {
                        <CustomExerciseForm
                            on_save={on_save_custom}
                            on_cancel={let s = show_custom_form.clone(); Callback::from(move |_| s.set(false))}
                        />
                    }
                } else { html! {} }}

                <div class="space-y-2">
                    { for custom_exercises.iter().map(|ex| {
                        let on_delete = on_delete_custom.clone();
                        let eid = ex.id.clone();
                        html! {
                            <div class="bg-gray-100 dark:bg-gray-800 rounded-xl p-3 flex justify-between items-center neu-flat transition-colors">
                                <div>
                                    <div class="font-medium text-gray-800 dark:text-gray-200">{&ex.name}</div>
                                    <div class="text-sm text-gray-500 dark:text-gray-400">
                                        {ex.category.to_string()}{" · "}{ex.equipment.to_string()}
                                    </div>
                                </div>
                                <button
                                    class="text-red-600 dark:text-red-400 text-sm hover:text-red-500 dark:hover:text-red-300"
                                    onclick={Callback::from(move |_| on_delete.emit(eid.clone()))}
                                >{"Delete"}</button>
                            </div>
                        }
                    })}
                    { if custom_exercises.is_empty() && !*show_custom_form {
                        html! { <p class="text-gray-500 text-sm">{"No custom exercises yet."}</p> }
                    } else { html! {} }}
                </div>
            </div>

            <div class="text-center py-4">
                <a href="https://buymeacoffee.com/tonybenoy" target="_blank" rel="noopener noreferrer"
                   class="inline-flex items-center gap-2 px-4 py-2 bg-yellow-400 hover:bg-yellow-500 text-gray-900 font-semibold rounded-lg neu-btn transition-colors text-sm">
                    <span>{"☕"}</span>
                    <span>{"Buy Me a Coffee"}</span>
                </a>
            </div>

            <details class="group bg-gray-100 dark:bg-gray-800 rounded-2xl neu-flat transition-colors">
                <summary class="px-4 py-4 cursor-pointer font-semibold text-gray-900 dark:text-gray-100 list-none flex justify-between items-center">
                    {"About Treening"}
                    <span class="text-gray-400 group-open:rotate-180 transition-transform">{"\u{25be}"}</span>
                </summary>
                <div class="px-4 pb-4 space-y-3 border-t border-gray-200 dark:border-gray-700 pt-3 mt-1">
                    <p class="text-sm text-gray-600 dark:text-gray-400 leading-relaxed">
                        {"Treening was born out of personal need. After losing 20 kg and hitting a weight loss plateau, I started hitting the gym \u{2014} but couldn't find a single workout tracker that was both free and subscription-free. So I built one myself in a day using Claude Code and Gemini."}
                    </p>
                    <p class="text-sm text-gray-600 dark:text-gray-400 leading-relaxed">
                        {"This is a side project built to scratch my own itch, so I may not be able to address feature requests or bug reports right away \u{2014} but I'll do my best. The easiest way to reach out is by opening an issue on GitHub:"}
                    </p>
                    <a href="https://github.com/tonybenoy/treening/issues" target="_blank" rel="noopener noreferrer"
                       class="inline-flex items-center gap-1 text-sm text-blue-600 dark:text-blue-400 hover:underline font-medium">
                        {"github.com/tonybenoy/treening/issues"}
                        <span>{"→"}</span>
                    </a>
                    <p class="text-sm text-gray-600 dark:text-gray-400 leading-relaxed">
                        {"Even better \u{2014} if you can fix the issue yourself and submit a pull request, that would be amazing! Contributions are always welcome."}
                    </p>
                </div>
            </details>
        </div>
    }
}
