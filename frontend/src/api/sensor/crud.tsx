import { Sensor, SensorAction } from '@/store/sensor';
import { createChannel, createClient } from 'nice-grpc-web';
import { uiBackendServiceUrl } from '@/api/urls';
import {
    SensorCRUDServiceDefinition,
    SensorCRUDServiceClient,
    BigInt,
    CrudFailureReason,
    Sensor as ProtoSensor,
} from '@/proto/sensor/sensor-crud';
import ToastNotification from '@/components/notification/ToastNotification';

function failureReasonToString(reason: CrudFailureReason): string {
    switch (reason) {
        case CrudFailureReason.UNRECOGNIZED:
            return 'Failure reason not recognized';
        case CrudFailureReason.UUID_FORMAT_ERROR:
            return 'Uuid has incorrect format';
        case CrudFailureReason.INVALID_UNIT_ERROR:
            return 'Unit is invalid (mismatch with backend?)';
        case CrudFailureReason.INVALID_QUANTITY_ERROR:
            return 'Quantity invalid (mismach with backend?)';
        case CrudFailureReason.UUID_NOT_PRESENT_ERROR:
            return 'Sensor uuid not found in database';
        case CrudFailureReason.DATABASE_INSERTION_ERROR:
            return 'Failed to insert into database';
    }
}

function sensorToProtoSensor(sensor: Sensor): ProtoSensor {
    return {
        id: sensor.id,
        name: sensor.name,
        description: sensor.description,
        latitude: sensor.location.lat,
        longitude: sensor.location.lng,
        signals: sensor.signals.map(s => ({
            alias: s.ingestionColumnAlias,
            quantity: s.quantity,
            unit: s.unit,
            ingestionUnit: s.ingestionUnit,
            prefix: s.ingestionPrefix,
        })),
    };
}

async function fetchSensors(dispatch: React.Dispatch<SensorAction>): Promise<void> {
    const channel = createChannel(uiBackendServiceUrl);
    const client: SensorCRUDServiceClient = createClient(SensorCRUDServiceDefinition, channel);

    let sensors: Sensor[] = [];
    try {
        for await (const response of client.getSensors({})) {
            if (response.sensor === undefined) {
                // Should not be reachable as the backend is streaming sensors to the frontend. If
                // so the backend is doing something wrong. This should not be an error for the
                // user as ignoring this sensor could be considered as "handling" the error.
                break;
            }

            const sensor = response.sensor;
            sensors.push({
                id: sensor.id,
                name: sensor.name,
                description: sensor.description,
                location: {
                    lng: sensor.longitude,
                    lat: sensor.latitude,
                },
                signals: sensor.signals.map(s => ({
                    quantity: s.quantity,
                    unit: s.unit,
                    ingestionUnit: s.ingestionUnit,
                    ingestionColumnAlias: s.alias,
                    ingestionPrefix: s.prefix || { sign: false, integer: [1], exponent: 0 },
                })),
            });
        }
    } catch (error) {
        ToastNotification('error', `Failed to fetch sensors`);
        console.error('Failed to fetch sensors', error);
    }

    dispatch({ type: 'load_sensors', sensors: sensors });
}

async function createSensor(
    dispatch: React.Dispatch<SensorAction>,
    sensor: Sensor
): Promise<boolean> {
    const channel = createChannel(uiBackendServiceUrl);
    const client: SensorCRUDServiceClient = createClient(SensorCRUDServiceDefinition, channel);

    let success = true;
    try {
        const response = await client.createSensor({ sensor: sensorToProtoSensor(sensor) });
        if (response.failures !== undefined) {
            const failures = response.failures.reasons.map(r => failureReasonToString(r));
            console.error(`Creating sensor failed because of following reasons: ${failures}`);
            success = false;
        } else {
            // Update the sensor ID with the one created on the backend. The id that is sent to the
            // backend is ignored right now. Even when this is changed, it would not hurt to do
            // this.
            sensor.id = response.uuid ? response.uuid : sensor.id;
            dispatch({ type: 'create_sensor', sensor: sensor });
        }
    } catch (error) {
        console.error('Failed to create sensor', error);
        success = false;
    }

    if (!success) {
        ToastNotification('error', 'Creating sensor failed');
    }
    return success;
}

async function deleteSensor(
    dispatch: React.Dispatch<SensorAction>,
    sensor_id: string
): Promise<boolean> {
    const channel = createChannel(uiBackendServiceUrl);
    const client: SensorCRUDServiceClient = createClient(SensorCRUDServiceDefinition, channel);

    let success = true;
    try {
        const response = await client.deleteSensor({ uuid: sensor_id });
        if (response.failures !== undefined) {
            const failures = response.failures.reasons.map(r => failureReasonToString(r));
            console.error(`Deleting sensor failed because of following reasons: ${failures}`);
            success = false;
        } else {
            dispatch({ type: 'delete_sensor', sensorId: sensor_id });
        }
    } catch (error) {
        console.error('Failed to delete sensor', error);
        success = false;
    }

    if (!success) {
        ToastNotification('error', 'Deleting sensor failed');
    }
    return success;
}

async function updateSensor(
    dispatch: React.Dispatch<SensorAction>,
    sensor: Sensor
): Promise<boolean> {
    const channel = createChannel(uiBackendServiceUrl);
    const client: SensorCRUDServiceClient = createClient(SensorCRUDServiceDefinition, channel);

    let success = true;
    try {
        const response = await client.updateSensor({ uuid: sensor.id, sensor: { ...sensor } });
        if (response.failures !== undefined) {
            const failures = response.failures.reasons.map(r => failureReasonToString(r));
            console.error(`Updating sensor failed because of following reasons: ${failures}`);
            success = false;
        } else {
            dispatch({ type: 'update_sensor', sensorId: sensor.id, sensor: sensor });
        }
    } catch (error) {
        console.error('Failed to update sensor', error);
        success = false;
    }

    if (!success) {
        ToastNotification('error', 'Updating sensor failed');
    }
    return success;
}

export { fetchSensors, createSensor, deleteSensor, updateSensor };
