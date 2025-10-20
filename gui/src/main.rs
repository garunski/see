use dioxus::prelude::*;

mod components;
mod state;
mod services {
    pub mod workflow;
}
mod app;

fn main() {
    launch(app::App);
}
