# Treening - Workout Tracker

**[treen.ing](https://treen.ing/)**

Treening is a modern, offline-capable gym workout tracker built with Rust and WebAssembly. It is designed to be fast, private, and easy to use directly in your browser or installed as a Progressive Web App (PWA).

## Features

- **Workout Logging:** Track your sets, reps, and weights in real-time.
- **Specialized Tracking:** Different metrics for Strength (Weight+Reps), Cardio (Dist+Time), Duration (Time), and Bodyweight exercises.
- **Rest Timer:** Automatic countdown timer after completing a set (configurable duration, +30s / Skip controls, vibration alert).
- **Per-Exercise Rest Timer:** Override the global rest duration on a per-exercise basis.
- **1RM Calculator:** Estimated one-rep max shown for every completed strength set using the Epley formula.
- **1RM Progress Chart:** Track your estimated 1RM over time in the Analytics Progress tab.
- **Previous Performance Overlay:** See your last workout's sets for each exercise displayed above the current sets.
- **Auto-Fill from Previous:** New exercises and routines auto-populate weight/reps from your most recent session.
- **Warm-Up Set Generator:** One-tap warm-up sets at 40%, 60%, 75%, and 90% of your working weight.
- **PR Highlights:** Yellow "PR" badge appears when you beat your all-time best weight for an exercise.
- **PR Animation:** Gold flash and ring highlight on sets that are new Personal Records.
- **Plate Calculator:** Tap the barbell icon next to any weight input to see the exact plates needed per side.
- **Reorder Exercises:** Move exercises up or down during a workout with arrow buttons.
- **Superset Support:** Group consecutive exercises into supersets with a purple border and badge.
- **Per-Set Notes:** Add notes to individual sets (e.g., "felt easy", "pause rep").
- **Undo After Remove:** Floating undo pill (5s auto-dismiss) when you delete an exercise or set.
- **Swipe to Delete:** Swipe set rows left on touch devices to delete; desktop x-button still works.
- **Repeat Workout:** Tap "Repeat" on any past workout to reload it as a new session.
- **Calendar Heatmap:** GitHub-style workout frequency heatmap on the Analytics page.
- **Volume Per Muscle Group:** Weekly volume line charts for your top muscle groups (collapsible).
- **Milestone Badges:** Achievement badges at 1, 5, 10, 25, 50, 100, 250, and 500 workouts.
- **Training Frequency Warnings:** Yellow/red chips when a muscle group hasn't been trained in 7/14+ days.
- **CSV Export:** Download all workout data as a spreadsheet-friendly CSV file.
- **Personal Profile:** Track your height, age, and gender to personalize your experience.
- **Body Progress:** Log your weight and body fat % over time with built-in progress charts.
- **Adaptive UI:** Full support for **Light and Dark modes** based on your system preference or manual toggle.
- **Routine Management:** Create and save your favorite workout routines for quick access.
- **Advanced Analytics:** Detailed charts for exercise progress, muscle group distribution, and body metrics.
- **Relative Ranking:** Compare intensity with friends using **Relative Volume** (Volume per kg of body weight) for a fairer competition.
- **Trusted Device Sync:** Pair your devices once, then data syncs automatically whenever both are open. Share a pairing link or enter a Device ID to connect.
- **Share Links:** Share your Friend Code or Device Pairing link via the native share sheet (mobile) or clipboard — recipients are auto-added when they open the link.
- **Auto Backup:** Automatic IndexedDB backup mirror with auto-restore if localStorage is cleared. Persistent storage is requested to prevent browser eviction.
- **Storage Awareness:** Detects when localStorage quota is exceeded and shows a warning banner so you can export your data before anything is lost.
- **Privacy Focused:** Your data stays on your device. No accounts, no tracking.
- **PWA Support:** Installable on iOS and Android for a native app-like experience.

## Tech Stack

- **Frontend:** [Yew](https://yew.rs/) (Rust WASM framework)
- **Styling:** [Tailwind CSS](https://tailwindcss.com/)
- **Build Tool:** [Trunk](https://trunkrs.dev/)
- **Connectivity:** [PeerJS](https://peerjs.com/) for P2P Signaling
- **Storage:** Browser LocalStorage + IndexedDB backup via [Gloo](https://github.com/rustwasm/gloo) and [web-sys](https://rustwasm.github.io/wasm-bindgen/api/web_sys/)
- **Time/Date:** [Chrono](https://github.com/chronotope/chrono)

## Getting Started

### Prerequisites

To build and run this project locally, you need:

1. **Rust:** Install from [rustup.rs](https://rustup.rs/)
2. **WASM Target:**
   ```bash
   rustup target add wasm32-unknown-unknown
   ```
3. **Trunk:**
   ```bash
   cargo install --locked trunk
   ```

### Running Locally

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd treening
   ```

2. Start the development server:
   ```bash
   trunk serve
   ```

3. Open your browser and navigate to `http://127.0.0.1:8080`.

### Building for Production

To create a production build:

```bash
trunk build --release
```

The output will be in the `dist/` directory.

## Project Structure

- `src/components/`: Reusable UI components (Nav, Workout Log, etc.).
- `src/pages/`: Main application views (Home, Exercises, Routines, Analytics).
- `src/models.rs`: Data structures for workouts, exercises, and routines.
- `src/storage.rs`: Logic for persisting data to the browser.
- `src/backup.rs`: IndexedDB auto-backup and persistent storage request.
- `src/data.rs`: Seed data and exercise definitions.
- `icons/`: SVG icons for the app and exercise categories.

## License

[MIT](LICENSE)
