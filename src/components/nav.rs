use crate::Route;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(BottomNav)]
pub fn bottom_nav() -> Html {
    let route: Route = use_route().unwrap_or(Route::Home);
    let navigator = use_navigator().unwrap();

    let nav_item = |r: Route, label: &str, icon: &str| {
        let active = route == r;
        let cls = if active {
            "flex flex-col items-center text-blue-600 dark:text-blue-400 transition-all duration-150 scale-110"
        } else {
            "flex flex-col items-center text-gray-500 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200 transition-all duration-150 scale-100"
        };
        let nav = navigator.clone();
        let onclick = Callback::from(move |_: MouseEvent| {
            nav.replace(&r);
        });
        html! {
            <button class={classes!(cls, "btn-press")} {onclick}>
                <span class="text-xl">{icon}</span>
                <span class="text-xs mt-0.5">{label}</span>
            </button>
        }
    };

    html! {
        <nav class="fixed bottom-0 left-0 right-0 bg-white dark:bg-gray-800 border-t border-gray-200 dark:border-gray-700 px-1 py-2 flex justify-around items-center z-50 safe-bottom transition-colors duration-200">
            {nav_item(Route::Home, "Home", "🏠")}
            {nav_item(Route::Exercises, "Exercises", "💪")}
            {nav_item(Route::Routines, "Routines", "📋")}
            {nav_item(Route::Workout, "Log", "🏋️")}
            {nav_item(Route::History, "History", "📅")}
            {nav_item(Route::Settings, "Settings", "⚙️")}
        </nav>
    }
}
