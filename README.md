    # Taurifest

    **Taurifest** is a declarative engine for building robust Tauri applications from a single `manifest.json` file. It's heavily inspired by the philosophy of [Serverokey](https://github.com/Xzdes/serverokey), adapting its "architecture-over-code" paradigm for the desktop environment.

    With Taurifest, you describe your application's UI, data sources, and business logic declaratively, letting the engine handle the implementation details.

    ## Core Concepts

    - **Single Source of Truth:** Your entire application is defined in a `manifest.json`.
    - **Declarative Logic:** Use JSON-based `steps` to describe what happens, not how.
    - **Reactive UI:** The UI automatically updates when data changes, powered by a Rust backend.

    ## Quick Start

    1. Add `taurifest` to your `Cargo.toml`:
    ```toml
    [dependencies]
    taurifest = "0.1.0"
    ```

    2. In your `main.rs`, use the `Builder`:
    ```rust
    use taurifest::Builder;

    fn main() {
        let tauri_builder = Builder::new("app").build(); // "app" is your manifest directory
        tauri_builder
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
    }