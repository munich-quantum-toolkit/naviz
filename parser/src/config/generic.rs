//! Convert the [parsed config][parser::parse] into a [generic Config][Config]
//! and allow parsing to concrete configuration using the provided helper-functions.

use super::{
    error::{Error, ErrorKind, TagError},
    parser::{self, Value},
};
use crate::common::{color::Color, percentage::Percentage};
use fraction::Fraction;
use itertools::{Either, Itertools};
use regex::Regex;
use std::{collections::HashMap, hash::Hash, mem::replace};

/// A generic [Config] to later parse.
/// The first element is the target mappings, which map from an identifier,
/// the second element is all the mappings which mapped from some other value.
#[derive(Default)]
pub struct Config(pub HashMap<String, ConfigItem>, pub Box<Maps>);

/// Collects properties which do not have an identifier as key,
/// but some other value.
#[derive(Default)]
pub struct Maps {
    pub string: HashMap<String, Value>,
    pub regex: Vec<(Regex, Value)>,
    pub number: HashMap<Fraction, Value>,
    pub percentage: HashMap<Percentage, Value>,
    pub boolean: HashMap<bool, Value>,
    pub color: HashMap<Color, Value>,
    pub tuple: Vec<(Vec<Value>, Value)>,
    pub set: Vec<(Vec<Value>, Value)>,
}

/// A [ConfigItem] representing either a [Value][ConfigItem::Value], a [Struct][ConfigItem::Struct],
/// or a [Map][ConfigItem::Map]
pub enum ConfigItem {
    /// A singular value
    Value(Value),
    /// A struct
    Struct(Config),
    /// A map from value to struct
    Map(Vec<(Value, Config)>),
}

impl Config {
    /// Insert a [parser::ConfigItem] into this [Config].
    /// Returns [`Some(old_value)`][Some] if an old value was overwritten,
    /// or [`None`] if the value did not previously exist.
    fn insert(&mut self, value: parser::ConfigItem) -> Option<ConfigItem> {
        let Config(target, maps) = self;
        match value {
            // Insert a property:
            parser::ConfigItem::Property(key, value) => match key {
                // Identifier: Insert into target-mappings (of current struct)
                parser::Value::Identifier(id) => target.insert(id, ConfigItem::Value(value)),
                // Other value: Insert into other-mappings (which can then be parsed into Maps)
                parser::Value::String(s) => maps.string.insert(s, value).map(ConfigItem::Value),
                parser::Value::Regex(r) => {
                    maps.regex.push((r, value));
                    None
                }
                parser::Value::Number(n) => maps.number.insert(n, value).map(ConfigItem::Value),
                parser::Value::Percentage(p) => {
                    maps.percentage.insert(p, value).map(ConfigItem::Value)
                }
                parser::Value::Boolean(b) => maps.boolean.insert(b, value).map(ConfigItem::Value),
                parser::Value::Color(c) => maps.color.insert(c, value).map(ConfigItem::Value),
                parser::Value::Tuple(t) => {
                    maps.tuple.push((t, value));
                    None
                }
                parser::Value::Set(s) => {
                    maps.set.push((s, value));
                    None
                }
            },
            // Insert a block (parses as a new struct):
            parser::ConfigItem::Block(key, value) => {
                target.insert(key, ConfigItem::Struct(value.into()))
            }
            // Insert a named block (parses as a map of new structs):
            parser::ConfigItem::NamedBlock(key, name, value) => {
                let t = target
                    .entry(key)
                    .or_insert_with(|| ConfigItem::Map(Vec::new()));
                if let ConfigItem::Map(t) = t {
                    t.push((name, value.into()));
                    None
                } else {
                    Some(replace(t, ConfigItem::Map(vec![(name, value.into())])))
                }
            }
        }
    }
}

impl From<parser::Config> for Config {
    /// Converts a [parser::Config] into a [Config].
    /// This silently overwrites duplicated values.
    fn from(value: parser::Config) -> Self {
        let mut target = Config::default();
        for item in value {
            target.insert(item);
        }
        target
    }
}

