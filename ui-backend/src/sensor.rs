use chrono::{DateTime, TimeDelta, Utc};
use futures::poll;
use num_bigint::{BigInt, Sign};
use proto::frontend::sensor_data_fetching::{
    AllSensorData, AllSensorDataEntry, AllSensorDataMessage, AllSensorDataRequest,
    SensorDataFetchingService, SignalToValuesMap, SignalValue as ProtoSignalValue,
    SignalValues as ProtoSignalValues, SingleSensorDataMessage, SingleSensorDataRequest,
};
use proto::frontend::{
    get_quantities_and_units_response::{Quantity as ProtoQuantity, Unit as ProtoUnit},
    BigDecimal as ProtoBigDecimal, CreateSensorRequest, CreateSensorResponse, CrudFailure,
    CrudFailureReason, DeleteSensorRequest, DeleteSensorResponse, GetQuantitiesAndUnitsResponse,
    GetSensorsRequest, GetSensorsResponse, ReadSensorRequest, ReadSensorResponse,
    Sensor as ProtoSensor, SensorCrudService, Signal as ProtoSignal, UpdateSensorRequest,
    UpdateSensorResponse,
};
use sensor_store::signal::SignalValues;
use sensor_store::{Quantity, Sensor, SensorStore as SensorStoreInner, Unit};
use sqlx::types::BigDecimal;
use std::collections::HashSet;
use std::task::Poll;
use std::time::Duration;
use std::{collections::HashMap, pin::Pin, str::FromStr};
use thiserror::Error;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, Streaming};
use tracing::{debug, error};
use uuid::Uuid;

const LIVE_DATA_FETCH_INTERVAL: Duration = Duration::from_millis(5000);

#[derive(Clone)]
pub struct SensorStore(SensorStoreInner);

impl SensorStore {
    pub async fn new() -> Self {
        Self(
            SensorStoreInner::new()
                .await
                .expect("Could not create sensor store."),
        )
    }

    pub fn as_inner(&self) -> &SensorStoreInner {
        &self.0
    }
}

impl std::ops::Deref for SensorStore {
    type Target = SensorStoreInner;

