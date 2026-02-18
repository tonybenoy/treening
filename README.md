# Treening - Workout Tracker

[![Hits](https://hits.sh/tonybenoy.github.io/treening.svg?view=today-total&style=flat-square&label=users)](https://hits.sh/tonybenoy.github.io/treening/)

Treening is a modern, offline-capable gym workout tracker built with Rust and WebAssembly. It is designed to be fast, private, and easy to use directly in your browser or installed as a Progressive Web App (PWA).

## Features

- **Workout Logging:** Track your sets, reps, and weights in real-time.
- **Specialized Tracking:** Different metrics for Strength (Weight+Reps), Cardio (Dist+Time), Duration (Time), and Bodyweight exercises.
- **Personal Profile:** Track your height, age, and gender to personalize your experience.
- **Body Progress:** Log your weight and body fat % over time with built-in progress charts.
- **Adaptive UI:** Full support for **Light and Dark modes** based on your system preference or manual toggle.
- **Routine Management:** Create and save your favorite workout routines for quick access.
- **Advanced Analytics:** Detailed charts for exercise progress, muscle group distribution, and body metrics.
- **Relative Ranking:** Compare intensity with friends using **Relative Volume** (Volume per kg of body weight) for a fairer competition.
- **Direct P2P Sync:** Transfer your data directly between devices using WebRTC—completely private and server-less.
- **Auto Backup:** Automatic IndexedDB backup mirror with auto-restore if localStorage is cleared. Persistent storage is requested to prevent browser eviction.
- **Storage Awareness:** Detects when localStorage quota is exceeded and shows a warning banner so you can export your data before anything is lost.
- **Privacy Focused:** Your data stays on your device. No accounts, no tracking. An anonymous page-view counter (hits.sh) is used solely to measure usage \u{2014} it can be disabled in Settings.
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
