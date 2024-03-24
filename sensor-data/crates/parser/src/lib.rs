use crate::error::ParseFailureFrom;
use proto::sensor_data_ingest::{ParseFailure, ParseFailureReason};

mod error;

// TODO: Allow the user to specify a sensor-unique format that contains a mapping in the
// database.

/// Input formats supported by the parser. These formats should line up with the formats defined in
/// [`sensor_data_ingest::sensor_data::FileFormat`](proto::sensor_data_ingest::sensor_data_file::FileFormat).
pub enum FileFormat {
    /// Csv file format where records are seperated using the `delimiter`.
    Csv { delimiter: u8 },
    /// JSON file format.
    Json,
}

/// When parsing a list of entries, use this as the top level key for the JSON object returned.
///
/// # Example
///
/// ```json
/// {
///   "entries": [
///     {
///       <record-0>
///     },
///     {
///       <record-1>
///     },
///     ...
///   ]
/// }
/// ```
const ENTRIES_KEY: &str = "entries";

/// Parses the sensor data from a byte slice into a BSON (binary JSON) object.
pub fn from_slice(input: &[u8], format: FileFormat) -> Result<bson::Document, ParseFailure> {
    Ok(match format {
        FileFormat::Csv { delimiter } => {
            let arr = csv::ReaderBuilder::new()
                .delimiter(delimiter)
                .from_reader(input)
                // Deserialize every entry as `bson::Document` to prevent everything being a single string.
                .deserialize::<bson::Document>()
                .map(|b| match b {
                    Err(e) => Err(ParseFailure::from_csv_error(e)),
                    Ok(d) => {
                        // Convert to `bson::Bson`.
                        Ok(bson::Bson::Document(d))
                    }
                })
                .collect::<Result<Vec<_>, _>>()?;

            bson_document_from_array(arr)
        }
        FileFormat::Json => {
            let result: bson::Bson =
                serde_json::from_slice(input).map_err(ParseFailure::from_json_error)?;

            use bson::Bson;
            match result {
                // Allow the user to send JSON that only contains a top level array. Not sure if
                // this is supported by the JSON specification, but might as well...
                Bson::Array(arr) => bson_document_from_array(arr),
                Bson::Document(doc) => doc,
                _ => {
                    // Any top level JSON structure that we do not recognize as parsable.
                    return Err(ParseFailure::new_string_detail(
                        ParseFailureReason::DataStructureNotSupported,
                        "top level item must be on of [`array`, `document`]",
                    ));
                }
            }
        }
    })
}

/// Create a [`bson::Document`] with a single array under the `ENTRIES_KEY`.
fn bson_document_from_array(arr: bson::Array) -> bson::Document {
    let mut result = bson::Document::default();
    result.insert(ENTRIES_KEY, arr);
    result
}

#[cfg(test)]
mod tests {
    use super::{from_slice, FileFormat, ENTRIES_KEY};

