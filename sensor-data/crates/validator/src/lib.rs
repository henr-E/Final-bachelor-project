use crate::util::bson_object_to_string;
use proto::sensor_data_ingest::{ParseFailure, ParseFailureReason, ParseResult};
use sensor_store::Signals;
use std::{borrow::Cow, collections::HashSet};

mod util;

type ValidateResponse = Option<Vec<ParseFailure>>;
type Fields<'a> = HashSet<Cow<'a, str>>;

pub struct Validator<'a> {
    fields: Fields<'a>,
}

impl<'a> Validator<'a> {
    pub fn new(fields: impl Into<Fields<'a>>) -> Self {
        Self {
            fields: fields.into(),
        }
    }

    pub fn from_signals<'s>(signals: &'s Signals<'a>) -> Self {
        let fields = signals
            .iter()
            .map(|s| s.name.clone())
            .collect::<HashSet<_>>();
        Self { fields }
    }
}

impl Validator<'_> {
    pub fn validate(&self, data: &bson::Bson) -> ParseResult {
        use bson::Bson;
        match data {
            Bson::Array(arr) => self.validate_from_array(arr),
            Bson::Document(doc) => self.validate_from_document(doc),
            obj => ParseFailure::new_string_detail(
                ParseFailureReason::DataStructureNotSupported,
                format!(
                    "unsupported top level object: {}",
                    bson_object_to_string(obj)
                ),
            )
            .into(),
        }
    }

    pub fn validate_from_document(&self, data: &bson::Document) -> ParseResult {
        self.validate_document(data)
            .map_or(ParseResult::new_ok(), |e| e.into())
    }

    pub fn validate_from_array(&self, data: &bson::Array) -> ParseResult {
        self.validate_array(data)
            .map_or(ParseResult::new_ok(), |e| e.into())
    }

    /// Validates the bson document. It can contain a single key with an array under it. In this
    /// case, it is assumed to be a list of records.
    fn validate_document(&self, data: &bson::Document) -> ValidateResponse {
        if data.len() == 1 {
            // Get the only inner value and validate as a single record.
            let key = data
                .keys()
                .next()
                .expect("data should contain at least a single key");
            let inner_value = data.get(key).expect("key should have a value");
            if let Some(arr) = inner_value.as_array() {
                // Validate the inner array as an array of records.
                self.validate_array(arr)
            } else {
                // Validate the top level object as a record with a single key
                self.validate_record(data)
            }
        } else {
            // Validate data as a single record
            self.validate_record(data)
        }
    }

    // Validates the bson array. Assumes that it is an array of records.
    fn validate_array(&self, data: &bson::Array) -> ValidateResponse {
        let failures = data
            .iter()
            .flat_map(|r| {
                let Some(doc) = r.as_document() else {
                    return Some(vec![ParseFailure::new_string_detail(
                        ParseFailureReason::DataStructureNotSupported,
                        format!("unsupported object in array: {}", bson_object_to_string(r)),
                    )]);
                };
                self.validate_record(doc)
            })
            // `flat_map` and `flatten` is needed here to flatten option into vec and vec into vec.
            .flatten()
            .collect::<Vec<ParseFailure>>();

        if failures.is_empty() {
            None
        } else {
            Some(failures)
        }
    }

    /// Validate the record. Document has to be a single record.
    fn validate_record(&self, data: &bson::Document) -> ValidateResponse {
        let fields = data.keys().map(|k| k.into()).collect::<HashSet<Cow<_>>>();

        if !self.fields.is_subset(&fields) {
            let different_fields = self.fields.difference(&fields);
            let failures = different_fields
                .map(|f| {
                    ParseFailure::new_string_detail(
                        ParseFailureReason::DataStructureNotSupported,
                        format!("unexpected field `{}` in record", f),
                    )
                })
                .collect::<Vec<_>>();
            return Some(failures);
        }

        let failures = data
            .iter()
            .filter_map(|(k, v)| match self.fields.contains(k.as_str()) {
                true => {
                    if Self::bson_value_is_decimal(v) {
                        None
                    } else {
                        Some(ParseFailure::new_string_detail(
                            ParseFailureReason::DataStructureNotSupported,
                            format!("invalid type of value: {}", v.to_string().trim_matches('"')),
                        ))
                    }
                }
                false => {
                    // Key is not in the specified format, but all required keys are present, so
                    // ignore this one.
                    None
                }
            })
            .collect::<Vec<ParseFailure>>();

        if failures.is_empty() {
            None
        } else {
            Some(failures)
        }
    }

    fn bson_value_is_decimal(value: &bson::Bson) -> bool {
        match value {
            bson::Bson::Double(_)
            | bson::Bson::String(_)
            | bson::Bson::Int32(_)
            | bson::Bson::Int64(_)
            | bson::Bson::Decimal128(_) => {
                // Ensure that the value is parsable by the crate used in sqlx decimal
                // representation.
                value
                    .to_string()
                    // Remove string quotes.
                    .trim_matches('"')
                    .parse::<rust_decimal::Decimal>()
                    .is_ok()
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Validator;
    use bson::{bson, doc};
    use std::collections::HashSet;

    #[test]
    fn validate_valid_object() {
        let value = doc! {
            "temperature": "25.7",
            "rain": "2330.523",
            // Wind in radians
            "wind": "0.5",
        };
        let validator = Validator {
            fields: HashSet::from(["temperature".into(), "rain".into(), "wind".into()]),
        };
        let result = validator.validate(&bson::Bson::Document(value));
        assert!(result.failures.is_empty());
    }

    #[test]
    fn validate_valid_array() {
        let value = vec![
            bson!({
                "foo": 123,
                "bar": 32.6,
                "qux": "23.2",
            }),
            bson!({
                "foo": "23.2",
                "bar": 43.2,
                "qux": "100000000.00000000000001",
            }),
        ];
        let validator = Validator {
            fields: HashSet::from(["foo".into(), "bar".into(), "qux".into()]),
        };
        let result = validator.validate(&bson::Bson::Array(value));
        assert!(result.failures.is_empty());
    }

    #[test]
    fn validate_valid_object_single_key_not_array() {
        let value = doc! {
            "temperature": "50"
        };
        let validator = Validator {
            fields: HashSet::from(["temperature".into()]),
        };
        let result = validator.validate(&bson::Bson::Document(value));
        assert!(result.failures.is_empty())
    }

    #[test]
    fn validate_valid_object_single_key_is_array() {
        let value = doc! {
            "records": [
                {
                    "temperature": "30.90",
                },
                {
                    "temperature": "30.8",
                }
            ]
        };
        let validator = Validator {
            fields: HashSet::from(["temperature".into()]),
        };
        let result = validator.validate(&bson::Bson::Document(value));
        assert!(result.failures.is_empty())
    }

    #[test]
    fn validate_invalid_object_single_key_is_array() {
        let value = doc! {
            "records": [
                "50",
                20,
                "foo",
            ]
        };
        let validator = Validator {
            fields: HashSet::from(["foo".into()]),
        };
        let result = validator.validate(&bson::Bson::Document(value));
        assert!(!result.failures.is_empty());
    }

    #[test]
    fn validate_invalid_object_incorrect_key() {
        let value = doc! {
            "temp": 50,
        };
        let validator = Validator {
            fields: HashSet::from(["foo".into()]),
        };
        let result = validator.validate(&bson::Bson::Document(value));
        assert!(!result.failures.is_empty());
    }

    #[test]
    fn validate_invalid_object_missing_key() {
        let value = doc! {
            "foo": 50,
        };
        let validator = Validator {
            fields: HashSet::from(["foo".into(), "bar".into()]),
        };
        let result = validator.validate(&bson::Bson::Document(value));
        assert!(!result.failures.is_empty());
    }

    #[test]
    fn validate_invalid_object_empty() {
        let value = doc! {};
        let validator = Validator {
            fields: HashSet::from(["foo".into(), "bar".into()]),
        };
        let result = validator.validate(&bson::Bson::Document(value));
        assert!(!result.failures.is_empty());
    }
}
