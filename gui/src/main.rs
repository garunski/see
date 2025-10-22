use dioxus::prelude::*;
use dioxus_desktop::{Config, WindowBuilder};

mod components;
mod pages;
mod router;
mod state;
mod services {
    pub mod workflow;
}
mod app;

fn main() {
    let _tracing_guard = see_core::init_tracing(None)
        .map_err(|e| format!("Failed to initialize tracing: {}", e))
        .expect("Failed to initialize tracing");

    tracing::info!("GUI starting");

    LaunchBuilder::desktop()
        .with_cfg(Config::new().with_window(WindowBuilder::new().with_title("See Workflow Engine")))
        .launch(app::App);

}
