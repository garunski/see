use dioxus::prelude::*;
use dioxus_desktop::{Config, WindowBuilder};

mod components;
mod icons;
mod layout;
mod pages;
mod queries;
mod services {
    pub mod database;
    pub mod execution;
    pub mod prompt;
    pub mod settings;
    pub mod workflow;

    pub use database::clear_database;
    pub use settings::SettingsService;
}

fn main() {
    let _tracing_guard = s_e_e_core::init_tracing(None)
        .map_err(|e| format!("Failed to initialize tracing: {}", e))
        .expect("Failed to initialize tracing");

    tracing::trace!("GUI application starting");

    let workspace_root = std::env::var("CARGO_MANIFEST_DIR")
        .ok()
        .and_then(|manifest| {
            if manifest.ends_with("/gui") {
                std::fs::canonicalize(&manifest)
                    .ok()
                    .map(|p| p.parent().unwrap().to_path_buf())
            } else {
                std::fs::canonicalize(&manifest).ok()
            }
        })
        .or_else(|| {
            let mut path = std::env::current_exe().ok()?;
            loop {
                if path.join("Cargo.toml").exists() {
                    return Some(path);
                }
                if path.parent()?.parent().is_none() {
                    break;
                }
                path = path.parent()?.to_path_buf();
            }
            None
        });

    if let Some(workspace) = workspace_root {
        if let Err(e) = std::env::set_current_dir(&workspace) {
            tracing::warn!("Failed to change to workspace root: {}", e);
        } else {
            tracing::trace!(
                "Changed working directory to workspace root: {:?}",
                workspace
            );
        }
    }

    tracing::debug!("Initializing persistence layer");
    let rt = tokio::runtime::Runtime::new().unwrap();
    if let Err(e) = rt.block_on(async {
        s_e_e_core::init_global_store().await?;
        tracing::debug!("Populating initial data");
        s_e_e_core::populate_initial_data().await
    }) {
        tracing::error!("Failed to initialize persistence layer: {}", e);
        eprintln!("Failed to initialize persistence layer: {}", e);
        std::process::exit(1);
    }
    drop(rt);
    tracing::debug!("Persistence layer initialized successfully");

    LaunchBuilder::desktop()
        .with_cfg(
            Config::new()
                .with_window(WindowBuilder::new().with_title("Speculative Execution Engine")),
        )
        .launch(layout::App);
}
