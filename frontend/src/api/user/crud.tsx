'use client'
import {createChannel, createClient,} from "nice-grpc-web";

import {AuthenticationServiceClient, AuthenticationServiceDefinition, User} from "@/proto/authentication/auth";
import {uiBackendServiceUrl} from "@/api/urls";

export async function BackendLogin(userData: User): Promise<any> {
    const channel = createChannel(uiBackendServiceUrl);
    const client: AuthenticationServiceClient = createClient(AuthenticationServiceDefinition, channel);
    return client.loginUser({user: {...userData}});
}

export async function BackendRegister(userData: User): Promise<any> {
    const channel = createChannel(uiBackendServiceUrl);
    const client: AuthenticationServiceClient = createClient(AuthenticationServiceDefinition, channel);
    return client.registerUser({user: {...userData}});
}
