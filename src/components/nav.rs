use yew::prelude::*;
use yew_router::prelude::*;
use crate::Route;

#[function_component(BottomNav)]
pub fn bottom_nav() -> Html {
    let route: Route = use_route().unwrap_or(Route::Landing);

    let nav_item = |r: Route, label: &str, icon: &str| {
        let active = route == r;
        let cls = if active {
            "flex flex-col items-center text-blue-400"
        } else {
            "flex flex-col items-center text-gray-500 hover:text-gray-300"
        };
        html! {
            <Link<Route> to={r} classes={classes!(cls)}>
                <span class="text-xl">{icon}</span>
                <span class="text-xs mt-0.5">{label}</span>
            </Link<Route>>
        }
    };

    html! {
        <nav class="fixed bottom-0 left-0 right-0 bg-gray-800 border-t border-gray-700 px-2 py-2 flex justify-around items-center z-50 safe-bottom">
            {nav_item(Route::Home, "Home", "\u{1f3e0}")}
            {nav_item(Route::Exercises, "Exercises", "\u{1f4aa}")}
            {nav_item(Route::Workout, "Workout", "\u{1f3cb}")}
            {nav_item(Route::Routines, "Routines", "\u{1f4cb}")}
            {nav_item(Route::History, "History", "\u{1f4c5}")}
            {nav_item(Route::Analytics, "Analytics", "\u{1f4ca}")}
            {nav_item(Route::Settings, "Settings", "\u{2699}")}
        </nav>
    }
}
