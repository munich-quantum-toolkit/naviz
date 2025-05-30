#![warn(clippy::all, rust_2018_idioms)]
// Copyright (c) 2023 - 2025 Chair for Design Automation, TUM
// Copyright (c) 2025 Munich Quantum Software Company GmbH
// All rights reserved.
//
// SPDX-License-Identifier: MIT
//
// Licensed under the MIT License

mod animator_adapter;
mod app;
mod aspect_panel;
mod canvas;
mod current_machine;
mod drawable;
mod error;
mod errors;
#[cfg(not(target_arch = "wasm32"))]
mod export_dialog;
mod future_helper;
mod import;
mod menu;
mod progress_bar;
mod util;
pub use app::App;
