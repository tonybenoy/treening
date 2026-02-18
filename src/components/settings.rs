use yew::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use crate::storage;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub on_import_complete: Callback<()>,
}

#[function_component(SettingsPanel)]
pub fn settings_panel(props: &Props) -> Html {
    let import_status = use_state(|| None::<String>);
    let config = use_state(storage::load_user_config);

    let on_toggle_social = {
        let config = config.clone();
        Callback::from(move |_| {
            let mut new_config = (*config).clone();
            new_config.social_enabled = !new_config.social_enabled;
            storage::save_user_config(&new_config);
            config.set(new_config);
        })
    };

    let on_change_theme = {
        let config = config.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlSelectElement = e.target_unchecked_into();
            let mut new_config = (*config).clone();
            new_config.theme = match input.value().as_str() {
                "light" => crate::models::Theme::Light,
                "system" => crate::models::Theme::System,
                _ => crate::models::Theme::Dark,
            };
            storage::save_user_config(&new_config);
            
            // Apply theme immediately for better UX
            let document = gloo::utils::document();
            let html = document.document_element().unwrap();
            match new_config.theme {
                crate::models::Theme::Dark => { let _ = html.set_attribute("class", "dark"); }
                crate::models::Theme::Light => { let _ = html.set_attribute("class", ""); }
                crate::models::Theme::System => {
                    let window = gloo::utils::window();
                    let is_dark = window.match_media("(prefers-color-scheme: dark)").unwrap().unwrap().matches();
                    let _ = html.set_attribute("class", if is_dark { "dark" } else { "" });
                }
            }
            
            config.set(new_config);
        })
    };

    let on_export = Callback::from(|_| {
        let data = storage::export_all_data();
        let blob_parts = js_sys::Array::new();
        blob_parts.push(&wasm_bindgen::JsValue::from_str(&data));
        let opts = web_sys::BlobPropertyBag::new();
        opts.set_type("application/json");
        if let Ok(blob) = web_sys::Blob::new_with_str_sequence_and_options(&blob_parts, &opts) {
            if let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) {
                let document = gloo::utils::document();
                if let Ok(elem) = document.create_element("a") {
                    let anchor: web_sys::HtmlAnchorElement = elem.unchecked_into();
                    anchor.set_href(&url);
                    anchor.set_download("treening-backup.json");
                    anchor.click();
                    let _ = web_sys::Url::revoke_object_url(&url);
                }
            }
        }
    });

    let on_import = {
        let import_status = import_status.clone();
        let on_import_complete = props.on_import_complete.clone();
        let config = config.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            if let Some(files) = input.files() {
                if let Some(file) = files.get(0) {
                    let reader = web_sys::FileReader::new().unwrap();
                    let reader_clone = reader.clone();
                    let status = import_status.clone();
                    let on_done = on_import_complete.clone();
                    let config = config.clone();

                    let closure = Closure::wrap(Box::new(move || {
                        if let Ok(result) = reader_clone.result() {
                            if let Some(text) = result.as_string() {
                                match storage::import_all_data(&text) {
                                    Ok(()) => {
                                        status.set(Some("Data imported successfully!".to_string()));
                                        config.set(storage::load_user_config());
                                        on_done.emit(());
                                    }
                                    Err(err) => {
                                        status.set(Some(format!("Import error: {}", err)));
                                    }
                                }
                            }
                        }
                    }) as Box<dyn Fn()>);

                    reader.set_onload(Some(closure.as_ref().unchecked_ref()));
                    closure.forget();
                    let _ = reader.read_as_text(&file);
                }
            }
        })
    };

    html! {
        <div class="space-y-4 transition-colors duration-200">
            <div class="bg-gray-100 dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-transparent">
                <h3 class="font-semibold mb-2 text-gray-900 dark:text-gray-100">{"Features"}</h3>
                <div class="flex items-center justify-between">
                    <div>
                        <div class="font-medium text-gray-800 dark:text-gray-200">{"Social Features"}</div>
                        <div class="text-sm text-gray-500 dark:text-gray-400">{"Enable friend syncing and leaderboards"}</div>
                    </div>
                    <button 
                        onclick={on_toggle_social}
                        class={classes!(
                            "relative", "inline-flex", "h-6", "w-11", "items-center", "rounded-full", "transition-colors", "focus:outline-none",
                            if config.social_enabled { "bg-blue-600" } else { "bg-gray-300 dark:bg-gray-700" }
                        )}
                    >
                        <span
                            class={classes!(
                                "inline-block", "h-4", "w-4", "transform", "rounded-full", "bg-white", "transition-transform",
                                if config.social_enabled { "translate-x-6" } else { "translate-x-1" }
                            )}
                        />
                    </button>
                </div>

                <div class="pt-4 mt-4 border-t border-gray-200 dark:border-gray-700/50 flex items-center justify-between">
                    <div>
                        <div class="font-medium text-gray-800 dark:text-gray-200">{"App Theme"}</div>
                        <div class="text-sm text-gray-500 dark:text-gray-400">{"Choose your preferred appearance"}</div>
                    </div>
                    <select 
                        onchange={on_change_theme}
                        class="bg-white dark:bg-gray-700 text-gray-900 dark:text-white text-sm rounded-lg px-2 py-1 outline-none border border-gray-300 dark:border-transparent focus:ring-1 focus:ring-blue-500"
                    >
                        <option value="dark" selected={config.theme == crate::models::Theme::Dark}>{"Dark"}</option>
                        <option value="light" selected={config.theme == crate::models::Theme::Light}>{"Light"}</option>
                        <option value="system" selected={config.theme == crate::models::Theme::System}>{"System"}</option>
                    </select>
                </div>


                <p class="text-[10px] text-gray-500 dark:text-gray-500 mt-3 italic">
                    { if config.social_enabled {
                        "Note: Routine link is hidden when Social is enabled to save space."
                    } else {
                        "Note: Routine link is shown when Social is disabled."
                    }}
                </p>
            </div>

            <div class="bg-gray-100 dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-transparent">
                <h3 class="font-semibold mb-2 text-gray-900 dark:text-gray-100">{"Export Data"}</h3>
                <p class="text-sm text-gray-500 dark:text-gray-400 mb-3">{"Download all your workout data as a JSON file."}</p>
                <button
                    class="w-full py-2 bg-blue-600 text-white rounded font-medium hover:bg-blue-700 shadow-sm transition-colors"
                    onclick={on_export}
                >{"Export JSON"}</button>
                <button
                    class="w-full py-2 mt-2 bg-green-600 text-white rounded font-medium hover:bg-green-700 shadow-sm transition-colors"
                    onclick={Callback::from(|_| {
                        let data = storage::export_csv();
                        let blob_parts = js_sys::Array::new();
                        blob_parts.push(&wasm_bindgen::JsValue::from_str(&data));
                        let opts = web_sys::BlobPropertyBag::new();
                        opts.set_type("text/csv");
                        if let Ok(blob) = web_sys::Blob::new_with_str_sequence_and_options(&blob_parts, &opts) {
                            if let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) {
                                let document = gloo::utils::document();
                                if let Ok(elem) = document.create_element("a") {
                                    let anchor: web_sys::HtmlAnchorElement = elem.unchecked_into();
                                    anchor.set_href(&url);
                                    anchor.set_download("treening-export.csv");
                                    anchor.click();
                                    let _ = web_sys::Url::revoke_object_url(&url);
                                }
                            }
                        }
                    })}
                >{"Export CSV"}</button>
            </div>
            <div class="bg-gray-100 dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-transparent">
                <h3 class="font-semibold mb-2 text-gray-900 dark:text-gray-100">{"Import Data"}</h3>
                <p class="text-sm text-gray-500 dark:text-gray-400 mb-3">{"Restore data from a previously exported JSON file. This will replace current data."}</p>
                <label class="block w-full py-2 bg-white dark:bg-gray-700 text-gray-800 dark:text-gray-200 border border-gray-300 dark:border-transparent rounded font-medium text-center cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-600 transition-colors">
                    {"Choose File"}
                    <input
                        type="file"
                        accept=".json"
                        class="hidden"
                        onchange={on_import}
                    />
                </label>
                { if let Some(status) = &*import_status {
                    html! { <p class="mt-2 text-sm text-green-600 dark:text-green-400 font-medium">{status}</p> }
                } else { html! {} }}
            </div>
        </div>
    }
}
