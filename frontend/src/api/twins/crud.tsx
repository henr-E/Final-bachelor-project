'use client';
import ToastNotification from '@/components/notification/ToastNotification';
import { presetObject, TwinServiceDefinition } from '@/proto/twins/twin';
import { ClientError, createChannel, Status } from 'nice-grpc-web';
import { uiBackendServiceUrl } from '@/api/urls';
import '@/store/twins/Provider';
import { clientAuthLayer } from '@/api/protecteRequestFactory';

export async function BackendCreateTwin(
    name: string,
    latitude: number,
    longitude: number,
    radius: number
) {
    const channel = createChannel(uiBackendServiceUrl);

    const client = clientAuthLayer.create(TwinServiceDefinition, channel);

    const request = { name: name, latitude: latitude, longitude: longitude, radius: radius };

    try {
        return await client.createTwin(request);
    } catch (error) {
        console.log(error);
        ToastNotification('error', 'There was a problem making the twin.');
    }
    return false;
}

export async function BackendGetTwins() {
    const channel = createChannel(uiBackendServiceUrl);
    const client = clientAuthLayer.create(TwinServiceDefinition, channel);
    try {
        return await client.getAllTwins({});
    } catch (error) {
        if (error instanceof ClientError && Status[error.code] == 'UNAUTHENTICATED') {
            return false;
        }
        ToastNotification('error', 'Failed to fetch all twins');
        console.error('Failed to fetch all twins:', error);
        return;
    }
}

export async function BackendDeleteTwin(twinId: number): Promise<boolean> {
    try {
        const channel = createChannel(uiBackendServiceUrl);
        const client = clientAuthLayer.create(TwinServiceDefinition, channel);
        const request = { id: twinId };
        await client.deleteTwin(request);
        return true;
    } catch (error) {
        ToastNotification('error', 'Failed to delete twin.');
        console.error('Failed to delete twin:', error);
        return false;
    }
}

export async function BackendGetBuildings(twinId: number) {
    try {
        ToastNotification('success', 'Your twin is being loaded.');
        const channel = createChannel(uiBackendServiceUrl);
        const client = clientAuthLayer.create(TwinServiceDefinition, channel);
        const request = { id: twinId };

        return await client.getBuildings(request);
    } catch (error) {
        ToastNotification('error', 'Failed to fetch buildings');
        console.error('Failed to fetch buildings:', error);
        return;
    }
}

export async function BackendDeleteBuilding(buildingId: number): Promise<boolean> {
    try {
        const channel = createChannel(uiBackendServiceUrl);
        const client = clientAuthLayer.create(TwinServiceDefinition, channel);
        const request = { id: buildingId };
        await client.deleteBuilding(request);
        return true;
    } catch (error) {
        ToastNotification('error', 'Failed to delete building.');
        console.error('Failed to delete building:', error);
        return false;
    }
}

export async function BackendUndoDeleteBuilding(buildingId: number): Promise<boolean> {
    try {
        const channel = createChannel(uiBackendServiceUrl);
        const client = clientAuthLayer.create(TwinServiceDefinition, channel);
        const request = { id: buildingId };
        await client.undoDeleteBuilding(request);
        return true;
    } catch (error) {
        ToastNotification('error', 'Failed to restore building.');
        console.error('Failed to restore building:', error);
        return false;
    }
}

export async function BackendCreatePreset(
    presetName: string,
    presetInfo: string,
    presetIs_edge: boolean
) {
    try {
        const channel = createChannel(uiBackendServiceUrl);
        const client = clientAuthLayer.create(TwinServiceDefinition, channel);
        const request: presetObject = { name: presetName, info: presetInfo, isEdge: presetIs_edge };
        return await client.createPreset(request);
    } catch (error) {
        ToastNotification('error', 'Failed to create preset.');
        console.error('Failed to create preset.', error);
        return;
    }
}

export async function BackendGetAllPreset() {
    try {
        const channel = createChannel(uiBackendServiceUrl);
        const client = clientAuthLayer.create(TwinServiceDefinition, channel);
        const response = await client.getAllPreset({});
        return response.preset;
    } catch (error) {
        ToastNotification('error', 'Failed to fetch preset.');
        console.error('Failed to fetch preset.', error);
        return;
    }
}
