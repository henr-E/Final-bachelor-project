use proto::sensor_data_ingest::{ParseFailure, ParseFailureReason};

/// Wrapper trait for implementing functions on foreighn type.
pub(crate) trait ParseFailureFrom {
    /// Create a [`ParseFailure`] from a [`csv::Error`].
    fn from_csv_error(err: csv::Error) -> ParseFailure;
    /// Create a [`ParseFailure`] from a [`serde_json::Error`].
    fn from_json_error(err: serde_json::Error) -> ParseFailure;
}

impl ParseFailureFrom for ParseFailure {
    fn from_csv_error(err: csv::Error) -> Self {
        use csv::ErrorKind;
        match err.kind() {
            ErrorKind::Io(err) => Self {
                reason: ParseFailureReason::Unknown.into(),
                // Is it safe to send back the I/O error? Does this contain information we do not
                // want a possible attacker to have?
                details: Some(prost_value::from_string(format!("{:?}", err))),
            },
            ErrorKind::Utf8 { .. } => Self {
                reason: ParseFailureReason::DataFormatParsingError.into(),
                details: Some(prost_value::from_string(err.to_string())),
            },
            ErrorKind::UnequalLengths { .. } => Self {
                reason: ParseFailureReason::DataFormatParsingError.into(),
                details: Some(prost_value::from_string(err.to_string())),
            },
            ErrorKind::Seek => Self {
                reason: ParseFailureReason::DataFormatParsingError.into(),
                // No idea what went wrong here (weird internal stuff not usable for user).
                details: None,
            },
            ErrorKind::Serialize(_) => Self {
                reason: ParseFailureReason::Unknown.into(),
                details: Some(prost_value::from_string(err.to_string())),
            },
            ErrorKind::Deserialize { .. } => Self {
                reason: ParseFailureReason::DataFormatParsingError.into(),
                details: Some(prost_value::from_string(format!("{:?}", err))),
            },
            _ => Self {
                reason: ParseFailureReason::DataFormatParsingError.into(),
                details: Some(prost_value::from_string(err.to_string())),
            },
        }
    }

    fn from_json_error(err: serde_json::Error) -> Self {
        use serde_json::error::Category;
        match err.classify() {
            Category::Io => Self {
                reason: ParseFailureReason::Unknown.into(),
                // Is it safe to send back the I/O error? Does this contain information we do not
                // want a possible attacker to have?
                details: Some(prost_value::from_string(format!(
                    "{:?}",
                    err.io_error_kind().expect("error should be an I/O error")
                ))),
            },
            Category::Syntax => Self {
                reason: ParseFailureReason::DataFormatParsingError.into(),
                details: Some(prost_value::from_string(err.to_string())),
            },
            Category::Data => Self {
                // The error happened because a library had a bug.
                reason: ParseFailureReason::Unknown.into(),
                details: Some(prost_value::from_string(err.to_string())),
            },
            Category::Eof => Self {
                reason: ParseFailureReason::DataFormatParsingError.into(),
                details: Some(prost_value::from_string(err.to_string())),
            },
        }
    }
}
