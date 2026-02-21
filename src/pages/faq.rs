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
        question: "Why is Treening free? What\u{2019}s the catch?",
        answer: "There is no catch. Most workout apps charge subscriptions because they run servers, hire trainers to write programs, and collect your data to sell or use for marketing. Treening has none of that \u{2014} no servers, no accounts, no data collection. Your data never leaves your device. The entire app runs in your browser as a static file, so it costs almost nothing to host. It was built out of frustration with paid apps that gate basic features like workout logging behind a paywall.",
    },
    FaqItem {
        question: "Does Treening collect any data?",
        answer: "No. Treening does not collect any data. There are no analytics, no tracking pixels, no cookies, and no server-side storage. Everything stays on your device. When you sync between devices, data goes directly peer-to-peer \u{2014} it never touches a server.",
    },
    FaqItem {
        question: "Can I reorder exercises during a workout?",
        answer: "Yes. Each exercise card has up and down arrow buttons in the header. Tap them to move an exercise up or down in your workout order.",
    },
    FaqItem {
        question: "Can I set a different rest timer per exercise?",
        answer: "Yes. Each strength exercise has a small rest-time input below the sets. Change it to override the global rest timer for that specific exercise. If you leave it at the default, the global setting from Settings is used.",
    },
    FaqItem {
        question: "How does auto-fill from previous work?",
        answer: "When you add an exercise (or load a routine), Treening looks up your most recent workout containing that exercise and pre-fills the first set with the same weight, reps, distance, or duration. This saves you from typing the same numbers every session.",
    },
    FaqItem {
        question: "What are warm-up sets?",
        answer: "For strength exercises where your first set weight is above the bar weight, a 'Warm-up Sets' button appears. Tapping it prepends four progressive warm-up sets at 40%, 60%, 75%, and 90% of your working weight (with 10, 6, 4, and 2 reps respectively), rounded to the nearest 2.5 kg.",
    },
    FaqItem {
        question: "Is there an undo feature?",
        answer: "Yes. When you remove an exercise or delete a set, a floating 'Undo' button appears at the bottom of the screen for 5 seconds. Tap it to restore the deleted item.",
    },
    FaqItem {
        question: "What is the estimated 1RM progress chart?",
        answer: "In the Progress tab of Analytics, alongside the max weight and volume charts, there is now an 'Est. 1RM Per Session' chart (pink line). It shows the highest estimated one-rep max from each session using the Epley formula, so you can track strength progress even when training with different rep ranges.",
    },
    FaqItem {
        question: "What is 'Volume Per Muscle Group'?",
        answer: "In the Analytics Overview tab, there is a collapsible 'Volume Per Muscle Group' section. It shows a line chart for each of your top 4 most-trained muscle groups, plotting weekly volume over the last 8 weeks. This helps you spot imbalances in your training.",
    },
    FaqItem {
        question: "What are the milestone badges?",
        answer: "The milestones row in Analytics Overview shows achievement badges at 1, 5, 10, 25, 50, 100, 250, and 500 total workouts. Achieved badges are highlighted in gold; unachieved ones are greyed out. Below the row, you can see how many workouts remain until your next milestone.",
    },
    FaqItem {
        question: "What do the training frequency chips mean?",
        answer: "If you haven't trained a muscle group in over 7 days, a yellow warning chip appears in Analytics Overview (e.g., 'Legs: 10d ago'). At 14+ days the chip turns red. Only muscle groups you have trained at least once are tracked.",
    },
    FaqItem {
        question: "What is the confetti animation on PR?",
        answer: "When you complete a set that is a new Personal Record (PR), the set row briefly flashes gold and gets a yellow ring highlight. This gives you instant visual feedback that you just hit a new best.",
    },
    FaqItem {
        question: "Can I repeat a previous workout?",
        answer: "Yes. In the History tab, expand any workout and tap 'Repeat'. This loads all exercises, sets, weights, and supersets from that workout into a new session with all sets marked as not completed, so you can do it again.",
    },
    FaqItem {
        question: "Can I swipe to delete sets?",
        answer: "Yes, on touch devices. Swipe a set row to the left; after 80 pixels a red 'Delete' background appears and the set is removed. On desktop, use the x button as usual. The undo pill appears in both cases.",
    },
    FaqItem {
        question: "Can I export my data as CSV?",
        answer: "Yes. Go to Settings and tap 'Export CSV'. This downloads a spreadsheet-friendly file with columns for date, workout name, duration, exercise, set number, weight, reps, distance, duration, completed status, and notes.",
    },
    FaqItem {
        question: "Is it open source?",
        answer: "Yes! The source code is available on GitHub at https://github.com/tonybenoy/treening. It is built using Rust and WebAssembly for maximum performance and privacy. Contributions are very welcome!",
    },
    FaqItem {
        question: "What is 'treen'?",
        answer: "Treen refers to small handmade household objects carved from wood \u{2014} things like spoons, bowls, snuff boxes, and mallets. Before mass production, every home had these hand-turned wooden items. The achievement badges in Treening are named after treen objects as a nod to craftsmanship and steady, patient progress.",
    },
    FaqItem {
        question: "What are the Treen achievement badges on the home page?",
        answer: "The achievement grid on the home page shows badges named after traditional treen (small handmade wooden objects). They are computed from your workout data \u{2014} nothing is stored separately. Badges include Wooden Spoon (1st workout), Cutting Board (10 workouts), Snuff Box (100 total sets), Turned Bowl (50 workouts), Mallet Head (any lift \u{2265}100kg), Lignum Vitae (30-day consecutive streak), and Master Carver (100 workouts). Earned badges appear in full color; unearned ones are grayed out.",
    },
    FaqItem {
        question: "What does 'Treening' mean?",
        answer: "Treening is an Estonian word meaning 'training' or 'workout'. It felt like the perfect name for a no-nonsense workout tracker \u{2014} and the domain treen.ing was too good to pass up.",
    },
    FaqItem {
        question: "Why was Treening built?",
        answer: "After losing 20 kg and hitting a weight loss plateau, the developer started going to the gym but couldn\u{2019}t find a workout tracker that was both free and subscription-free. So Treening was built in a day using Claude Code and Gemini to solve that problem.",
    },
    FaqItem {
        question: "What is the AI Assistant?",
        answer: "The AI Assistant is an optional local AI coach that can answer questions about your workouts, suggest what to train, and analyze your progress. It runs entirely in your browser using WebGPU (via WebLLM) \u{2014} no data is sent to any server. Enable it in Settings under 'AI Assistant'.",
    },
    FaqItem {
        question: "How do I enable the AI Assistant?",
        answer: "Go to Settings and toggle on 'AI Assistant'. You can also choose which AI model to use \u{2014} from SmolLM2 360M (~200MB, fastest) to Gemma 2 2B (~1GB, best quality). Then navigate to the AI page from the home screen. On your first use you will need to download the selected model. This is a one-time download that is cached in your browser for offline use.",
    },
    FaqItem {
        question: "Which AI model should I choose?",
        answer: "It depends on your device. SmolLM2 360M (~200MB) is the fastest and lightest but gives basic answers. Qwen3 0.6B (~500MB, default) is a good balance of quality and size. Llama 3.2 1B (~600MB) and Gemma 2 2B (~1GB) give the best answers but need more GPU memory. If responses seem slow or your browser struggles, try a smaller model.",
    },
    FaqItem {
        question: "Which browsers support the AI Assistant?",
        answer: "The AI Assistant requires WebGPU, which is available in Chrome/Edge 113+ (Desktop and Android) and Safari 18+ (macOS). If your browser does not support WebGPU, you will see an 'Unsupported' message. Firefox does not yet support WebGPU by default.",
    },
    FaqItem {
        question: "Does the AI Assistant send my data anywhere?",
        answer: "No. The AI model runs 100% locally in your browser. Your workout data is computed into a summary and fed to the model as context on your device only. Nothing is sent to any server or API. This is possible because the models are small enough to run directly on your GPU via WebGPU.",
    },
    FaqItem {
        question: "Is there a community for Treening users?",
        answer: "Yes! We have GitHub Discussions at https://github.com/tonybenoy/treening/discussions. You can introduce yourself, suggest features, ask questions, share your achievements, and chat with other users. Feature requests with the most upvotes get prioritized.",
    },
    FaqItem {
        question: "How do I report a bug or request a feature?",
        answer: "The best way is to open an issue on GitHub at https://github.com/tonybenoy/treening/issues. Select the bug report or feature request template and provide as much detail as you can. Since this is a side project, it may take some time to address, but every report is appreciated.",
    },
    FaqItem {
        question: "Can I contribute to the project?",
        answer: "Absolutely! If you find a bug and know how to fix it, or want to add a feature, submitting a pull request on GitHub is the fastest way to get it into the app. Check out the repository at https://github.com/tonybenoy/treening.",
    },
    FaqItem {
        question: "How can I support the development of Treening?",
        answer: "If you enjoy using Treening and want to support its development, you can buy the developer a coffee at https://buymeacoffee.com/tonybenoy. It is completely optional but very much appreciated!",
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
                        <details class="group bg-gray-100 dark:bg-gray-800 rounded-xl neu-flat transition-colors">
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
