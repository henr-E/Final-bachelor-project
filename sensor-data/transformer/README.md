# Transformer

## Supported formats

### Document

#### Multiple measurements

This code snippet can be used to generate a BSON file with multiple measurements:

```rs
let bson = doc! {
    "measurements": [
        {"timestamp":  Utc::now().to_rfc3339(), "energy_consumption": 5.6},
        {"timestamp": (Utc::now() + Duration::from_secs(1)).to_rfc3339(), "energy_consumption": 2.99}
    ]
};
File::create("uploads/document_multiple.bson")
    .unwrap()
    .write_all(&bson::to_vec(&bson).unwrap())
    .unwrap();
```

The inclusion of a "measurements" item in the BSON object is required because BSON mandates a top-level item. This item can be named anything, as long as it's present.

Parsing an Array(vec<Bson>) is not supported. Although the bson! macro can generate an Array, it can't be converted into a bson::Document, which is necessary for writing to a file or accessing values.

#### Single measurement

This code snippet can be used to generate a BSON file with a single measurement:

```rs
let bson = doc! {
    "timestamp":  Utc::now().to_rfc3339(), "energy_consumption": 5.6
};
File::create("uploads/document_single.bson")
    .unwrap()
    .write_all(&bson::to_vec(&bson).unwrap())
    .unwrap();
```

## Expected database entries

Perform these queries to insert example data when using the `uploads/multiple_example.bson` and `uploads/single_example.bson` file (make sure that these files are present in the `uploads` directory):

```sql
INSERT INTO sensors (id, name, description, location, user_id) VALUES ('515d5e01-7f8d-1857-f503-8923c8552f3f', 'sensor_val', 'This is a test sensor', point(1.0, 2.0), 0);
INSERT INTO archive_sensor_data_files(identifier, time, sensor_id, path, metadata) VALUES ('414d4e01-7f8d-1857-f503-8923c8442f3f', now(), '515d5e01-7f8d-1857-f503-8923c8552f3f', 'uploads/document_multiple.bson', '');
INSERT INTO archive_sensor_data_files(identifier, time, sensor_id, path, metadata) VALUES ('313d3e02-6f5d-1857-f503-8923c8332f3f', now(), '515d5e01-7f8d-1857-f503-8923c8552f3f', 'uploads/document_single.bson', '');
INSERT INTO sensor_signals (alias, unit, quantity, sensor_id, prefix) VALUES ('energy_consumption', 'watt', 'energy', '515d5e01-7f8d-1857-f503-8923c8552f3f', 1000);
INSERT INTO sensor_signals (alias, unit, quantity, sensor_id, prefix) VALUES ('timestamp', 'utc', 'timestamp', '515d5e01-7f8d-1857-f503-8923c8552f3f', 1);
```
