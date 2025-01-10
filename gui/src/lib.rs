#![warn(clippy::all, rust_2018_idioms)]

mod animator_adapter;
mod app;
mod aspect_panel;
mod canvas;
mod current_machine;
#[cfg(not(target_arch = "wasm32"))]
mod export_dialog;
mod future_helper;
mod import;
mod menu;
mod progress_bar;
mod util;
pub use app::App;
