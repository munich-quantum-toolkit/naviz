// Copyright (c) 2023 - 2025 Chair for Design Automation, TUM
// Copyright (c) 2025 Munich Quantum Software Company GmbH
// All rights reserved.
//
// SPDX-License-Identifier: MIT
//
// Licensed under the MIT License

pub mod common;
pub mod config;
pub mod input;

/// Error returned when parsing/lexing.
/// Contains Reference to the input.
pub type ParseError<I> = winnow::error::ParseError<I, ParseErrorInner>;

/// Inner error in [ParseError].
pub type ParseErrorInner = winnow::error::ContextError;
