//! Sensor data fetching service.
//!
//! This module includes the implemenation of the `SensorDataFetchingService` and some helper
//! functions to convert from native rust types to protobuf message types and back.

use super::{into_proto_big_decimal, SensorStore};
use chrono::{DateTime, TimeDelta, Utc};
use futures::poll;
use proto::frontend::sensor_data_fetching::{
    AllSensorData, AllSensorDataEntry, AllSensorDataMessage, AllSensorDataRequest,
    SensorDataFetchingService, SignalToValuesMap, SignalValue as ProtoSignalValue,
    SignalValues as ProtoSignalValues, SingleSensorDataMessage, SingleSensorDataRequest,
};
use sensor_store::{signal::SignalValues, Sensor, SensorStore as SensorStoreInner};
use std::{collections::HashMap, task::Poll, time::Duration};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, Streaming};
use tracing::{debug, error};
use uuid::Uuid;

const LIVE_DATA_FETCH_INTERVAL: Duration = Duration::from_millis(5000);

/// Convert [`SignalValues`] type into protobuf message `SignalValues` type.
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

/// Fetch sensor data from the [`SensorStore`](SensorStoreInner) based on the specified lookback
/// for all sensors in te `sensors` stream.
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
    /// Fetch sensor data for all sensors currently registered from `now` until the given
    /// `lookback` into the past.
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

    /// Fetch sensor data for a single sensor currently registered from `now` until the given
    /// `lookback` into the past.
    async fn fetch_sensor_data_single_sensor(
        &self,
        request: Request<SingleSensorDataRequest>,
    ) -> Result<Response<SignalToValuesMap>, Status> {
        fn internal_err_format(err: &dyn std::fmt::Display) -> Status {
            Status::internal(format!("could not fetch data for a single sensor: {}", err))
        }

        use std::str::FromStr;

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
    /// Stream of sensor data for all sensors.
    ///
    /// When new sensors are registered, they will be included in the next response if data is
    /// ingested for them.
    ///
    /// The `lookback` is only used for the first response. Following responses inlcude all data
    /// that has not been included since the last response.
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
                // Check for the shutdown signal from the client.
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
    /// Stream of sensor data for a single sensor.
    ///
    /// When new sensors register, they will be included in the next response if data is ingested
    /// for them.
    ///
    /// The `lookback` is only used for the first response. Following responses inlcude all data
    /// that has not been included since the last response.
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
        use std::str::FromStr;
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
                // Check for the shutdown signal from the client.
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
                if !result.is_empty() {
                    // Get the last timestamp data was successfully fetched at.
                    let last_data_timestamp = last_data_timestamp(&result);

                    tracing::trace!("sending on channel");
                    send_on_channel(&tx, Ok(SignalToValuesMap { signals: result })).await?;

                    // Update the timestamp the data was last requested at.
                    if let Some(last_data_timestamp) = last_data_timestamp {
                        last_timestamp = Some(last_data_timestamp);
                    }
                }

                // if there is no new sensor data, sleep for a bit.
                tokio::time::sleep(LIVE_DATA_FETCH_INTERVAL).await;
            }

            debug!("Closing down single sensor data stream");
            Ok(())
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
