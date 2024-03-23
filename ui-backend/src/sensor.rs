use num_bigint::{BigInt, Sign};
use proto::frontend::proto_sensor_crud::CrudFailureReason;
use proto::frontend::proto_sensor_crud::{
    create_sensor_response::Result as CResult, delete_sensor_response::Result as DResult,
    read_sensor_response::Result as RResult, update_sensor_response::Result as UResult,
    BigInt as ProtoBigInt, CreateSensorRequest, CreateSensorResponse, CrudFailure,
    DeleteSensorRequest, DeleteSensorResponse, ReadSensorRequest, ReadSensorResponse,
    Sensor as ProtoSensor, Signal as ProtoSignal, UpdateSensorRequest, UpdateSensorResponse,
};
use proto::frontend::SensorCrudService;
use sensor_store::quantity::Quantity;
use sensor_store::unit::Unit;
use sensor_store::{sensor::Sensor, SensorStore as SensorStore_};
use sqlx::types::BigDecimal;
use std::str::FromStr;
use thiserror::Error;
use tonic::{Request, Response, Status};
use tracing::error;
use uuid::Uuid;

pub struct SensorStore(SensorStore_);

fn make_database_error() -> CrudFailure {
    CrudFailure {
        failures: vec![CrudFailureReason::DatabaseInsertionError.into()],
    }
}
fn make_uuid_error() -> CrudFailure {
    CrudFailure {
        failures: vec![CrudFailureReason::UuidFormatError.into()],
    }
}
fn make_uuid_not_found_error() -> CrudFailure {
    CrudFailure {
        failures: vec![CrudFailureReason::UuidNotPresentError.into()],
    }
}
fn make_signal_format_error(err: SignalError) -> CrudFailure {
    CrudFailure {
        failures: vec![match err {
            SignalError::UnitParseError => CrudFailureReason::InvalidUnitError,
            SignalError::QuantityParseError => CrudFailureReason::InvalidQuantityError,
        }
        .into()],
    }
}
impl SensorStore {
    pub async fn new() -> Self {
        Self(
            SensorStore_::new()
                .await
                .expect("Could not create sensor store."),
        )
    }
    pub fn get_store(&self) -> &SensorStore_ {
        &self.0
    }
}

#[derive(Error, Debug)]
enum SignalError {
    #[error("Invalid quantity!")]
    QuantityParseError,
    #[error("Invalid unit!")]
    UnitParseError,
}

/// Transforms ORM sensor to expected gRPC sensor type.
///
/// This operation may fail, if the prefix exceeds u64 integer part.
fn into_proto_sensor(sensor: Sensor) -> ProtoSensor {
    let mut signals: Vec<ProtoSignal> = Vec::new();
    // for every signal from orm signal
    for signal in sensor.signals().iter() {
        // extract the prefix.
        let (big_int, exponent) = signal.prefix.as_bigint_and_exponent();
        let (sign, integer) = big_int.to_u32_digits();
        let prefix: ProtoBigInt = ProtoBigInt {
            integer: integer.to_vec(),
            sign: sign != Sign::Minus,
            exponent,
        };
        // create expected gRPC signal type.
        signals.push(ProtoSignal {
            unit: signal.unit.to_string(),
            alias: signal.name.to_string(),
            prefix: Some(prefix),
            quantity: signal.quantity.to_string(),
        })
    }
    ProtoSensor {
        name: sensor.name.to_string(),
        uuid: sensor.id.to_string(),
        description: sensor.description.unwrap_or_default().to_string(),
        longitude: sensor.location.0,
        latitude: sensor.location.1,
        signals,
    }
}

fn into_sensor(sensor: ProtoSensor) -> Result<Sensor<'static>, SignalError> {
    // Create a random uuid for the sensor.
    let sensor_uuid = Uuid::now_v7();
    let description = match sensor.description.is_empty() {
        true => None,
        false => Some(sensor.description),
    };
    // build a orm sensor.
    let mut builder = Sensor::builder(
        sensor_uuid,
        sensor.name,
        description,
        (sensor.longitude, sensor.latitude),
    );
    // push all provided signals trough the builder.
    for signal in sensor.signals.iter() {
        let prefix = signal.prefix.clone().unwrap();
        let sign = match prefix.sign {
            true => num_bigint::Sign::Plus,
            false => num_bigint::Sign::Minus,
        };
        let bigint = BigInt::new(sign, prefix.integer);
        let bigdecimal = BigDecimal::new(bigint, prefix.exponent);
        let quantity = match Quantity::from_str(&signal.quantity) {
            Ok(q) => q,
            Err(_) => return Err(SignalError::QuantityParseError),
        };
        let unit = match Unit::from_str(&signal.unit) {
            Ok(u) => u,
            Err(_) => return Err(SignalError::UnitParseError),
        };
        builder.add_signal(0, signal.alias.clone(), quantity, unit, bigdecimal);
    }

    Ok(builder.build())
}

