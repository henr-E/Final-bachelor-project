'use client'

import { createChannel, createClient } from "nice-grpc-web";
import { TwinServiceDefinition } from '@/proto/twins/twin';
import { uiBackendServiceUrl } from "@/api/urls";
import ToastNotification from "@/components/notification/ToastNotification";
import "@/store/twins/Provider"

export async function createTwin(name: string, latitude:number, longitude:number , radius: number) {
    const channel = createChannel(uiBackendServiceUrl);
    const client = createClient(TwinServiceDefinition, channel);

    const request = {name: name,  latitude: latitude, longitude: longitude, radius: radius };

    try {
        const response = await client.createTwin(request);
        ToastNotification("success", "The twin was created successfully.");
        return response.id;
    }
    catch (error) {
        console.log(error);
        ToastNotification("error", "There was a problem making the twin.");
    }
    return false;
}
