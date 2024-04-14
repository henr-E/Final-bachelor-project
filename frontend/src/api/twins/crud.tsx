'use client';
import ToastNotification from "@/components/notification/ToastNotification";
import {buildingObject, TwinServiceDefinition} from '@/proto/twins/twin';
import {createChannel, createClient} from "nice-grpc-web";
import {uiBackendServiceUrl} from "@/api/urls";
import "@/store/twins/Provider"

export async function BackendCreateTwin(name: string, latitude: number, longitude: number, radius: number) {
    const channel = createChannel(uiBackendServiceUrl);
    const client = createClient(TwinServiceDefinition, channel);

    const request = {name: name, latitude: latitude, longitude: longitude, radius: radius};

    try {
        return await client.createTwin(request);
    } catch (error) {
        console.log(error);
        ToastNotification("error", "There was a problem making the twin.");
    }
    return false;
}

export async function BackendGetTwins(){
    const channel = createChannel(uiBackendServiceUrl);
    const client = createClient(TwinServiceDefinition, channel);
    try {
        return await client.getAllTwins({});

    } catch (error) {
        ToastNotification("error","Failed to fetch all twins")
            console.error("Failed to fetch all twins:", error);
        return;
    }
}

export async function BackendGetBuildings(twinId: number) {
    try {
        ToastNotification("success", "Your twin is being loaded.");
        const channel = createChannel(uiBackendServiceUrl);
        const client = createClient(TwinServiceDefinition, channel);
        const request = {id: twinId};

        return await client.getBuildings(request);
    } catch (error) {
        ToastNotification("error","Failed to fetch all buildings")
        console.error("Failed to fetch buildings:", error);
        return;
    }
}

export async function BackendDeleteBuilding(buildingId: number): Promise<boolean> {
    try {
        const channel = createChannel(uiBackendServiceUrl);
        const client = createClient(TwinServiceDefinition, channel);
        const request = {id: buildingId};
        const response = await client.deleteBuilding(request);
        return response.deleted;

    } catch (error) {
        ToastNotification("error", "Failed to delete building.");
        console.error("Failed to delete building:", error);
        return false;
    }
}

export async function BackendUndoDeleteBuilding(buildingId: number): Promise<boolean> {
    try {
        const channel = createChannel(uiBackendServiceUrl);
        const client = createClient(TwinServiceDefinition, channel);
        const request = {id: buildingId};
        const response = await client.undoDeleteBuilding(request);
        return response.undone
    } catch (error) {
        ToastNotification("error", "Failed to restore building.");
        console.error("Failed to restore building:", error);
        return false;
    }
}