/// Get and remove a raw [ConfigItem] from the [Config] at the specified `name`.
/// Will return [ErrorKind::MissingField] if not found.
#[inline]
pub fn get_item_raw(config: &mut Config, name: &'static str) -> Result<ConfigItem, Error> {
    config
        .0
        .remove(name)
        .ok_or(ErrorKind::MissingField(name).into())
}

/// Get a value from a [Config].
/// Will return [ErrorKind::MissingField] if not found
/// and all errors the target-type returns during conversion using [TryInto::try_into].
#[inline]
pub fn get_item<T>(config: &mut Config, name: &'static str) -> Result<T, Error>
where
    ConfigItem: TryInto<T, Error = Error>,
{
    get_item_raw(config, name)?.try_into().tag(name)
}

/// Get a struct from a [Config].
/// Will return [ErrorKind::MissingField] if not found
/// and all errors the target-type returns during conversion using [TryInto::try_into].
#[inline]
pub fn get_item_struct<T: TryFrom<Config, Error = Error>>(
    config: &mut Config,
    name: &'static str,
) -> Result<T, Error> {
    match get_item_raw(config, name)? {
        ConfigItem::Struct(s) => s.try_into(),
        _ => Err(ErrorKind::WrongType("block").into()),
    }
    .tag(name)
}

impl TryFrom<ConfigItem> for String {
    type Error = Error;
    fn try_from(value: ConfigItem) -> Result<Self, Self::Error> {
        match value {
            ConfigItem::Value(Value::String(s)) => Ok(s),
            _ => Err(ErrorKind::WrongType("string").into()),
        }
    }
}

impl TryFrom<ConfigItem> for Regex {
    type Error = Error;
    fn try_from(value: ConfigItem) -> Result<Self, Self::Error> {
        match value {
            ConfigItem::Value(Value::Regex(r)) => Ok(r),
            _ => Err(ErrorKind::WrongType("regex").into()),
        }
    }
}

impl TryFrom<ConfigItem> for Fraction {
    type Error = Error;
    fn try_from(value: ConfigItem) -> Result<Self, Self::Error> {
        match value {
            ConfigItem::Value(Value::Number(n)) => Ok(n),
            _ => Err(ErrorKind::WrongType("number").into()),
        }
    }
}

impl TryFrom<ConfigItem> for bool {
    type Error = Error;
    fn try_from(value: ConfigItem) -> Result<Self, Self::Error> {
        match value {
            ConfigItem::Value(Value::Boolean(b)) => Ok(b),
            _ => Err(ErrorKind::WrongType("boolean").into()),
        }
    }
}

impl TryFrom<ConfigItem> for Color {
    type Error = Error;
    fn try_from(value: ConfigItem) -> Result<Self, Self::Error> {
        match value {
            ConfigItem::Value(Value::Color(c)) => Ok(c),
            _ => Err(ErrorKind::WrongType("color").into()),
        }
    }
}

pub fn get_item_named_struct<T, O>(
    config: &mut Config,
    name: &'static str,
    mut filter: impl FnMut(Value) -> Option<O>,
) -> Result<Vec<(O, T)>, Error>
where
    Config: TryInto<T, Error = Error>,
{
    match get_item_raw(config, name)? {
        ConfigItem::Map(s) => s
            .into_iter()
            .filter_map(|(v, c)| filter(v).map(|k| (k, c)))
            .map(|(k, c)| c.try_into().map(|t| (k, t)))
            .collect(),
        _ => Err(ErrorKind::WrongType("block").into()),
    }
    .tag(name)
}

// #[inline]
// pub fn get_item_map_vec<K, V>(config: &mut Config, name: &'static str) -> Result<Vec<(K, V)>, Error>
// where
//     K: MappedProperty<Output = Vec<(K, Value)>>,
//     V: FilteredFrom<Value>,
// {
//     let maps = config
//         .0
//         .get_mut(name)
//         .ok_or(Error::from(ErrorKind::MissingField(name)))
//         .tag(name)?;
//     match maps {
//         &mut ConfigItem::Struct(ref mut x) => {
//             let Config(_, ref mut m) = x;
//             let target = K::get(m);
//             let (taken, left): (Vec<_>, Vec<_>) =
//                 std::mem::take(target)
//                     .into_iter()
//                     .partition_map(|(k, v)| match V::check(v) {
//                         FilteredMapResult::Take(v) => Either::Left((k, v)),
//                         FilteredMapResult::Leave(v) => Either::Right((k, v)),
//                     });
//             *target = left;
//             Ok(taken)
//         }
//         _ => Err(ErrorKind::WrongType("block").into()),
//     }
//     .tag(name)
// }

// #[inline]
// pub fn get_item_map<K, V>(config: &mut Config, name: &'static str) -> Result<HashMap<K, V>, Error>
// where
//     K: MappedProperty<Output = HashMap<K, Value>> + Eq + Hash,
//     V: FilteredFrom<Value>,
// {
//     let maps = config
//         .0
//         .get_mut(name)
//         .ok_or(Error::from(ErrorKind::MissingField(name)))
//         .tag(name)?;
//     match maps {
//         &mut ConfigItem::Struct(ref mut x) => {
//             let Config(_, ref mut m) = x;
//             let target = K::get(m);
//             let (taken, left): (HashMap<_, _>, HashMap<_, _>) = std::mem::take(target)
//                 .into_iter()
//                 .partition_map(|(k, v)| match V::check(v) {
//                     FilteredMapResult::Take(v) => Either::Left((k, v)),
//                     FilteredMapResult::Leave(v) => Either::Right((k, v)),
//                 });
//             *target = left;
//             Ok(taken)
//         }
//         _ => Err(ErrorKind::WrongType("block").into()),
//     }
//     .tag(name)
// }

/// Get a map from in [Maps] of the struct in the specified field (by `name`).
///
/// The value-types implement all the required traits.
#[inline]
pub fn get_item_map<K, V, M, MI>(config: &mut Config, name: &'static str) -> Result<M, Error>
where
    K: MappedProperty<Output = MI>,
    V: FilteredFrom<Value>,
    M: MapOrVec<K, V>,
    MI: MapOrVec<K, Value>,
{
    // Get the `Maps` of the specified field
    let maps = config
        .0
        .get_mut(name)
        .ok_or(Error::from(ErrorKind::MissingField(name)))
        .tag(name)?;

    match maps {
        // Needs to be a struct
        &mut ConfigItem::Struct(ref mut x) => {
            let Config(_, ref mut m) = x;
            let target = K::get(m); // The target map
            let (taken, left): (M, MI) = std::mem::take(target) // Temporarily take the target map to partition
                .into_iter()
                .partition_map(|(k, v)| match V::filtered_from(v) {
                    // Partition and map to the target type (if valid value)
                    FilteredMapResult::Take(v) => Either::Left((k, v)),
                    FilteredMapResult::Leave(v) => Either::Right((k, v)),
                });
            // Write back leftover values
            *target = left;
            Ok(taken)
        }
        // Not a struct
        _ => Err(ErrorKind::WrongType("block").into()),
    }
    .tag(name)
}

/// A trait that marks types which are either a [Vec<(K, V)>] or a [HashMap<K, V>].
pub trait MapOrVec<K, V>: IntoIterator<Item = (K, V)> + Default + Extend<(K, V)> {}
impl<K, V> MapOrVec<K, V> for Vec<(K, V)> {}
impl<K: Eq + Hash, V> MapOrVec<K, V> for HashMap<K, V> {}

/// A trait which allows getting the respective map from [Maps] for the implementing type.
pub trait MappedProperty {
    /// The output type (of the field in [Maps])
    type Output;

    /// Access the respective field from [Maps]
    fn get(maps: &mut Maps) -> &mut Self::Output;
}

impl MappedProperty for String {
    type Output = HashMap<String, Value>;
    fn get(maps: &mut Maps) -> &mut Self::Output {
        &mut maps.string
    }
}

impl MappedProperty for Regex {
    type Output = Vec<(Regex, Value)>;
    fn get(maps: &mut Maps) -> &mut Self::Output {
        &mut maps.regex
    }
}

impl MappedProperty for Fraction {
    type Output = HashMap<Fraction, Value>;
    fn get(maps: &mut Maps) -> &mut Self::Output {
        &mut maps.number
    }
}

impl MappedProperty for Percentage {
    type Output = HashMap<Percentage, Value>;
    fn get(maps: &mut Maps) -> &mut Self::Output {
        &mut maps.percentage
    }
}

impl MappedProperty for bool {
    type Output = HashMap<bool, Value>;
    fn get(maps: &mut Maps) -> &mut Self::Output {
        &mut maps.boolean
    }
}

impl MappedProperty for Color {
    type Output = HashMap<Color, Value>;
    fn get(maps: &mut Maps) -> &mut Self::Output {
        &mut maps.color
    }
}

impl MappedProperty for Vec<Value> {
    type Output = Vec<(Vec<Value>, Value)>;
    fn get(maps: &mut Maps) -> &mut Self::Output {
        &mut maps.tuple
    }
}

/// The result of a filtered mapping-operation.
pub enum FilteredMapResult<Input, Output> {
    /// Take the mapped value
    Take(Output),
    /// Leave the original value
    Leave(Input),
}

/// A trait which allows taking a mapped value or leaving the original value.
pub trait FilteredFrom<T>: Sized {
    /// Either take the value, apply a mapping-function, to convert to Self,
    /// and return [FilteredMapResult::Take(Self)][FilteredMapResult::Take],
    /// or leave the original value and return
    /// [FilteredMapResult::Leave(value)][FilteredMapResult::Leave],
    fn filtered_from(value: T) -> FilteredMapResult<T, Self>;
}

impl FilteredFrom<Value> for String {
    fn filtered_from(value: Value) -> FilteredMapResult<Value, Self> {
        match value {
            Value::String(s) => FilteredMapResult::Take(s),
            _ => FilteredMapResult::Leave(value),
        }
    }
}

impl FilteredFrom<Value> for Regex {
    fn filtered_from(value: Value) -> FilteredMapResult<Value, Self> {
        match value {
            Value::Regex(r) => FilteredMapResult::Take(r),
            _ => FilteredMapResult::Leave(value),
        }
    }
}

impl FilteredFrom<Value> for Fraction {
    fn filtered_from(value: Value) -> FilteredMapResult<Value, Self> {
        match value {
            Value::Number(n) => FilteredMapResult::Take(n),
            _ => FilteredMapResult::Leave(value),
        }
    }
}

impl FilteredFrom<Value> for Percentage {
    fn filtered_from(value: Value) -> FilteredMapResult<Value, Self> {
        match value {
            Value::Percentage(p) => FilteredMapResult::Take(p),
            _ => FilteredMapResult::Leave(value),
        }
    }
}

impl FilteredFrom<Value> for Color {
    fn filtered_from(value: Value) -> FilteredMapResult<Value, Self> {
        match value {
            Value::Color(c) => FilteredMapResult::Take(c),
            _ => FilteredMapResult::Leave(value),
        }
    }
}

/// A marker for an identifier
pub struct Identifier(pub String);

impl FilteredFrom<Value> for Identifier {
    fn filtered_from(value: Value) -> FilteredMapResult<Value, Self> {
        match value {
            Value::Identifier(t) => FilteredMapResult::Take(Identifier(t)),
            _ => FilteredMapResult::Leave(value),
        }
    }
}

impl FilteredFrom<Value> for Vec<Value> {
    fn filtered_from(value: Value) -> FilteredMapResult<Value, Self> {
        match value {
            Value::Tuple(t) => FilteredMapResult::Take(t),
            _ => FilteredMapResult::Leave(value),
        }
    }
}
