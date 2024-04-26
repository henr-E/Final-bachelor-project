'use client';
import { createChannel, createClient } from 'nice-grpc-web';
import { uiBackendServiceUrl } from '@/api/urls';
import {
    CrudFailureReason,
    Sensor,
    SensorCRUDServiceClient,
    SensorCRUDServiceDefinition,
} from '@/proto/sensor/sensor-crud';
import { QuantityWithUnits } from '@/store/sensor';
import ToastNotification from '@/components/notification/ToastNotification';

export function failureReasonToString(reason: CrudFailureReason): string {
    switch (reason) {
        case CrudFailureReason.UNRECOGNIZED:
            return 'Failure reason not recognized';
        case CrudFailureReason.UUID_FORMAT_ERROR:
            return 'Uuid has incorrect format';
        case CrudFailureReason.INVALID_UNIT_ERROR:
            return 'Unit is invalid (mismatch with backend?)';
        case CrudFailureReason.INVALID_QUANTITY_ERROR:
            return 'Quantity invalid (mismatch with backend?)';
        case CrudFailureReason.UUID_NOT_PRESENT_ERROR:
            return 'Sensor uuid not found in database';
        case CrudFailureReason.DATABASE_INSERTION_ERROR:
            return 'Failed to insert into database';
        case CrudFailureReason.DUPLICATE_QUANTITY_ERROR:
            return 'Quantity must be unique';
    }
}

export async function BackendGetQuantityWithUnits(): Promise<Record<string, QuantityWithUnits>> {
    const channel = createChannel(uiBackendServiceUrl);
    const client: SensorCRUDServiceClient = createClient(SensorCRUDServiceDefinition, channel);

    let quantitiesWithUnits: Record<string, QuantityWithUnits> = {};
    try {
        for await (const response of client.getQuantitiesAndUnits({})) {
            const quantity = response.quantity;
            if (quantity === undefined || quantitiesWithUnits.hasOwnProperty(quantity.id)) {
                // Should not be reachable as the backend is streaming these to the frontend. If
                // so the backend is doing something wrong. This should not be an error for the
                // user as ignoring this quantity could be considered as "handling" the error.
                // If not units are associated with the quantity, ignore it.
                break;
            }

            quantitiesWithUnits[quantity.id] = {
                quantity: {
                    id: quantity.id,
                    repr: quantity.repr.toUpperCase(),
                },
                units: response.units.map(u => ({
                    id: u.id,
                    repr: u.repr.toUpperCase(),
                })),
                baseUnit: response.baseUnit,
            };
        }
    } catch (error) {
        ToastNotification('error', `Failed to fetch sensors: ${error}`);
        console.error('Failed to fetch sensors', error);
    }

    return quantitiesWithUnits;
}

export async function BackendGetSensors(twinId: number): Promise<Sensor[]> {
    const channel = createChannel(uiBackendServiceUrl);
    const client: SensorCRUDServiceClient = createClient(SensorCRUDServiceDefinition, channel);

    let sensors: Sensor[] = [];
    try {
        for await (const response of client.getSensors({ twinId: twinId })) {
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
                latitude: sensor.latitude,
                longitude: sensor.longitude,
                signals: sensor.signals.map(s => ({
                    id: s.id,
                    quantity: s.quantity,
                    unit: s.unit,
                    ingestionUnit: s.ingestionUnit,
                    alias: s.alias,
                    prefix: s.prefix || { sign: false, integer: [1], exponent: 0 },
                })),
                twinId: sensor.twinId,
                buildingId: sensor.buildingId,
            });
        }

        return sensors;
    } catch (error) {
        ToastNotification('error', `Failed to fetch sensors`);
        console.error('Failed to fetch sensors', error);
        return [];
    }
}

export async function BackendCreateSensor(sensor: Sensor): Promise<boolean> {
    const channel = createChannel(uiBackendServiceUrl);
    const client: SensorCRUDServiceClient = createClient(SensorCRUDServiceDefinition, channel);

    let success = true;
    try {
        const response = await client.createSensor({ sensor: sensor });
        if (response.failures !== undefined) {
            const failures = response.failures.reasons.map(r => failureReasonToString(r));
            console.error(`Creating sensor failed because of following reasons: ${failures}`);
            success = false;
        } else {
            // Update the sensor ID with the one created on the backend. The id that is sent to the
            // backend is ignored right now. Even when this is changed, it would not hurt to do
            // this.
            sensor.id = response.uuid ? response.uuid : sensor.id;
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

export async function BackendDeleteSensor(sensor_id: string): Promise<boolean> {
    const channel = createChannel(uiBackendServiceUrl);
    const client: SensorCRUDServiceClient = createClient(SensorCRUDServiceDefinition, channel);

    let success = true;
    try {
        const response = await client.deleteSensor({ uuid: sensor_id });
        if (response.failures !== undefined) {
            const failures = response.failures.reasons.map(r => failureReasonToString(r));
            console.error(`Deleting sensor failed because of following reasons: ${failures}`);
            success = false;
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
