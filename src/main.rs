mod backup;
mod components;
mod data;
mod models;
mod pages;
mod sharing;
mod storage;

use wasm_bindgen::JsCast;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::models::Theme;
use components::nav::BottomNav;
use pages::analytics::AnalyticsPage;
use pages::exercises::ExercisesPage;
use pages::faq::FaqPage;
use pages::history::HistoryPage;
use pages::home::HomePage;
use pages::routines::RoutinesPage;
use pages::settings::SettingsPage;
use pages::shared::SharedPage;
use pages::social::SocialPage;
use pages::workout::WorkoutPage;

#[function_component(ThemeManager)]
fn theme_manager() -> Html {
    let route = use_route::<Route>().unwrap_or(Route::Home);

    use_effect_with(route, move |_| {
        let config = storage::load_user_config();
        let document = gloo::utils::document();
        let html = document.document_element().unwrap();

        match config.theme {
            Theme::Dark => {
                let _ = html.set_attribute("class", "dark");
            }
            Theme::Light => {
                let _ = html.set_attribute("class", "");
            }
            Theme::System => {
                let window = gloo::utils::window();
                let is_dark = window
                    .match_media("(prefers-color-scheme: dark)")
                    .unwrap()
                    .unwrap()
                    .matches();
                if is_dark {
                    let _ = html.set_attribute("class", "dark");
                } else {
                    let _ = html.set_attribute("class", "");
                }
            }
            Theme::AmoledBlack => {
                let _ = html.set_attribute("class", "dark amoled");
            }
        }
        || ()
    });
    html! {}
}

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/exercises")]
    Exercises,
    #[at("/workout")]
    Workout,
    #[at("/history")]
    History,
    #[at("/routines")]
    Routines,
    #[at("/settings")]
    Settings,
    #[at("/social")]
    Social,
    #[at("/faq")]
    Faq,
    #[at("/analytics")]
    Analytics,
    #[at("/shared")]
    Shared,
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn switch(routes: Route) -> Html {
    let (key, page) = match routes {
        Route::Home => ("home", html! { <HomePage /> }),
        Route::Exercises => ("exercises", html! { <ExercisesPage /> }),
        Route::Workout => ("workout", html! { <WorkoutPage /> }),
        Route::History => ("history", html! { <HistoryPage /> }),
        Route::Routines => ("routines", html! { <RoutinesPage /> }),
        Route::Settings => ("settings", html! { <SettingsPage /> }),
        Route::Social => ("social", html! { <SocialPage /> }),
        Route::Faq => ("faq", html! { <FaqPage /> }),
        Route::Analytics => ("analytics", html! { <AnalyticsPage /> }),
        Route::Shared => ("shared", html! { <SharedPage /> }),
        Route::NotFound => ("home", html! { <HomePage /> }),
    };
    html! { <div key={key} class="page-enter">{page}</div> }
}

#[function_component(App)]
fn app() -> Html {
    let storage_warning = use_state(|| false);
    let warning_dismissed = use_state(|| false);

    // Startup: request persistent storage, try restore from backup, check save errors
    {
        let storage_warning = storage_warning.clone();
        use_effect_with((), move |_| {
            backup::request_persistent_storage();
            storage::try_restore_from_backup();

            // Check for save failures periodically
            let interval = gloo::timers::callback::Interval::new(2_000, move || {
                if storage::has_save_failed() {
                    storage_warning.set(true);
                }
            });
            move || drop(interval)
        });
    }

    let on_dismiss = {
        let warning_dismissed = warning_dismissed.clone();
        Callback::from(move |_: MouseEvent| {
            storage::clear_save_failed();
            warning_dismissed.set(true);
        })
    };

    html! {
        <HashRouter>
            <ThemeManager />
            <div class="min-h-screen pb-20 flex flex-col bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 transition-colors duration-200">
                if *storage_warning && !*warning_dismissed {
                    <div class="bg-yellow-500 text-black px-4 py-2 text-sm flex items-center justify-between">
                        <span>{"Storage full! Export your data in Settings to avoid data loss."}</span>
                        <button onclick={on_dismiss} class="ml-4 font-bold text-lg leading-none hover:opacity-70">
                            {"\u{00d7}"}
                        </button>
                    </div>
                }
                <div class="flex-grow">
                    <Switch<Route> render={switch} />
                </div>
                <footer class="mt-8 mb-4 px-4 text-center text-gray-500 text-xs space-y-3">
                    <div class="flex justify-center items-center gap-4">
                        <a href="https://github.com/tonybenoy/treening" target="_blank" class="hover:text-blue-400 transition-colors flex items-center gap-1.5">
                            <svg class="w-4 h-4 fill-current" viewBox="0 0 24 24"><path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/></svg>
                            <span>{"GitHub"}</span>
                        </a>
                    </div>
                    <div>
                        {"made with "} <span class="text-red-500">{"❤️"}</span> {" by "}
                        <a href="https://github.com/tonybenoy" target="_blank" class="hover:underline text-blue-400">{"Tony"}</a>
                        {" using "}
                        <a href="https://claude.ai" target="_blank" class="hover:underline text-blue-400">{"Claude Code"}</a>
                        {" & "}
                        <a href="https://gemini.google.com" target="_blank" class="hover:underline text-blue-400">{"Gemini"}</a>
                    </div>
                </footer>
            </div>
            <BottomNav />
        </HashRouter>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());

    // Prevent swipe-back from closing the PWA by keeping a safety history entry.
    // When the user would "leave" the app (popstate with no more back entries),
    // we push them back to the home page instead.
    let window = gloo::utils::window();
    let _ = window.history().unwrap().replace_state_with_url(
        &wasm_bindgen::JsValue::from_str("app"),
        "",
        Some(&window.location().href().unwrap_or_default()),
    );
    let on_popstate = wasm_bindgen::closure::Closure::wrap(Box::new(move |_e: web_sys::Event| {
        let w = gloo::utils::window();
        let hash = w.location().hash().unwrap_or_default();
        // If hash is empty or just "#", user tried to go back past the app root
        if hash.is_empty() || hash == "#" || hash == "#/" {
            let _ = w.history().unwrap().push_state_with_url(
                &wasm_bindgen::JsValue::from_str("app"),
                "",
                Some("/#/"),
            );
        }
    })
        as Box<dyn FnMut(web_sys::Event)>);
    let _ =
        window.add_event_listener_with_callback("popstate", on_popstate.as_ref().unchecked_ref());
    on_popstate.forget();

    yew::Renderer::<App>::new().render();
}
