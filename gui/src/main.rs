use dioxus::prelude::*;
use dioxus_desktop::{Config, WindowBuilder};

mod components;
mod hooks;
mod icons;
mod layout;
mod pages;
mod state;
mod services {
    pub mod database;
    pub mod history;
    pub mod prompt;
    pub mod workflow;

    pub use database::clear_database;
}

fn main() {
    let _tracing_guard = s_e_e_core::init_tracing(None)
        .map_err(|e| format!("Failed to initialize tracing: {}", e))
        .expect("Failed to initialize tracing");

    tracing::info!("GUI starting");

    LaunchBuilder::desktop()
        .with_cfg(
            Config::new()
                .with_window(WindowBuilder::new().with_title("Speculative Execution Engine")),
        )
        .launch(layout::App);
}
