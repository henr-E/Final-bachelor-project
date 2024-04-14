'use client'
import {createChannel, createClient} from "nice-grpc-web";
import {
    CreateSimulationParams,
    Simulation,
    SimulationInterfaceServiceDefinition,
    Simulations
} from "@/proto/simulation/frontend";
import {uiBackendServiceUrl} from "@/api/urls";
import {SensorCRUDServiceClient, SensorCRUDServiceDefinition} from "@/proto/sensor/sensor-crud";
import ToastNotification from "@/components/notification/ToastNotification";

//todo the twinId should be a number as it is stored as SERIAL in the database and as number in the frontend
export async function BackendGetSimulations(twinId: string): Promise<Simulations> {
    try {
        const channel = createChannel(uiBackendServiceUrl);
        const client = createClient(SimulationInterfaceServiceDefinition, channel);
        return await client.getAllSimulations({twinId});
    } catch (error) {
        console.error("Failed to load simulations:", error);
        //todo problem with return statement
        // @ts-ignore
        return [];
    }
}

export async function BackendCreateSimulation(simulationParams: CreateSimulationParams): Promise<any> {
    const channel = createChannel(uiBackendServiceUrl);
    const client = createClient(SimulationInterfaceServiceDefinition, channel);

    try {
        return client.createSimulation(simulationParams);
    } catch (error) {
        console.error('Failed to create simulation', error);
    }
}

export async function BackendDeleteSimulation(
    simulation_id: string
): Promise<boolean> {
    return true;
}

export async function BackendGetSimulationDetails(simulationId: string): Promise<Simulation> {
    const channel = createChannel(uiBackendServiceUrl);
    const client = createClient(SimulationInterfaceServiceDefinition, channel);
    return await client.getSimulation({uuid: simulationId});
}




















