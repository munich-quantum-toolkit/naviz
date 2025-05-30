// Copyright (c) 2023 - 2025 Chair for Design Automation, TUM
// Copyright (c) 2025 Munich Quantum Software Company GmbH
// All rights reserved.
//
// SPDX-License-Identifier: MIT
//
// Licensed under the MIT License

use egui::Ui;

/// Something that can be drawn to a [Ui].
pub trait Drawable {
    /// Draws this [Drawable] to the [Ui]
    fn draw(self, ui: &mut Ui);
}
