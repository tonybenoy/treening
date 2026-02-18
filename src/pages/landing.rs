use yew::prelude::*;
use yew_router::prelude::*;
use crate::Route;

#[function_component(LandingPage)]
pub fn landing_page() -> Html {
    let navigator = use_navigator().unwrap();

    let go_workout = {
        let nav = navigator.clone();
        Callback::from(move |_| nav.push(&Route::Workout))
    };

    html! {
        <div class="px-4 py-6 pb-24 space-y-8 max-w-lg mx-auto">
            // Hero
            <div class="text-center space-y-3">
                <h1 class="text-4xl font-bold">{"Treening"}</h1>
                <p class="text-gray-400 text-lg">{"Your offline gym workout tracker"}</p>
                <p class="text-gray-500 text-sm">
                    {"A privacy-first, no-account-needed PWA built in Rust/WASM. "}
                    {"Track workouts, plan routines, and monitor your progress \u{2014} all data stays on your device."}
                </p>
            </div>

            // CTA
            <button
                class="w-full py-4 bg-blue-600 rounded-xl text-lg font-bold hover:bg-blue-700 active:bg-blue-800 transition"
                onclick={go_workout}
            >{"Start a Workout"}</button>

            // Features
            <div class="space-y-4">
                <h2 class="text-xl font-semibold">{"What you can do"}</h2>
                <div class="grid grid-cols-1 gap-3">
                    <div class="bg-gray-800 rounded-lg p-4">
                        <div class="font-medium mb-1">{"\u{1f4aa} 80+ Exercises"}</div>
                        <p class="text-sm text-gray-400">
                            {"Browse a built-in library of gym exercises with muscle groups, equipment info, and illustrations. Add your own custom exercises too."}
                        </p>
                    </div>
                    <div class="bg-gray-800 rounded-lg p-4">
                        <div class="font-medium mb-1">{"\u{1f3cb} Log Workouts"}</div>
                        <p class="text-sm text-gray-400">
                            {"Track sets, reps, and weight for each exercise. A built-in timer tracks your session duration. Save completed workouts to your history."}
                        </p>
                    </div>
                    <div class="bg-gray-800 rounded-lg p-4">
                        <div class="font-medium mb-1">{"\u{1f4cb} Plan Routines"}</div>
                        <p class="text-sm text-gray-400">
                            {"Create named routines like \"Push Day\" or \"Leg Day\". Pick exercises for each routine, then start a workout from any routine with one tap."}
                        </p>
                    </div>
                    <div class="bg-gray-800 rounded-lg p-4">
                        <div class="font-medium mb-1">{"\u{1f4c5} Workout History"}</div>
                        <p class="text-sm text-gray-400">
                            {"Review all your past workouts with dates, exercises, sets, and duration. Delete old entries you no longer need."}
                        </p>
                    </div>
                    <div class="bg-gray-800 rounded-lg p-4">
                        <div class="font-medium mb-1">{"\u{1f4e4} Export & Import"}</div>
                        <p class="text-sm text-gray-400">
                            {"Back up all your data as a JSON file. Import it on another device or restore after clearing your browser. Your data is always portable."}
                        </p>
                    </div>
                    <div class="bg-gray-800 rounded-lg p-4">
                        <div class="font-medium mb-1">{"\u{1f4f4} Works Offline"}</div>
                        <p class="text-sm text-gray-400">
                            {"Install as a PWA on your phone. After the first visit, everything works without internet \u{2014} perfect for the gym."}
                        </p>
                    </div>
                </div>
            </div>

            // How to use
            <div class="space-y-4">
                <h2 class="text-xl font-semibold">{"How to use"}</h2>
                <ol class="space-y-3 text-sm text-gray-300">
                    <li class="flex gap-3">
                        <span class="flex-shrink-0 w-7 h-7 bg-blue-600 rounded-full flex items-center justify-center font-bold text-sm">{"1"}</span>
                        <div>
                            <div class="font-medium text-gray-100">{"Browse exercises"}</div>
                            <div class="text-gray-400">{"Go to the Exercises tab to explore the full library. Filter by category (Chest, Back, Legs, etc.) or search by name."}</div>
                        </div>
                    </li>
                    <li class="flex gap-3">
                        <span class="flex-shrink-0 w-7 h-7 bg-blue-600 rounded-full flex items-center justify-center font-bold text-sm">{"2"}</span>
                        <div>
                            <div class="font-medium text-gray-100">{"Create a routine (optional)"}</div>
                            <div class="text-gray-400">{"Go to Routines and tap \"+ New Routine\". Name it and add exercises. Next time you can start a workout from it directly."}</div>
                        </div>
                    </li>
                    <li class="flex gap-3">
                        <span class="flex-shrink-0 w-7 h-7 bg-blue-600 rounded-full flex items-center justify-center font-bold text-sm">{"3"}</span>
                        <div>
                            <div class="font-medium text-gray-100">{"Start a workout"}</div>
                            <div class="text-gray-400">{"Go to the Workout tab or tap the button above. Add exercises, log your sets (weight + reps), and mark them complete as you go."}</div>
                        </div>
                    </li>
                    <li class="flex gap-3">
                        <span class="flex-shrink-0 w-7 h-7 bg-blue-600 rounded-full flex items-center justify-center font-bold text-sm">{"4"}</span>
                        <div>
                            <div class="font-medium text-gray-100">{"Finish & review"}</div>
                            <div class="text-gray-400">{"Tap \"Finish & Save\" when done. Your workout appears in History. Use Settings to export your data for backup."}</div>
                        </div>
                    </li>
                </ol>
            </div>

            // Links
            <div class="flex gap-3">
                <Link<Route> to={Route::Faq} classes="flex-1 py-3 bg-gray-800 rounded-lg text-center text-sm font-medium hover:bg-gray-700">
                    {"FAQ"}
                </Link<Route>>
                <Link<Route> to={Route::Settings} classes="flex-1 py-3 bg-gray-800 rounded-lg text-center text-sm font-medium hover:bg-gray-700">
                    {"Settings"}
                </Link<Route>>
            </div>
        </div>
    }
}