    #[test]
    fn parse_csv() {
        let data = r#"
            Date;Time;Global_active_power;Global_reactive_power;Voltage;Global_intensity;Sub_metering_1;Sub_metering_2;Sub_metering_3
            16/12/2006;17:24:00;4.216;0.418;234.840;18.400;0.000;1.000;17.000
            16/12/2006;17:25:00;5.360;0.436;233.630;23.000;0.000;1.000;16.000
            16/12/2006;17:26:00;5.374;0.498;233.290;23.000;0.000;2.000;17.000
            16/12/2006;17:27:00;5.388;0.502;233.740;23.000;0.000;1.000;17.000
            16/12/2006;17:28:00;3.666;0.528;235.680;15.800;0.000;1.000;17.000
            16/12/2006;17:29:00;3.520;0.522;235.020;15.000;0.000;2.000;17.000
            16/12/2006;17:30:00;3.702;0.520;235.090;15.800;0.000;1.000;17.000
            16/12/2006;17:31:00;3.700;0.520;235.220;15.800;0.000;1.000;17.000
            16/12/2006;17:32:00;3.668;0.510;233.990;15.800;0.000;1.000;17.000
            16/12/2006;17:33:00;3.662;0.510;233.860;15.800;0.000;2.000;16.000
            16/12/2006;17:34:00;4.448;0.498;232.860;19.600;0.000;1.000;17.000
            16/12/2006;17:35:00;5.412;0.470;232.780;23.200;0.000;1.000;17.000
            16/12/2006;17:36:00;5.224;0.478;232.990;22.400;0.000;1.000;16.000
            16/12/2006;17:37:00;5.268;0.398;232.910;22.600;0.000;2.000;17.000
            16/12/2006;17:38:00;4.054;0.422;235.240;17.600;0.000;1.000;17.000
            16/12/2006;17:39:00;3.384;0.282;237.140;14.200;0.000;0.000;17.000
            16/12/2006;17:40:00;3.270;0.152;236.730;13.800;0.000;0.000;17.000
        "#.replace(' ', "");

        let doc = from_slice(data.trim().as_bytes(), FileFormat::Csv { delimiter: b';' }).unwrap();
        let records = doc.get(ENTRIES_KEY).unwrap();
        let bson::Bson::Array(records) = records else {
            panic!("records not of type array");
        };

        let record0 = records[0]
            .as_document()
            .expect("record is not a `bson::Document`");
        assert_eq!(record0.get_str("Date").unwrap(), "16/12/2006");
        assert_eq!(record0.get_f64("Global_active_power").unwrap(), 4.216);
        assert_eq!(record0.get_f64("Voltage").unwrap(), 234.84);

        let record16 = records[16]
            .as_document()
            .expect("record is not a `bson::Document`");
        assert_eq!(record16.get_str("Time").unwrap(), "17:40:00");
        assert_eq!(record16.get_f64("Global_active_power").unwrap(), 3.27);
        assert_eq!(record16.get_f64("Voltage").unwrap(), 236.73);
    }

    #[test]
    fn parse_json_list() {
        let data = r#"
            [
                {
                    "Date": "16/12/2006",
                    "Time": "17:24:00",
                    "Global_active_power": 4.216,
                    "Global_reactive_power": 0.418,
                    "Voltage": 234.84,
                    "Global_intensity": 18.4,
                    "Sub_metering_1": 0,
                    "Sub_metering_2": 1,
                    "Sub_metering_3": 17
                },
                {
                    "Date": "16/12/2006",
                    "Time": "17:25:00",
                    "Global_active_power": 5.36,
                    "Global_reactive_power": 0.436,
                    "Voltage": 233.63,
                    "Global_intensity": 23,
                    "Sub_metering_1": 0,
                    "Sub_metering_2": 1,
                    "Sub_metering_3": 16
                },
                {
                    "Date": "16/12/2006",
                    "Time": "17:26:00",
                    "Global_active_power": 5.374,
                    "Global_reactive_power": 0.498,
                    "Voltage": 233.29,
                    "Global_intensity": 23,
                    "Sub_metering_1": 0,
                    "Sub_metering_2": 2,
                    "Sub_metering_3": 17
                }
            ]
        "#;

        let doc = from_slice(data.as_bytes(), FileFormat::Json).unwrap();
        let records = doc.get(ENTRIES_KEY).unwrap();
        let bson::Bson::Array(records) = records else {
            panic!("records not of type array");
        };

        let record0 = records[0]
            .as_document()
            .expect("record is not a `bson::Document`");
        assert_eq!(record0.get_str("Date").unwrap(), "16/12/2006");
        assert_eq!(record0.get_f64("Global_active_power").unwrap(), 4.216);
        assert_eq!(record0.get_f64("Voltage").unwrap(), 234.84);

        let record2 = records[2]
            .as_document()
            .expect("record is not a `bson::Document`");
        assert_eq!(record2.get_str("Time").unwrap(), "17:26:00");
        assert_eq!(record2.get_f64("Global_active_power").unwrap(), 5.374);
        assert_eq!(record2.get_f64("Voltage").unwrap(), 233.29);
    }
}
