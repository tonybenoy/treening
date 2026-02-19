# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Development Commands

```bash
# Dev server (http://127.0.0.1:8080)
trunk serve

# Production build (output in dist/)
trunk build --release

# Prerequisites (one-time setup)
rustup target add wasm32-unknown-unknown
cargo install --locked trunk
```

There are no tests or linting configured. The project compiles to WebAssembly via Trunk, which handles the wasm-bindgen and asset pipeline automatically using `index.html` as the entry point (configured in `Trunk.toml`).

## Architecture

**Treening** is an offline-first gym workout tracker PWA built with Rust/WASM using the Yew framework. It compiles to WebAssembly and runs entirely in the browser with no backend server.

### Core Modules

- **`src/main.rs`** — App entry point, defines `Route` enum (hash-based routing via `yew-router`), theme management, and storage-full warning banner.
- **`src/models.rs`** — All data types: `Workout`, `WorkoutExercise`, `WorkoutSet`, `Exercise`, `Routine`, `BodyMetric`, `Friend`, `UserConfig`, `AppData`. Exercise tracking has four types: Strength, Cardio, Duration, Bodyweight.
- **`src/storage.rs`** — Persistence layer using browser `LocalStorage` (via `gloo`). Each entity type has its own key (`treening_workouts`, etc.). All saves trigger a debounced IndexedDB backup. Includes JSON import/export and merge logic (dedup by ID).
- **`src/backup.rs`** — IndexedDB backup mirror with auto-restore if localStorage is cleared. Requests persistent storage to prevent browser eviction.
- **`src/data.rs`** — Seed data with built-in exercise definitions.
- **`src/sharing.rs`** — Workout sharing via compressed+base64-encoded URL fragments.

### UI Structure

- **`src/pages/`** — One file per route: home, exercises, workout, history, routines, settings, social, faq, analytics, shared.
- **`src/components/`** — Reusable components: nav (bottom navigation), workout_log, exercise_list, exercise_detail, charts, settings, sync (P2P via PeerJS/WebRTC), routine_editor, custom_exercise, history, share_modal.

### Key Patterns

- All components are Yew `#[function_component]` using hooks (`use_state`, `use_effect_with`, `use_memo`).
- Styling is Tailwind CSS classes (inline in Rust `html!` macros), with dark mode via `dark:` prefix classes.
- Data flow: components call `storage::load_*()` / `storage::save_*()` directly — there is no global state store.
- P2P sync uses WebRTC data channels via PeerJS (loaded from CDN in `index.html`), with Rust interop through `wasm-bindgen` and `web-sys`.
- The app uses `HashRouter` (not `BrowserRouter`), so all routes are `/#/path`.
