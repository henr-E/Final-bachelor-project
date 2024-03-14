//! Simple utility crate for easily creating prost types from frequently used rust types.

use std::collections::BTreeMap;

use prost_types::{value::Kind, Struct, Value};

/// Changes a string into a Value
pub fn from_string(value: impl Into<String>) -> Value {
    Value {
        kind: Some(Kind::StringValue(value.into())),
    }
}

/// Changes a f64 into a Value
pub fn from_f64(value: f64) -> Value {
    Value {
        kind: Some(Kind::NumberValue(value)),
    }
}

/// Changes a BTreeMap<String, Value> into a Value
pub fn from_map(value: BTreeMap<String, Value>) -> Value {
    Value {
        kind: Some(Kind::StructValue(Struct { fields: value })),
    }
}

/// Changes a serde_json::Map<String, serde_json::Value> into a Struct
pub fn to_struct(json: serde_json::Map<String, serde_json::Value>) -> Struct {
    Struct {
        fields: json
            .into_iter()
            .map(|(k, v)| (k, serde_json_to_prost(v)))
            .collect(),
    }
}

/// Changes a Value to a serde_json::Value
pub fn prost_to_serde_json(x: Value) -> serde_json::Value {
    use prost_types::value::Kind::*;
    use serde_json::Value::*;
    match x.kind {
        Some(x) => match x {
            NullValue(_) => Null,
            BoolValue(v) => Bool(v),
            NumberValue(n) => Number(serde_json::Number::from_f64(n).unwrap()),
            StringValue(s) => String(s),
            ListValue(lst) => Array(lst.values.into_iter().map(prost_to_serde_json).collect()),
            StructValue(v) => Object(
                v.fields
                    .into_iter()
                    .map(|(k, v)| (k, prost_to_serde_json(v)))
                    .collect(),
            ),
        },
        None => unimplemented!(),
    }
}

/// Changes a serde_json::Value to a Value
pub fn serde_json_to_prost(json: serde_json::Value) -> Value {
    use prost_types::value::Kind::*;
    use serde_json::Value::*;
    Value {
        kind: Some(match json {
            Null => NullValue(0 /* wat? */),
            Bool(v) => BoolValue(v),
            Number(n) => NumberValue(n.as_f64().expect("Non-f64-representable number")),
            String(s) => StringValue(s),
            Array(v) => ListValue(prost_types::ListValue {
                values: v.into_iter().map(serde_json_to_prost).collect(),
            }),
            Object(v) => StructValue(to_struct(v)),
        }),
    }
}

#[cfg(test)]
mod tests {
    use prost_types::value::Kind::{BoolValue, ListValue, NumberValue};
    use serde_json::json;

    use super::*;

    #[allow(dead_code)]
    struct Tree {
        number: i64,
        bool: bool,
        string: String,
        null: serde_json::Value,
        array: Vec<i32>,
    }

    #[test]
    fn test_to_struct() {
        let json = json!({
            "number": 0,
            "bool": true,
            "string": "this is a string",
            "null": serde_json::Value::Null,
            "array": [0, 1, 2]
        });
        let res = to_struct(json.as_object().expect("an object").clone());

        let array = prost_types::ListValue {
            values: vec![
                Value {
                    kind: std::option::Option::from(NumberValue(0f64)),
                },
                Value {
                    kind: std::option::Option::from(NumberValue(1f64)),
                },
                Value {
                    kind: std::option::Option::from(NumberValue(2f64)),
                },
            ],
        };
        let mut tree: BTreeMap<String, Value> = BTreeMap::new();
        tree.insert(
            "number".to_string(),
            Value {
                kind: std::option::Option::from(NumberValue(0f64)),
            },
        );
        tree.insert(
            "bool".to_string(),
            Value {
                kind: std::option::Option::from(BoolValue(true)),
            },
        );
        tree.insert(
            "array".to_string(),
            Value {
                kind: std::option::Option::from(ListValue(array)),
            },
        );
        tree.insert(
            "null".to_string(),
            serde_json_to_prost(serde_json::Value::Null),
        );
        tree.insert(
            "string".to_string(),
            serde_json_to_prost(serde_json::Value::String("this is a string".to_string())),
        );
        let str = Struct { fields: tree };

        assert_eq!(res, str);
    }

    #[test]
    fn test_switch_json_prost() {
        let array = prost_types::ListValue {
            values: vec![
                Value {
                    kind: std::option::Option::from(NumberValue(0f64)),
                },
                Value {
                    kind: std::option::Option::from(NumberValue(1f64)),
                },
                Value {
                    kind: std::option::Option::from(NumberValue(2f64)),
                },
            ],
        };
        let mut tree: BTreeMap<String, Value> = BTreeMap::new();
        tree.insert(
            "number".to_string(),
            Value {
                kind: std::option::Option::from(NumberValue(0f64)),
            },
        );
        tree.insert(
            "bool".to_string(),
            Value {
                kind: std::option::Option::from(BoolValue(true)),
            },
        );
        tree.insert(
            "array".to_string(),
            Value {
                kind: std::option::Option::from(ListValue(array)),
            },
        );
        tree.insert(
            "null".to_string(),
            serde_json_to_prost(serde_json::Value::Null),
        );
        tree.insert(
            "string".to_string(),
            serde_json_to_prost(serde_json::Value::String("this is a string".to_string())),
        );

        let json = json!({
            "number": 0.0,
            "bool": true,
            "string": "this is a string",
            "null": serde_json::Value::Null,
            "array": [0.0, 1.0, 2.0]
        });

        let mut tree_json: serde_json::map::Map<String, serde_json::Value> =
            serde_json::map::Map::new();
        for (name, value) in tree.clone() {
            let temp = prost_to_serde_json(value);
            tree_json.insert(name, temp);
        }

        assert_eq!(json.as_object().expect("an object").clone(), tree_json);

        let mut tree_from_json: BTreeMap<String, Value> = BTreeMap::new();
        for (name, value) in tree_json.iter() {
            let temp = serde_json_to_prost(value.clone());
            tree_from_json.insert(name.clone(), temp);
        }
        assert_eq!(tree, tree_from_json);
    }
}
