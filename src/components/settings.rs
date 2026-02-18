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
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            if let Some(files) = input.files() {
                if let Some(file) = files.get(0) {
                    let reader = web_sys::FileReader::new().unwrap();
                    let reader_clone = reader.clone();
                    let status = import_status.clone();
                    let on_done = on_import_complete.clone();

                    let closure = Closure::wrap(Box::new(move || {
                        if let Ok(result) = reader_clone.result() {
                            if let Some(text) = result.as_string() {
                                match storage::import_all_data(&text) {
                                    Ok(()) => {
                                        status.set(Some("Data imported successfully!".to_string()));
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
        <div class="space-y-4">
            <div class="bg-gray-800 rounded-lg p-4">
                <h3 class="font-semibold mb-2">{"Export Data"}</h3>
                <p class="text-sm text-gray-400 mb-3">{"Download all your workout data as a JSON file."}</p>
                <button
                    class="w-full py-2 bg-blue-600 rounded font-medium hover:bg-blue-700"
                    onclick={on_export}
                >{"Export JSON"}</button>
            </div>
            <div class="bg-gray-800 rounded-lg p-4">
                <h3 class="font-semibold mb-2">{"Import Data"}</h3>
                <p class="text-sm text-gray-400 mb-3">{"Restore data from a previously exported JSON file. This will replace current data."}</p>
                <label class="block w-full py-2 bg-gray-700 rounded font-medium text-center cursor-pointer hover:bg-gray-600">
                    {"Choose File"}
                    <input
                        type="file"
                        accept=".json"
                        class="hidden"
                        onchange={on_import}
                    />
                </label>
                { if let Some(status) = &*import_status {
                    html! { <p class="mt-2 text-sm text-green-400">{status}</p> }
                } else { html! {} }}
            </div>
        </div>
    }
}
