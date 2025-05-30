// Copyright (c) 2023 - 2025 Chair for Design Automation, TUM
// Copyright (c) 2025 Munich Quantum Software Company GmbH
// All rights reserved.
//
// SPDX-License-Identifier: MIT
//
// Licensed under the MIT License

use super::{
    error::{Error, ErrorKind},
    generic::ConfigItem,
    parser::Value,
};
use fraction::Fraction;

/// A position (x- and y- coordinate)
pub type Position = (Fraction, Fraction);

impl TryFrom<ConfigItem> for Position {
    type Error = Error;
    fn try_from(value: ConfigItem) -> Result<Self, Self::Error> {
        match value {
            ConfigItem::Value(Value::Tuple(t)) => match t[..] {
                [Value::Number(x), Value::Number(y)] => Ok((x, y)),
                _ => Err(ErrorKind::WrongType("position").into()),
            },
            _ => Err(ErrorKind::WrongType("position").into()),
        }
    }
}
