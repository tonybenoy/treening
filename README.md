# Treening - Workout Tracker

Treening is a modern, offline-capable gym workout tracker built with Rust and WebAssembly. It is designed to be fast, private, and easy to use directly in your browser or installed as a Progressive Web App (PWA).

## Features

- **Workout Logging:** Track your sets, reps, and weights in real-time.
- **Exercise Library:** Comprehensive list of exercises with categorized body parts.
- **Routine Management:** Create and save your favorite workout routines for quick access.
- **Analytics & History:** View your progress over time and review past workouts.
- **Offline First:** Works without an internet connection using Service Workers and local storage.
- **Privacy Focused:** Your data stays on your device. No accounts, no tracking.
- **Direct P2P Sync:** Transfer your data directly between devices using WebRTC and QR codes—completely private and server-less.
- **PWA Support:** Installable on iOS and Android for a native app-like experience.

## Tech Stack

- **Frontend:** [Yew](https://yew.rs/) (Rust WASM framework)
- **Styling:** [Tailwind CSS](https://tailwindcss.com/)
- **Build Tool:** [Trunk](https://trunkrs.dev/)
- **Storage:** Browser LocalStorage via [Gloo](https://github.com/rustwasm/gloo)
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
- `src/data.rs`: Seed data and exercise definitions.
- `icons/`: SVG icons for the app and exercise categories.

## License

[MIT](LICENSE)
