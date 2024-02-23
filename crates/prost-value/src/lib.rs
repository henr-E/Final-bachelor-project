//! Simple utility crate for easily creating prost types from frequently used rust types.

use prost_types::{value::Kind, Struct, Value};
use std::collections::BTreeMap;

pub fn from_string(value: impl Into<String>) -> Value {
    Value {
        kind: Some(Kind::StringValue(value.into())),
    }
}

pub fn from_f64(value: f64) -> Value {
    Value {
        kind: Some(Kind::NumberValue(value)),
    }
}

pub fn from_map(value: BTreeMap<String, Value>) -> Value {
    Value {
        kind: Some(Kind::StructValue(Struct { fields: value })),
    }
}
