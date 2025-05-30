// Copyright (c) 2023 - 2025 Chair for Design Automation, TUM
// Copyright (c) 2025 Munich Quantum Software Company GmbH
// All rights reserved.
//
// SPDX-License-Identifier: MIT
//
// Licensed under the MIT License

pub mod config;
pub mod state;

pub type Color = [u8; 4];
pub type Position = (f32, f32);
pub type Size = (f32, f32);
pub type Extent = (Position, Position);
