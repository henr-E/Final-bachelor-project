'use client';
import { createChannel } from 'nice-grpc-web';
import {
    CreateSimulationParams,
    Simulation,
    SimulationInterfaceServiceDefinition,
    Simulations,
    SimulationStatus,
} from '@/proto/simulation/frontend';
import { ComponentsInfo, Simulators } from '@/proto/simulation/simulation-manager';
import { uiBackendServiceUrl } from '@/api/urls';
import ToastNotification from '@/components/notification/ToastNotification';
import { clientAuthLayer } from '@/api/protecteRequestFactory';

//todo the twinId should be a number as it is stored as SERIAL in the database and as number in the frontend
export async function BackendGetSimulations(twinId: string): Promise<Simulations> {
    try {
        const channel = createChannel(uiBackendServiceUrl);
        const client = clientAuthLayer.create(SimulationInterfaceServiceDefinition, channel);
        return await client.getAllSimulations({ twinId });
    } catch (error) {
        console.error('Failed to load simulations:', error);
        //todo problem with return statement
        // @ts-ignore
        return [];
    }
}

export async function BackendCreateSimulation(
    simulationParams: CreateSimulationParams
): Promise<any> {
    const channel = createChannel(uiBackendServiceUrl);
    const client = clientAuthLayer.create(SimulationInterfaceServiceDefinition, channel);

    try {
        return client.createSimulation(simulationParams);
    } catch (error) {
        console.error('Failed to create simulation', error);
    }
}

export async function BackendDeleteSimulation(simulationId: number): Promise<boolean> {
    try {
        const channel = createChannel(uiBackendServiceUrl);
        const client = clientAuthLayer.create(SimulationInterfaceServiceDefinition, channel);
        const request = { id: simulationId };
        const response = await client.deleteSimulation(request);
        return response.deleted;
    } catch (error) {
        ToastNotification('error', 'Failed to delete twin.');
        console.error('Failed to delete twin:', error);
        return false;
    }
}

export async function BackendGetSimulationDetails(simulationId: string): Promise<Simulation> {
    try {
        const channel = createChannel(uiBackendServiceUrl);
        const client = clientAuthLayer.create(SimulationInterfaceServiceDefinition, channel);
        return await client.getSimulation({ uuid: simulationId });
    } catch (error) {
        ToastNotification(
            'error',
            'Simulation details request failed (see console for more details)'
        );
        console.error('Simulation details request failed:', error);
        return {
            creationDateTime: 0,
            endDateTime: 0,
            framesLoaded: 0,
            id: 0,
            name: 'Failed to load',
            startDateTime: 0,
            status: SimulationStatus.FAILED,
        };
    }
}

export async function BackendGetSimulators(): Promise<Simulators> {
    try {
        const channel = createChannel(uiBackendServiceUrl);
        const client = clientAuthLayer.create(SimulationInterfaceServiceDefinition, channel);
        return await client.getSimulators(Request);
    } catch (error) {
        ToastNotification('error', 'Failed to load simulators (see console for more details)');
        console.error('Failed to load simulators:', error);
        return { simulator: [] };
    }
}

export async function BackendGetComponent(): Promise<ComponentsInfo> {
    try {
        const channel = createChannel(uiBackendServiceUrl);
        const client = clientAuthLayer.create(SimulationInterfaceServiceDefinition, channel);
        return await client.getComponents({});
    } catch (error) {
        ToastNotification(
            'error',
            'Failed to load simulation components (see console for more details)'
        );
        console.error('Failed to load simulation components:', error);
        return { components: {} };
    }
}
