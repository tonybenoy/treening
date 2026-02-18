use yew::prelude::*;

struct FaqItem {
    question: &'static str,
    answer: &'static str,
}

const FAQS: &[FaqItem] = &[
    FaqItem {
        question: "Do I need to create an account?",
        answer: "No. Treening has no accounts, no login, and no server. All your data is stored locally in your browser's localStorage.",
    },
    FaqItem {
        question: "Does it work offline?",
        answer: "Yes! After your first visit, the app is cached by a service worker and works fully offline. You can install it as a PWA on your phone for the best experience.",
    },
    FaqItem {
        question: "How do I install it on my phone?",
        answer: "On Android (Chrome): tap the menu and select \"Add to Home screen\" or \"Install app\". On iOS (Safari): tap the share button and select \"Add to Home Screen\". The app will appear as a standalone app.",
    },
    FaqItem {
        question: "Are there different tracking metrics?",
        answer: "Yes. Treening uses specialized metrics for different exercises: Strength (Weight + Reps), Cardio (Distance + Time), Duration (Time only), and Bodyweight (Reps only). The app automatically chooses the best metric for built-in exercises.",
    },
    FaqItem {
        question: "Can I track my body weight and progress?",
        answer: "Yes. Go to Settings to fill out your Personal Profile and use the 'Body Progress' section to log your weight and body fat %. You can view your progress charts in the 'Body' tab of Analytics.",
    },
    FaqItem {
        question: "What is 'Relative Volume'?",
        answer: "Relative Volume is your total workout volume divided by your body weight. This provides a fairer 'intensity score' when comparing rankings with friends, as it accounts for different body sizes.",
    },
    FaqItem {
        question: "Does it support Dark Mode?",
        answer: "Yes. Treening supports Light, Dark, and System-default modes. You can change your preference in the Settings tab under 'App Theme'.",
    },
    FaqItem {
        question: "Where is my data stored?",
        answer: "Your primary data lives in your browser's localStorage, and an automatic backup is kept in IndexedDB. If you clear all browser data both copies will be lost \u{2014} use the Export feature in Settings to keep an external backup.",
    },
    FaqItem {
        question: "How do I back up my data?",
        answer: "Treening automatically mirrors every save to IndexedDB as a safety net. If localStorage is ever cleared, the app will auto-restore from the IndexedDB backup on next load. For extra safety, go to Settings and tap \"Export Data\" to download a JSON file you can keep offline.",
    },
    FaqItem {
        question: "What does the 'Storage full' warning mean?",
        answer: "It means your browser's localStorage quota has been exceeded and new data could not be saved. Go to Settings and export your data immediately, then free up space by clearing old browser data for other sites. The IndexedDB backup should still have your most recent data.",
    },
    FaqItem {
        question: "Can I transfer data to another device?",
        answer: "Yes. You have two options: 1. Use the \"Sync Devices\" feature on the Home or Settings page to transfer data directly (P2P) between devices by scanning a QR code or entering a Meeting ID. 2. Export your data as JSON on one device and import it on the other.",
    },
    FaqItem {
        question: "How does Sync work?",
        answer: "Sync uses PeerJS (WebRTC) to create a direct, private connection between two browsers. When you scan the QR code or enter a Meeting ID, the devices \"handshake\" and send your workout data directly to each other. Your data is never stored on any server during this process.",
    },
    FaqItem {
        question: "Will syncing overwrite my existing data?",
        answer: "No. Syncing uses an intelligent merge. It will combine the workouts, routines, and custom exercises from both devices, skipping any duplicates. It's safe to sync even if both devices have existing data.",
    },
    FaqItem {
        question: "What is the Rest Timer?",
        answer: "When you mark a set as complete, a countdown timer automatically starts at the bottom of the screen. The default is 90 seconds but you can change it in Settings under 'Rest Timer'. You can add 30 seconds or skip the timer at any time. Your phone will vibrate when the rest period ends.",
    },
    FaqItem {
        question: "What is the 1RM estimate?",
        answer: "For every completed strength set with more than 1 rep, Treening shows your estimated one-rep max (1RM) using the Epley formula: weight \u{00d7} (1 + reps/30). This helps you gauge your true maximal strength without actually testing it.",
    },
    FaqItem {
        question: "How does 'Previous Performance' work?",
        answer: "When you add an exercise to your workout, Treening looks up your most recent workout that included the same exercise and shows a summary of the sets (e.g., 'Last: S1: 80kg x8, S2: 85kg x6') in gray text above your current sets. This helps you decide what weights to use.",
    },
    FaqItem {
        question: "What does the yellow 'PR' badge mean?",
        answer: "A yellow 'PR' (Personal Record) badge appears next to a set when the weight you entered exceeds the heaviest weight you have ever completed for that exercise across all previous workouts. It is a quick way to see when you are hitting new highs.",
    },
    FaqItem {
        question: "How does the Plate Calculator work?",
        answer: "Tap the barbell icon next to any weight input to see the exact plates needed per side (25, 20, 15, 10, 5, 2.5, 1.25 kg). The calculation uses your configured bar weight (default 20 kg), which you can change in Settings under 'Bar Weight'.",
    },
    FaqItem {
        question: "What are Supersets?",
        answer: "Supersets let you group two or more exercises together to perform them back-to-back with minimal rest. Tap 'Group' on an exercise to link it with the exercise above. Grouped exercises display a purple left border and a 'Superset' badge. Tap 'Ungroup' to remove an exercise from the superset.",
    },
    FaqItem {
        question: "Can I add notes to individual sets?",
        answer: "Yes. Tap the note icon on any set row to expand an inline text field where you can write a note for that specific set (e.g., 'pause rep', 'felt easy'). The note is saved with the set and also included when you share a workout.",
    },
    FaqItem {
        question: "What is the Calendar Heatmap?",
        answer: "The Calendar Heatmap on the Analytics Overview tab is a GitHub-style grid showing the last 20 weeks of workout activity. Each cell represents a day: dark means no workout, light green means one workout, and bright green means two or more. It gives you a quick visual overview of your training consistency.",
    },
    FaqItem {
        question: "Can I add my own exercises?",
        answer: "Yes. Go to the Exercises tab and tap \"Add Custom Exercise\". You can also choose which metric to track (Strength, Cardio, etc.) for your custom moves.",
    },
    FaqItem {
        question: "What are routines?",
        answer: "Routines are pre-planned workout templates. Create one in the Routines tab (e.g., \"Push Day\"). Then start a workout from that routine with one tap \u{2014} all exercises are pre-loaded.",
    },
    FaqItem {
        question: "Can I edit a saved workout?",
        answer: "Yes. In the History tab, you can expand any workout and tap 'Edit Workout' to change the name, delete exercises, or update sets and reps.",
    },
    FaqItem {
        question: "Does Treening collect any data?",
        answer: "No. Treening does not collect any data. Everything stays on your device.",
    },
    FaqItem {
        question: "Is it open source?",
        answer: "Yes! The source code is available on GitHub. It is built using Rust and WebAssembly for maximum performance and privacy.",
    },
];

#[function_component(FaqPage)]
pub fn faq_page() -> Html {
    html! {
        <div class="px-4 py-4 pb-24 max-w-lg mx-auto space-y-4 transition-colors duration-200">
            <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">{"FAQ"}</h1>
            <p class="text-gray-500 dark:text-gray-400 text-sm">{"Frequently asked questions about Treening."}</p>
            <div class="space-y-3">
                { for FAQS.iter().map(|faq| {
                    html! {
                        <details class="group bg-gray-100 dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-transparent transition-colors shadow-sm">
                            <summary class="px-4 py-3 cursor-pointer font-bold text-gray-800 dark:text-gray-200 hover:text-blue-600 dark:hover:text-blue-400 transition-colors list-none flex justify-between items-center">
                                {faq.question}
                                <span class="text-gray-400 group-open:rotate-180 transition-transform">{"\u{25be}"}</span>
                            </summary>
                            <div class="px-4 pb-4 text-sm text-gray-600 dark:text-gray-400 leading-relaxed border-t border-gray-200 dark:border-gray-700 pt-3 mt-1">
                                {faq.answer}
                            </div>
                        </details>
                    }
                })}
            </div>
        </div>
    }
}
