//! The [MachineConfig] and sub-types.
//! See documentation of file-format.

use super::{
    error::Error,
    generic::{get_item, get_item_named_struct, get_item_struct, Config},
    parser::Value,
    position::Position,
};
use fraction::Fraction;
use std::collections::HashMap;

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
pub struct MachineConfig {
    pub name: String,
    pub movement: MovementConfig,
    pub time: TimeConfig,
    pub distance: DistanceConfig,
    pub zone: HashMap<String, ZoneConfig>,
    pub trap: HashMap<String, TrapConfig>,
}

impl TryFrom<Config> for MachineConfig {
    type Error = Error;
    fn try_from(mut value: Config) -> Result<Self, Self::Error> {
        Ok(Self {
            name: get_item(&mut value, "name")?,
            movement: get_item_struct(&mut value, "movement")?,
            time: get_item_struct(&mut value, "time")?,
            distance: get_item_struct(&mut value, "distance")?,
            zone: get_item_named_struct(&mut value, "zone", |v| match v {
                Value::Identifier(id) => Some(id),
                _ => None,
            })?
            .into_iter()
            .collect(),
            trap: get_item_named_struct(&mut value, "trap", |v| match v {
                Value::Identifier(id) => Some(id),
                _ => None,
            })?
            .into_iter()
            .collect(),
        })
    }
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
pub struct MovementConfig {
    pub max_speed: Fraction,
}

impl TryFrom<Config> for MovementConfig {
    type Error = Error;
    fn try_from(mut value: Config) -> Result<Self, Self::Error> {
        Ok(Self {
            max_speed: get_item(&mut value, "max_speed")?,
        })
    }
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
pub struct TimeConfig {
    pub load: Fraction,
    pub store: Fraction,
    pub ry: Fraction,
    pub rz: Fraction,
    pub cz: Fraction,
    pub unit: String,
}

impl TryFrom<Config> for TimeConfig {
    type Error = Error;
    fn try_from(mut value: Config) -> Result<Self, Self::Error> {
        Ok(Self {
            load: get_item(&mut value, "load")?,
            store: get_item(&mut value, "store")?,
            ry: get_item(&mut value, "ry")?,
            rz: get_item(&mut value, "rz")?,
            cz: get_item(&mut value, "cz")?,
            unit: get_item(&mut value, "unit")?,
        })
    }
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
pub struct DistanceConfig {
    pub interaction: Fraction,
    pub unit: String,
}

impl TryFrom<Config> for DistanceConfig {
    type Error = Error;
    fn try_from(mut value: Config) -> Result<Self, Self::Error> {
        Ok(Self {
            interaction: get_item(&mut value, "interaction")?,
            unit: get_item(&mut value, "unit")?,
        })
    }
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
pub struct ZoneConfig {
    pub from: Position,
    pub to: Position,
}

impl TryFrom<Config> for ZoneConfig {
    type Error = Error;
    fn try_from(mut value: Config) -> Result<Self, Self::Error> {
        Ok(Self {
            from: get_item(&mut value, "from")?,
            to: get_item(&mut value, "to")?,
        })
    }
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
pub struct TrapConfig {
    pub position: Position,
}

impl TryFrom<Config> for TrapConfig {
    type Error = Error;
    fn try_from(mut value: Config) -> Result<Self, Self::Error> {
        Ok(Self {
            position: get_item(&mut value, "position")?,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::{lexer, parser};

    #[test]
    fn example() {
        let input = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/rsc/test/example.namachine"
        ));

        let expected = MachineConfig {
            name: "Name".to_string(),
            movement: MovementConfig {
                max_speed: Fraction::new(23u64, 1u64),
            },
            time: TimeConfig {
                load: Fraction::new(21u64, 5u64),
                store: Fraction::new(12u64, 1u64),
                ry: Fraction::new(1u64, 10u64),
                rz: Fraction::new(3u64, 1u64),
                cz: Fraction::new(1u64, 1u64),
                unit: "us".to_string(),
            },
            distance: DistanceConfig {
                interaction: Fraction::new(12u64, 1u64),
                unit: "um".to_string(),
            },
            zone: HashMap::from([
                (
                    "zone0".to_string(),
                    ZoneConfig {
                        from: (Fraction::new(0u64, 1u64), Fraction::new(0u64, 1u64)),
                        to: (Fraction::new(10u64, 1u64), Fraction::new(10u64, 1u64)),
                    },
                ),
                (
                    "zone1".to_string(),
                    ZoneConfig {
                        from: (-Fraction::new(61u64, 5u64), Fraction::new(8u64, 1u64)),
                        to: (Fraction::new(23u64, 1u64), Fraction::new(4u64, 1u64)),
                    },
                ),
            ]),
            trap: HashMap::from([
                (
                    "trap0".to_string(),
                    TrapConfig {
                        position: (Fraction::new(0u64, 1u64), Fraction::new(0u64, 1u64)),
                    },
                ),
                (
                    "trap1".to_string(),
                    TrapConfig {
                        position: (Fraction::new(1u64, 1u64), Fraction::new(1u64, 1u64)),
                    },
                ),
            ]),
        };

        let lexed = lexer::lex(input).expect("Failed to lex");
        let parsed = parser::parse(lexed.as_slice()).expect("Failed to parse");
        let generic: Config = parsed.into();
        let config: MachineConfig = generic.try_into().expect("Failed to load config");

        assert_eq!(config, expected);
    }
}
