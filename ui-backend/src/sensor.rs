use num_bigint::{BigInt, Sign};
use proto::frontend::{
    get_quantities_and_units_response::{Quantity as ProtoQuantity, Unit as ProtoUnit},
    BigInt as ProtoBigInt, CreateSensorRequest, CreateSensorResponse, CrudFailure,
    CrudFailureReason, DeleteSensorRequest, DeleteSensorResponse, GetQuantitiesAndUnitsResponse,
    GetSensorsRequest, GetSensorsResponse, ReadSensorRequest, ReadSensorResponse,
    Sensor as ProtoSensor, SensorCrudService, Signal as ProtoSignal, UpdateSensorRequest,
    UpdateSensorResponse,
};
use sensor_store::{Quantity, Sensor, SensorStore as SensorStoreInner, Unit};
use sqlx::types::BigDecimal;
use std::collections::HashSet;
use std::{pin::Pin, str::FromStr};
use thiserror::Error;
use tonic::{Request, Response, Status};
use tracing::error;
use uuid::Uuid;

pub struct SensorStore(SensorStoreInner);

impl SensorStore {
    pub async fn new() -> Self {
        Self(
            SensorStoreInner::new()
                .await
                .expect("Could not create sensor store."),
        )
    }
}

impl std::convert::AsRef<SensorStoreInner> for SensorStore {
    fn as_ref(&self) -> &SensorStoreInner {
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

impl From<SignalError> for CrudFailure {
    fn from(val: SignalError) -> Self {
        CrudFailure::new_single(match val {
            SignalError::QuantityParseError => CrudFailureReason::InvalidQuantityError,
            SignalError::UnitParseError => CrudFailureReason::InvalidUnitError,
        })
    }
}

/// Transforms ORM sensor to expected gRPC sensor type.
///
/// This operation may fail, if the prefix exceeds u64 integer part.
fn into_proto_sensor(sensor: Sensor) -> ProtoSensor {
    // for every signal from orm signal
    let signals = sensor
        .signals()
        .iter()
        .map(|s| {
            // extract the prefix.
            let (big_int, exponent) = s.prefix.as_bigint_and_exponent();
            let (sign, integer) = big_int.to_u32_digits();
            let prefix: ProtoBigInt = ProtoBigInt {
                integer: integer.to_vec(),
                sign: sign == Sign::Minus,
                exponent: -exponent,
            };
            // create expected gRPC signal type.
            ProtoSignal {
                alias: s.name.to_string(),
                quantity: s.quantity.to_string(),
                unit: s.unit.base_unit().to_string(),
                ingestion_unit: s.unit.to_string(),
                prefix: Some(prefix),
            }
        })
        .collect::<Vec<_>>();

    ProtoSensor {
        name: sensor.name.to_string(),
        id: sensor.id.to_string(),
        description: sensor.description.unwrap_or_default().to_string(),
        longitude: sensor.location.0,
        latitude: sensor.location.1,
        signals,
        twin_id: sensor.twin_id,
        building_id: sensor.building_id,
    }
}

fn into_sensor(sensor: ProtoSensor) -> Result<Sensor<'static>, SignalError> {
    // Create a random uuid for the sensor.
    let sensor_uuid = Uuid::now_v7();
    let description = (!sensor.description.is_empty()).then_some(sensor.description);
    // build an orm sensor.
    let mut builder = Sensor::builder(
        sensor_uuid,
        sensor.name,
        description,
        (sensor.longitude, sensor.latitude),
        sensor.twin_id,
        sensor.building_id,
    );
    // push all provided signals trough the builder.
    for signal in sensor.signals.into_iter() {
        let prefix = signal.prefix.unwrap_or_else(ProtoBigInt::one);
        let sign = match prefix.sign {
            true => num_bigint::Sign::Minus,
            false => num_bigint::Sign::Plus,
        };
        let bigint = BigInt::new(sign, prefix.integer);
        // Scale/exponent is inverted in the `BigDecimal` type. See
        // [documentation](https://docs.rs/bigdecimal/0.4.3/src/bigdecimal/lib.rs.html#191)
        // for more info.
        let bigdecimal = BigDecimal::new(bigint, -prefix.exponent);
        let quantity = match Quantity::from_str(&signal.quantity) {
            Ok(q) => q,
            Err(_) => return Err(SignalError::QuantityParseError),
        };
        let unit = match Unit::from_str(&signal.ingestion_unit) {
            Ok(u) => u,
            Err(_) => return Err(SignalError::UnitParseError),
        };
        builder.add_signal(0, signal.alias.clone(), quantity, unit, bigdecimal);
    }

    Ok(builder.build())
}

#[tonic::async_trait]
impl SensorCrudService for SensorStore {
    type GetQuantitiesAndUnitsStream = Pin<
        Box<
            dyn tokio_stream::Stream<Item = Result<GetQuantitiesAndUnitsResponse, Status>>
                + Send
                + 'static,
        >,
    >;
    /// Get all supported quantities with associated units.
    async fn get_quantities_and_units(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<Response<Self::GetQuantitiesAndUnitsStream>, Status> {
        fn response_from_quantity(q: Quantity) -> GetQuantitiesAndUnitsResponse {
            GetQuantitiesAndUnitsResponse {
                quantity: Some(ProtoQuantity {
                    id: q.to_string(),
                    repr: q.to_string(),
                }),
                units: q
                    .associated_units()
                    .into_iter()
                    .map(|u| ProtoUnit {
                        id: u.to_string(),
                        repr: u.to_string(),
                    })
                    .collect::<Vec<_>>(),
                base_unit: q.associated_base_unit().to_string(),
            }
        }

        Ok(tonic::Response::new(Box::pin(futures::stream::iter(
            sensor_store::Quantity::all()
                .into_iter()
                .map(|q| Ok(response_from_quantity(q))),
        ))))
    }

    type GetSensorsStream = Pin<
        Box<dyn tokio_stream::Stream<Item = Result<GetSensorsResponse, Status>> + Send + 'static>,
    >;
    /// Get all sensors registered in the database.
    async fn get_sensors(
        &self,
        request: Request<GetSensorsRequest>,
    ) -> Result<Response<Self::GetSensorsStream>, Status> {
        use futures::stream::StreamExt;

        tracing::debug!("fetching all sensors");
        let req = request.into_inner();
        // Collect all sensors into vec and return that stream. This temporary collection
        // is needed to avoid lifetime errors.
        let sensors: Vec<Result<GetSensorsResponse, Status>> = self
            .as_ref()
            .get_all_sensors_for_twin(req.twin_id)
            .await
            .map_err(|e| Status::internal(format!("could not get all sensors: {}", e)))?
            .map(|s| match s {
                Ok(s) => Ok(GetSensorsResponse {
                    sensor: Some(into_proto_sensor(s)),
                }),
                Err(e) => Err(Status::internal(e.to_string())),
            })
            .collect()
            .await;

        Ok(tonic::Response::new(Box::pin(futures::stream::iter(
            sensors,
        ))))
    }

    /// Create a sensor given it's [`Uuid`].
    ///
    /// This function can fail if the provided sensor's format is incorrect or
    /// if there is any error on the side of the database(decimal type errors, ...)
    async fn create_sensor(
        &self,
        request: Request<CreateSensorRequest>,
    ) -> Result<Response<CreateSensorResponse>, Status> {
        let request = request.into_inner();
        tracing::debug!("creating sensor: {:?}", request.sensor);
        let Some(proto_sensor) = request.sensor else {
            return Err(Status::invalid_argument("sensor field must be set"));
        };

        let unique_signals = proto_sensor
            .signals
            .iter()
            .map(|s| &s.quantity)
            .collect::<HashSet<_>>();
        if proto_sensor.signals.len() != unique_signals.len() {
            return CreateSensorResponse::failures(CrudFailure::new_single(
                CrudFailureReason::DuplicateQuantityError,
            ))
            .into();
        }

        let sensor = match into_sensor(proto_sensor) {
            Ok(s) => s,
            Err(e) => {
                return CreateSensorResponse::failures(e.into()).into();
            }
        };

        match self.as_ref().store_sensor(sensor).await {
            Ok(uuid) => CreateSensorResponse::uuid(uuid).into(),
            Err(e) => {
                error!("Error setting sensor into the database {e}");
                CreateSensorResponse::failures(CrudFailure::new_database_error()).into()
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
        let Ok(sensor_id) = Uuid::parse_str(&req_uuid) else {
            return ReadSensorResponse::failures(CrudFailure::new_uuid_format_error()).into();
        };
        // get the sensor from the database.
        // this might error if the uuid is not present in the database.
        let Ok(sensor) = self.as_ref().get_sensor(sensor_id).await else {
            return ReadSensorResponse::failures(CrudFailure::new_uuid_not_found_error()).into();
        };
        // convert orm sensor into gRPC sensor.
        let sensor: ProtoSensor = into_proto_sensor(sensor);
        // create response containing requested sensor.
        ReadSensorResponse::sensor(sensor).into()
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
        let Ok(sensor_id) = Uuid::parse_str(&req_uuid) else {
            return UpdateSensorResponse::failures(CrudFailure::new_uuid_format_error()).into();
        };
        // delete sensor entry for this `sensor_id`.
        if self.as_ref().delete_sensor(sensor_id).await.is_err() {
            return UpdateSensorResponse::failures(CrudFailure::new_uuid_not_found_error()).into();
        }
        // create ORM sensor.
        let Some(proto_sensor) = req.sensor else {
            return Err(Status::invalid_argument("sensor field must be set"));
        };
        let mut sensor = match into_sensor(proto_sensor) {
            Ok(s) => s,
            Err(e) => {
                return UpdateSensorResponse::failures(e.into()).into();
            }
        };
        // set required sensor_id.
        sensor.id = sensor_id;
        // push sensor into database.
        let result = self.as_ref().store_sensor(sensor).await;
        UpdateSensorResponse::success(result.is_ok()).into()
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
        let Ok(sensor_id) = Uuid::parse_str(&req_uuid) else {
            return DeleteSensorResponse::failures(CrudFailure::new_uuid_format_error()).into();
        };
        // delete sensor id from database.
        match self.as_ref().delete_sensor(sensor_id).await {
            Ok(_) => DeleteSensorResponse::success(true),
            Err(_) => DeleteSensorResponse::failures(CrudFailure::new_uuid_not_found_error()),
        }
        .into()
    }
}
