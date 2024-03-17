'use client'

import { createChannel, createClient } from "nice-grpc-web";
import { TwinServiceDefinition } from '@/proto/twins/twin';
import { uiBackendServiceUrl } from "@/api/urls";
import ToastNotification from "@/components/notification/ToastNotification";
import "@/store/twins/Provider"
import {Twin, TwinContext, TwinProvider} from "@/store/twins";
import {reducer} from "next/dist/client/components/router-reducer/router-reducer";
import {useContext} from "react";

export async function createTwin(id: string, name: string, latitude:number, longitude:number , radius: number) {
    const channel = createChannel(uiBackendServiceUrl);
    const client = createClient(TwinServiceDefinition, channel);

    const request = { id, name, latitude, longitude, radius };

    try {
        const response = await client.createTwin(request);
        //if true
        if (response.createdTwin) {
            ToastNotification("success", "The twin was created successfully.");
            return true;
        } else {
            ToastNotification("error", "Bad response from server.");
        }
    }
    catch (error) {
        ToastNotification("error", "There was a problem making the twin.");
    }
    return false;
}