#[tonic::async_trait]
impl SensorCrudService for SensorStore {
    /// Create a sensor given it's [`Uuid`].
    ///
    /// This function can fail if the provided sensor's format is incorrect or
    /// if there is any error on the side of the database(decimal type errors, ...)
    async fn create_sensor(
        &self,
        request: Request<CreateSensorRequest>,
    ) -> Result<Response<CreateSensorResponse>, Status> {
        let proto_sensor: ProtoSensor = request.into_inner().sensor.unwrap();
        let sensor = match into_sensor(proto_sensor) {
            Ok(s) => s,
            Err(e) => {
                return Ok(tonic::Response::new(CreateSensorResponse {
                    result: Some(CResult::Failures(make_signal_format_error(e))),
                }))
            }
        };
        match self.get_store().store_sensor(sensor).await {
            Ok(uuid) => Ok(tonic::Response::new(CreateSensorResponse {
                result: Some(CResult::Uuid(uuid.to_string())),
            })),
            Err(e) => {
                error!("Error setting sensor into the database {e}");
                Ok(tonic::Response::new(CreateSensorResponse {
                    result: Some(CResult::Failures(make_database_error())),
                }))
            }
        }
    }
    /// Fetch a sensor given it's [`Uuid`].
    ///
    /// This function can fail if the uuid is formated incorrectly or
    /// if the sensor with that uuid is not found in the database.
    async fn read_sensor(
        &self,
        request: Request<ReadSensorRequest>,
    ) -> Result<Response<ReadSensorResponse>, Status> {
        let req_uuid: String = request.into_inner().uuid;
        // check if the provided uuid has a valid format.
        let sensor_id: Uuid = match Uuid::parse_str(&req_uuid) {
            Ok(id) => id,
            Err(_) => {
                return Ok(tonic::Response::new(ReadSensorResponse {
                    result: Some(RResult::Failures(make_uuid_error())),
                }));
            }
        };
        // get the sensor from the database.
        // this might error if the uuid is not present in the database.
        let sensor: Sensor = match self.get_store().get_sensor(sensor_id).await {
            Ok(s) => s,
            Err(_) => {
                return Ok(tonic::Response::new(ReadSensorResponse {
                    result: Some(RResult::Failures(make_uuid_not_found_error())),
                }));
            }
        };
        // convert orm sensor into gRPC sensor.
        let sensor: ProtoSensor = into_proto_sensor(sensor);
        // create response containing requested sensor.
        Ok(tonic::Response::new(ReadSensorResponse {
            result: Some(RResult::Sensor(sensor)),
        }))
    }
    /// Update a sensor.
    ///
    /// This function can fail if the provided [`Uuid`] wasn't found in the database,
    /// if there is any error on the side of the database(decimal type errors, ...) or
    /// if the sensor has incorrect format.
    async fn update_sensor(
        &self,
        request: Request<UpdateSensorRequest>,
    ) -> Result<Response<UpdateSensorResponse>, Status> {
        let req = request.into_inner();
        let req_uuid: String = req.uuid;
        // check if the provided uuid has a valid format.
        let sensor_id: Uuid = match Uuid::parse_str(&req_uuid) {
            Ok(id) => id,
            Err(_) => {
                return Ok(tonic::Response::new(UpdateSensorResponse {
                    result: Some(UResult::Failures(make_uuid_error())),
                }));
            }
        };
        // delete sensor entry for this `sensor_id`.
        let result = self.get_store().delete_sensor(sensor_id).await;
        if result.is_err() {
            return Ok(tonic::Response::new(UpdateSensorResponse {
                result: Some(UResult::Failures(make_uuid_not_found_error())),
            }));
        }
        // create ORM sensor.
        let proto_sensor: ProtoSensor = req.sensor.unwrap();
        let mut sensor = match into_sensor(proto_sensor) {
            Ok(s) => s,
            Err(e) => {
                return Ok(tonic::Response::new(UpdateSensorResponse {
                    result: Some(UResult::Failures(make_signal_format_error(e))),
                }))
            }
        };
        // set required sensor_id.
        sensor.id = sensor_id;
        // push sensor into database.
        let result = self.get_store().store_sensor(sensor).await;
        Ok(tonic::Response::new(UpdateSensorResponse {
            result: Some(UResult::Success(result.is_ok())),
        }))
    }
    /// Delete a sensor.
    ///
    /// This function can fail if the provided [`Uuid`]'s structure is incorrect or
    /// if the uuid wan't found in the database.
    async fn delete_sensor(
        &self,
        request: Request<DeleteSensorRequest>,
    ) -> Result<Response<DeleteSensorResponse>, Status> {
        let req_uuid: String = request.into_inner().uuid;
        // check if the provided uuid has a valid format.
        let sensor_id: Uuid = match Uuid::parse_str(&req_uuid) {
            Ok(id) => id,
            Err(_) => {
                return Ok(tonic::Response::new(DeleteSensorResponse {
                    result: Some(DResult::Failures(make_uuid_error())),
                }));
            }
        };
        // delete sensor id from database.
        match self.get_store().delete_sensor(sensor_id).await {
            Ok(_) => Ok(tonic::Response::new(DeleteSensorResponse {
                result: Some(DResult::Success(true)),
            })),
            Err(_) => Ok(tonic::Response::new(DeleteSensorResponse {
                result: Some(DResult::Failures(make_uuid_not_found_error())),
            })),
        }
    }
}
