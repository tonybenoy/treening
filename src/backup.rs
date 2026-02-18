use wasm_bindgen::prelude::*;
use wasm_bindgen::closure::Closure;
use web_sys::{IdbOpenDbRequest, IdbDatabase, IdbTransactionMode, IdbRequest};

const DB_NAME: &str = "treening_backup";
const STORE_NAME: &str = "app_data";
const BACKUP_KEY: &str = "backup";

fn open_db(on_success: impl FnOnce(IdbDatabase) + 'static) {
    let window = gloo::utils::window();
    let idb_factory = window
        .indexed_db()
        .ok()
        .flatten();

    let idb_factory = match idb_factory {
        Some(f) => f,
        None => {
            log::warn!("IndexedDB not available");
            return;
        }
    };

    let open_req: IdbOpenDbRequest = match idb_factory.open_with_u32(DB_NAME, 1) {
        Ok(r) => r,
        Err(_) => {
            log::warn!("Failed to open IndexedDB");
            return;
        }
    };

    // Create object store on upgrade
    let on_upgrade = Closure::once(Box::new(move |event: web_sys::Event| {
        let target = event.target().unwrap();
        let req: IdbOpenDbRequest = target.unchecked_into();
        let db: IdbDatabase = req.result().unwrap().unchecked_into();
        if !db.object_store_names().contains(STORE_NAME) {
            let _ = db.create_object_store(STORE_NAME);
        }
    }) as Box<dyn FnOnce(_)>);
    open_req.set_onupgradeneeded(Some(on_upgrade.as_ref().unchecked_ref()));
    on_upgrade.forget();

    let on_success_cb = Closure::once(Box::new(move |event: web_sys::Event| {
        let target = event.target().unwrap();
        let req: IdbOpenDbRequest = target.unchecked_into();
        let db: IdbDatabase = req.result().unwrap().unchecked_into();
        on_success(db);
    }) as Box<dyn FnOnce(_)>);
    open_req.set_onsuccess(Some(on_success_cb.as_ref().unchecked_ref()));
    on_success_cb.forget();

    let on_error = Closure::once(Box::new(move |_event: web_sys::Event| {
        log::warn!("Failed to open IndexedDB database");
    }) as Box<dyn FnOnce(_)>);
    open_req.set_onerror(Some(on_error.as_ref().unchecked_ref()));
    on_error.forget();
}

pub fn save_backup(data: &str) {
    let data = data.to_string();
    open_db(move |db: IdbDatabase| {
        let tx = match db.transaction_with_str_and_mode(STORE_NAME, IdbTransactionMode::Readwrite) {
            Ok(t) => t,
            Err(_) => return,
        };
        let store = match tx.object_store(STORE_NAME) {
            Ok(s) => s,
            Err(_) => return,
        };
        let _ = store.put_with_key(
            &JsValue::from_str(&data),
            &JsValue::from_str(BACKUP_KEY),
        );
    });
}

pub fn load_backup(on_loaded: impl FnOnce(Option<String>) + 'static) {
    open_db(move |db: IdbDatabase| {
        let tx = match db.transaction_with_str_and_mode(STORE_NAME, IdbTransactionMode::Readonly) {
            Ok(t) => t,
            Err(_) => {
                on_loaded(None);
                return;
            }
        };
        let store = match tx.object_store(STORE_NAME) {
            Ok(s) => s,
            Err(_) => {
                on_loaded(None);
                return;
            }
        };
        let get_req: IdbRequest = match store.get(&JsValue::from_str(BACKUP_KEY)) {
            Ok(r) => r,
            Err(_) => {
                on_loaded(None);
                return;
            }
        };

        let on_success = Closure::once(Box::new(move |event: web_sys::Event| {
            let target = event.target().unwrap();
            let req: IdbRequest = target.unchecked_into();
            let result = req.result().unwrap_or(JsValue::UNDEFINED);
            if result.is_undefined() || result.is_null() {
                on_loaded(None);
            } else {
                on_loaded(result.as_string());
            }
        }) as Box<dyn FnOnce(_)>);
        get_req.set_onsuccess(Some(on_success.as_ref().unchecked_ref()));
        on_success.forget();

        let on_error = Closure::once(Box::new(move |_event: web_sys::Event| {
            log::warn!("Failed to read backup from IndexedDB");
        }) as Box<dyn FnOnce(_)>);
        get_req.set_onerror(Some(on_error.as_ref().unchecked_ref()));
        on_error.forget();
    });
}

pub fn request_persistent_storage() {
    let window = gloo::utils::window();
    let navigator = window.navigator();
    let storage = navigator.storage();
    {
        let promise = match storage.persist() {
            Ok(p) => p,
            Err(_) => {
                log::warn!("Failed to call storage.persist()");
                return;
            }
        };
        let future = wasm_bindgen_futures::JsFuture::from(promise);
        wasm_bindgen_futures::spawn_local(async move {
            match future.await {
                Ok(val) => {
                    let granted = val.as_bool().unwrap_or(false);
                    if granted {
                        log::info!("Persistent storage granted");
                    } else {
                        log::info!("Persistent storage not granted");
                    }
                }
                Err(_) => {
                    log::warn!("Failed to request persistent storage");
                }
            }
        });
    }
}
