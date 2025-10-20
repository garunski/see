use dioxus::prelude::*;

mod components;
mod pages;
mod router;
mod state;
mod services {
    pub mod workflow;
}
mod app;

fn main() {
    launch(app::App);
}
