pub(crate) fn bson_object_to_string(obj: &bson::Bson) -> &'static str {
    match obj {
        bson::Bson::Double(_) => "double",
        bson::Bson::String(_) => "string",
        bson::Bson::Array(_) => "array",
        bson::Bson::Document(_) => "document",
        bson::Bson::Boolean(_) => "boolean",
        bson::Bson::Null => "null-object",
        bson::Bson::RegularExpression(_) => "regular-expression",
        bson::Bson::JavaScriptCode(_) => "javascript-code",
        bson::Bson::JavaScriptCodeWithScope(_) => "javascript-code-with-scope",
        bson::Bson::Int32(_) => "int32",
        bson::Bson::Int64(_) => "int64",
        bson::Bson::Timestamp(_) => "timestamp",
        bson::Bson::Binary(_) => "binary",
        bson::Bson::ObjectId(_) => "object-id",
        bson::Bson::DateTime(_) => "datetime",
        bson::Bson::Symbol(_) => "symbol",
        bson::Bson::Decimal128(_) => "decimal",
        bson::Bson::Undefined => "undefined-object",
        bson::Bson::MaxKey => "max-key",
        bson::Bson::MinKey => "min-key",
        bson::Bson::DbPointer(_) => "db-pointer",
    }
}