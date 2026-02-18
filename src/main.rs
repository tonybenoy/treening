mod models;
mod data;
mod storage;
mod components;
mod pages;

use yew::prelude::*;
use yew_router::prelude::*;

use components::nav::BottomNav;
use pages::landing::LandingPage;
use pages::home::HomePage;
use pages::exercises::ExercisesPage;
use pages::workout::WorkoutPage;
use pages::history::HistoryPage;
use pages::routines::RoutinesPage;
use pages::settings::SettingsPage;
use pages::faq::FaqPage;
use pages::analytics::AnalyticsPage;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Landing,
    #[at("/home")]
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
    #[at("/faq")]
    Faq,
    #[at("/analytics")]
    Analytics,
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Landing => html! { <LandingPage /> },
        Route::Home => html! { <HomePage /> },
        Route::Exercises => html! { <ExercisesPage /> },
        Route::Workout => html! { <WorkoutPage /> },
        Route::History => html! { <HistoryPage /> },
        Route::Routines => html! { <RoutinesPage /> },
        Route::Settings => html! { <SettingsPage /> },
        Route::Faq => html! { <FaqPage /> },
        Route::Analytics => html! { <AnalyticsPage /> },
        Route::NotFound => html! { <LandingPage /> },
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <HashRouter>
            <div class="min-h-screen pb-20 flex flex-col">
                <div class="flex-grow">
                    <Switch<Route> render={switch} />
                </div>
                <footer class="mt-8 mb-4 px-4 text-center text-gray-500 text-xs">
                    {"made with "} <span class="text-red-500">{"❤️"}</span> {" by "} 
                    <a href="https://github.com/tonybenoy" target="_blank" class="hover:underline text-blue-400">{"Tony"}</a>
                    {" using "}
                    <a href="https://claude.ai" target="_blank" class="hover:underline text-blue-400">{"Claude Code"}</a>
                    {" & "}
                    <a href="https://gemini.google.com" target="_blank" class="hover:underline text-blue-400">{"Gemini"}</a>
                </footer>
            </div>
            <BottomNav />
        </HashRouter>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