    fn deref(&self) -> &Self::Target {
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

fn into_proto_big_decimal(decimal: &BigDecimal) -> ProtoBigDecimal {
    let (big_int, exponent) = decimal.as_bigint_and_exponent();
    let (sign, integer) = big_int.to_u32_digits();
    ProtoBigDecimal {
        integer: integer.to_vec(),
        sign: sign == Sign::Minus,
        // Scale/exponent is inverted in the `BigDecimal` type. See
        // [documentation](https://docs.rs/bigdecimal/0.4.3/src/bigdecimal/lib.rs.html#191)
        // for more info.
        exponent: -exponent,
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
            let prefix = into_proto_big_decimal(&s.prefix);
            // create expected gRPC signal type.
            ProtoSignal {
                id: s.id,
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
    // push all provided signals through the builder.
    for signal in sensor.signals.into_iter() {
        let prefix = signal.prefix.unwrap_or_else(ProtoBigDecimal::one);
        let sign = match prefix.sign {
            true => Sign::Minus,
            false => Sign::Plus,
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
        _request: Request<()>,
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

        Ok(Response::new(Box::pin(futures::stream::iter(
            Quantity::all()
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

        debug!("fetching all sensors");
        let req = request.into_inner();
        // Collect all sensors into vec and return that stream. This temporary collection
        // is needed to avoid lifetime errors.
        let sensors: Vec<Result<GetSensorsResponse, Status>> = self
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

        Ok(Response::new(Box::pin(futures::stream::iter(sensors))))
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
        debug!("creating sensor: {:?}", request.sensor);
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
        match self.store_sensor(sensor).await {
            Ok(uuid) => CreateSensorResponse::uuid(uuid).into(),
            Err(e) => {
                error!("Error setting sensor into the database {e}");
                CreateSensorResponse::failures(CrudFailure::new_database_error()).into()
            }
        }
    }

    /// Fetch a sensor given it's [`Uuid`].
    ///
    /// This function can fail if the uuid is formatted incorrectly or
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
        let Ok(sensor) = self.get_sensor(sensor_id).await else {
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
        if self.as_inner().delete_sensor(sensor_id).await.is_err() {
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
        let result = self.store_sensor(sensor).await;
        UpdateSensorResponse::success(result.is_ok()).into()
    }

    /// Delete a sensor.
    ///
    /// This function can fail if the provided [`Uuid`]'s structure is incorrect or
    /// if the uuid wasn't found in the database.
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
        match self.as_inner().delete_sensor(sensor_id).await {
            Ok(_) => DeleteSensorResponse::success(true),
            Err(_) => DeleteSensorResponse::failures(CrudFailure::new_uuid_not_found_error()),
        }
        .into()
    }
}

fn into_proto_signal_values(signal_values: SignalValues) -> ProtoSignalValues {
    ProtoSignalValues {
        value: signal_values
            .values
            .into_iter()
            .map(|sv| ProtoSignalValue {
                timestamp: sv.timestamp.timestamp(),
                value: Some(into_proto_big_decimal(&sv.value)),
            })
            .collect(),
    }
}

async fn fetch_sensor_data_into_hashmap<'a, E>(
    sensor_store: &SensorStoreInner,
    sensors: impl futures::stream::Stream<Item = Result<Sensor<'a>, E>> + Send,
    lookback: Duration,
    internal_err_format: fn(&dyn std::fmt::Display) -> Status,
) -> Result<HashMap<String, SignalToValuesMap>, Status>
where
    E: std::fmt::Display + Send,
{
    use futures::stream::StreamExt;

    let mut sensor_signal_values_iter = sensors
        .then(|s| async {
            match s {
                Ok(s) => match s
                    .signal_values_for_interval_since_now(sensor_store, lookback)
                    .await
                {
                    Ok(svs) => Ok(svs
                        .unwrap_or(HashMap::with_capacity(0))
                        .into_iter()
                        .map(|(id, sv)| (id, into_proto_signal_values(sv)))
                        .collect::<HashMap<i32, ProtoSignalValues>>()),
                    Err(e) => Err(internal_err_format(&e)),
                }
                .map(|svs| (s.id, svs)),
                Err(e) => Err(internal_err_format(&e)),
            }
        })
        // Needed because of async internal workings with `Unpin`.
        .boxed();

    let mut result = HashMap::new();
    while let Some(signal_values) = sensor_signal_values_iter.next().await {
        let (id, signal_values) = match signal_values {
            Ok(signal_values) => signal_values,
            Err(e) => return Err(internal_err_format(&e)),
        };

        // Do not add sensor that have no associated data.
        if signal_values.is_empty() {
            continue;
        }

        let old_value = result.insert(
            id.to_string(),
            SignalToValuesMap {
                signals: signal_values,
            },
        );
        if old_value.is_some() {
            return Err(internal_err_format(&format!(
                "duplicate sensor entry (should be unreachable): duplicate `{}`",
                id
            )));
        };
    }

    Ok(result)
}

#[tonic::async_trait]
impl SensorDataFetchingService for SensorStore {
    async fn fetch_sensor_data_all_sensors(
        &self,
        request: Request<AllSensorDataRequest>,
    ) -> Result<Response<AllSensorData>, Status> {
        fn internal_err_format(err: &dyn std::fmt::Display) -> Status {
            Status::internal(format!("could not fetch data for all sensors: {}", err))
        }

        debug!("fetching data for all sensors");
        let req = request.into_inner();

        let lookback = Duration::from_secs(req.lookback);
        let sensors = self
            .get_all_sensors()
            .await
            .map_err(|e| internal_err_format(&e))?;

        let sensor_data =
            fetch_sensor_data_into_hashmap(self.as_inner(), sensors, lookback, internal_err_format)
                .await?;

        Ok(Response::new(AllSensorData {
            sensors: sensor_data,
        }))
    }

    async fn fetch_sensor_data_single_sensor(
        &self,
        request: Request<SingleSensorDataRequest>,
    ) -> Result<Response<SignalToValuesMap>, Status> {
        fn internal_err_format(err: &dyn std::fmt::Display) -> Status {
            Status::internal(format!("could not fetch data for a single sensor: {}", err))
        }

        debug!("fetching data for a single sensor");
        let req = request.into_inner();

        let lookback = Duration::from_secs(req.lookback);
        let Ok(sensor_id) = Uuid::from_str(&req.sensor_id) else {
            return Err(Status::invalid_argument("sensor id is not a valid uuid"));
        };

        let sensor = match self.get_sensor(sensor_id).await {
            Ok(sensor) => sensor,
            Err(e) => {
                return Err(match e {
                    sensor_store::Error::SensorIdNotFound => {
                        Status::invalid_argument("sensor id does not exist")
                    }
                    e @ sensor_store::Error::Sqlx(_) => internal_err_format(&e),
                })
            }
        };

        let signal_values = sensor
            .signal_values_for_interval_since_now(self, lookback)
            .await
            .map_err(|e| internal_err_format(&e))?;
        let result = signal_values
            .unwrap_or(HashMap::with_capacity(0))
            .into_iter()
            .map(|(id, svs)| (id, into_proto_signal_values(svs)))
            .collect::<HashMap<_, _>>();

        Ok(Response::new(SignalToValuesMap { signals: result }))
    }

    type FetchSensorDataAllSensorsStreamStream = ReceiverStream<Result<AllSensorDataEntry, Status>>;
    async fn fetch_sensor_data_all_sensors_stream(
        &self,
        request: Request<Streaming<AllSensorDataMessage>>,
    ) -> Result<Response<Self::FetchSensorDataAllSensorsStreamStream>, Status> {
        fn internal_err_format(err: &dyn std::fmt::Display) -> Status {
            Status::internal(format!(
                "internal error during live sensor data stream for all sensors: {}",
                err
            ))
        }

        async fn send_on_channel(
            tx: &mpsc::Sender<Result<AllSensorDataEntry, Status>>,
            data: Result<AllSensorDataEntry, Status>,
        ) -> Result<(), Status> {
            if tx.is_closed() {
                return Err(Status::unavailable("The channel has been closed."));
            }

            if let Err(err) = tx.send(data).await {
                error!("Sending data on the sensor data stream failed: {}.", err);
                return Err::<(), _>(internal_err_format(&err));
            }
            Ok(())
        }

        /// Find the largest timestamp nested deep in the data.
        fn last_data_timestamp(data: &HashMap<String, SignalToValuesMap>) -> Option<DateTime<Utc>> {
            // The ugliest function I have ever written. Sorry if you have to debug this :)
            data.iter()
                .flat_map(|(_, svm)| svm.signals.values())
                .flat_map(|vs| vs.value.iter())
                .map(|v| v.timestamp)
                .max()
                .map(|t| {
                    DateTime::from_timestamp(t, 0).expect("failed to convert seconds to timestamp")
                })
        }

        use proto::frontend::sensor_data_fetching::all_sensor_data_message::StartOrShutdown;
        use tokio_stream::StreamExt;

        debug!("starting sensor data stream for all sensors");
        let mut req_stream = request.into_inner();

        // The first message sent over the stream should be a request containing setup parameters.
        // The second and last one, should be the shutdown signal.
        let Some(Ok(AllSensorDataMessage {
            start_or_shutdown: Some(StartOrShutdown::Request(req)),
        })) = req_stream.next().await
        else {
            return Err(Status::invalid_argument(
                "expected a single start message as the first stream entry",
            ));
        };

        let lookback = Duration::from_secs(req.lookback);
        // Avoid reference to self by cloning the inner SensorStore.
        let sensor_store = self.as_inner().clone();

        let (tx, rx) = mpsc::channel(4);

        tokio::spawn(async move {
            let mut last_timestamp = None;

            loop {
                if let Poll::Ready(Some(Ok(AllSensorDataMessage {
                    start_or_shutdown: Some(req),
                }))) = poll!(Box::pin(req_stream.next()))
                {
                    match req {
                        StartOrShutdown::Request(_) => return Err(Status::invalid_argument(
                            "expected a shutdown signal as the second value sent over the channel",
                        )),
                        StartOrShutdown::Shutdown(_) => {
                            // Shutdown signal received from the client. Close the loop and clean
                            // up remaining resources.
                            break;
                        }
                    }
                }

                let sensors = match sensor_store.get_all_sensors().await {
                    Ok(s) => s,
                    Err(e) => {
                        // As this is the first return of the tokio spawn body, and because an
                        // async block does not allow you to define the return type explicitly, the
                        // first return has to be an explicit one.
                        //
                        // Clippy tries to help you here by telling you this can be converted to
                        // `?` syntax, however if you apply the suggestion, compilation will fail.
                        // Hence the allow attribute here.
                        #[allow(clippy::question_mark)]
                        if let Err(err) = send_on_channel(&tx, Err(internal_err_format(&e))).await {
                            return Err::<(), _>(err);
                        };
                        continue;
                    }
                };

                // Get the new lookback based on the last timestamp present in the data last sent.
                // NOTE: The conversion to seconds and back to duration is because postgresql does
                // not support higher interval precision.
                let lookback = match last_timestamp {
                    Some(lt) => {
                        let delta: TimeDelta = Utc::now() - lt;
                        Duration::from_secs(
                            u64::try_from(delta.num_seconds())
                                .expect("cannot create duration from negative amount of seconds"),
                        )
                    }
                    None => lookback,
                };

                let sensor_data = match fetch_sensor_data_into_hashmap(
                    &sensor_store,
                    sensors,
                    lookback,
                    internal_err_format,
                )
                .await
                {
                    Ok(sd) => sd,
                    Err(e) => {
                        send_on_channel(&tx, Err(internal_err_format(&e))).await?;
                        continue;
                    }
                };

                // Skip sending when no sensor data was found.
                if sensor_data.is_empty() {
                    continue;
                }

                // Get the last timestamp data was successfully fetched at.
                let last_data_timestamp = last_data_timestamp(&sensor_data);

                tracing::trace!("sending on channel");
                send_on_channel(
                    &tx,
                    Ok(AllSensorDataEntry {
                        sensors: sensor_data,
                    }),
                )
                .await?;

                // Update the timestamp the data was last requested at.
                if let Some(last_data_timestamp) = last_data_timestamp {
                    last_timestamp = Some(last_data_timestamp);
                }

                tokio::time::sleep(LIVE_DATA_FETCH_INTERVAL).await;
            }

            debug!("Closing down single sensor data stream");
            Ok(())
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    type FetchSensorDataSingleSensorStreamStream =
        ReceiverStream<Result<SignalToValuesMap, Status>>;
    async fn fetch_sensor_data_single_sensor_stream(
        &self,
        request: Request<Streaming<SingleSensorDataMessage>>,
    ) -> Result<Response<Self::FetchSensorDataSingleSensorStreamStream>, Status> {
        fn internal_err_format(err: &dyn std::fmt::Display) -> Status {
            Status::internal(format!(
                "internal error during live sensor data stream for sensors: {}",
                err
            ))
        }

        async fn send_on_channel(
            tx: &mpsc::Sender<Result<SignalToValuesMap, Status>>,
            data: Result<SignalToValuesMap, Status>,
        ) -> Result<(), Status> {
            if tx.is_closed() {
                return Err(Status::internal("e"));
            }
            if let Err(err) = tx.send(data).await {
                error!("Sending data on the sensor data stream failed: {}.", err);
                return Err::<(), _>(internal_err_format(&err));
            }
            Ok(())
        }

        /// Find the largest timestamp nested deep in the data.
        fn last_data_timestamp(data: &HashMap<i32, ProtoSignalValues>) -> Option<DateTime<Utc>> {
            // The ugliest function I have ever written. Sorry if you have to debug this :)
            data.iter()
                .flat_map(|(_, svm)| svm.value.iter())
                .map(|vs| vs.timestamp)
                .max()
                .map(|t| {
                    DateTime::from_timestamp(t, 0).expect("failed to convert seconds to timestamp")
                })
        }

        use proto::frontend::sensor_data_fetching::single_sensor_data_message::StartOrShutdown;
        use tokio_stream::StreamExt;

        debug!("starting sensor data stream for a single sensor");
        let mut req_stream = request.into_inner();

        // The first message sent over the stream should be a request containing setup parameters.
        // The second and last one, should be the shutdown signal.
        let Some(Ok(SingleSensorDataMessage {
            start_or_shutdown: Some(StartOrShutdown::Request(req)),
        })) = req_stream.next().await
        else {
            return Err(Status::invalid_argument(
                "expected a single start message as the first stream entry",
            ));
        };

        let lookback = Duration::from_secs(req.lookback);
        let Ok(sensor_id) = Uuid::from_str(&req.sensor_id) else {
            return Err(Status::invalid_argument("sensor id is not a valid uuid"));
        };
        let sensor_store = self.as_inner().clone();

        let (tx, rx) = mpsc::channel(4);

        tokio::spawn(async move {
            // Avoid reference to self by cloning the inner SensorStore.
            let sensor = match sensor_store.get_sensor(sensor_id).await {
                Ok(sensor) => sensor,
                Err(e) => {
                    return Err::<(), Status>(match e {
                        sensor_store::Error::SensorIdNotFound => {
                            Status::invalid_argument("sensor id does not exist")
                        }
                        e @ sensor_store::Error::Sqlx(_) => internal_err_format(&e),
                    })
                }
            };

            let mut last_timestamp = None;

            loop {
                if let Poll::Ready(Some(Ok(SingleSensorDataMessage {
                    start_or_shutdown: Some(req),
                }))) = poll!(Box::pin(req_stream.next()))
                {
                    match req {
                        StartOrShutdown::Request(_) => return Err(Status::invalid_argument(
                            "expected a shutdown signal as the second value sent over the channel",
                        )),
                        StartOrShutdown::Shutdown(_) => {
                            // Shutdown signal received from the client. Close the loop and clean
                            // up remaining resources.
                            debug!("Shutdown signal received");
                            break;
                        }
                    }
                }

                // Get the new lookback based on the last timestamp present in the data last sent.
                // NOTE: The conversion to seconds and back to duration is because postgresql does
                // not support higher interval precision.
                let lookback = match last_timestamp {
                    Some(lt) => {
                        let delta: TimeDelta = Utc::now() - lt;
                        Duration::from_secs(
                            u64::try_from(delta.num_seconds())
                                .expect("cannot create duration from negative amount of seconds"),
                        )
                    }
                    None => lookback,
                };

                let signal_values = match sensor
                    .signal_values_for_interval_since_now(&sensor_store, lookback)
                    .await
                {
                    Ok(s) => s,
                    Err(e) => {
                        send_on_channel(&tx, Err(internal_err_format(&e))).await?;
                        continue;
                    }
                };
                let result = signal_values
                    .unwrap_or(HashMap::with_capacity(0))
                    .into_iter()
                    .map(|(id, svs)| (id, into_proto_signal_values(svs)))
                    .collect::<HashMap<_, _>>();

                // Skip sending when no sensor data was found.
                if result.is_empty() {
                    // if there is no new sensor data, sleep for a bit.
                    tokio::time::sleep(LIVE_DATA_FETCH_INTERVAL).await;
                    continue;
                }

                // Get the last timestamp data was successfully fetched at.
                let last_data_timestamp = last_data_timestamp(&result);

                tracing::trace!("sending on channel");
                send_on_channel(&tx, Ok(SignalToValuesMap { signals: result })).await?;

                // Update the timestamp the data was last requested at.
                if let Some(last_data_timestamp) = last_data_timestamp {
                    last_timestamp = Some(last_data_timestamp);
                }

                tokio::time::sleep(LIVE_DATA_FETCH_INTERVAL).await;
            }

            debug!("Closing down single sensor data stream");
            Ok(())
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
