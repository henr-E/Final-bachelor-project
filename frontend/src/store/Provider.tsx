'use client';

import React from 'react';
import { UserProvider } from './user';
import { TwinProvider } from './twins';
import { SensorProvider } from './sensor';
import {SimulationProvider} from "@/store/simulation";

function Provider({ children }: { children: React.ReactNode }) {
    return (
        <UserProvider>
            <TwinProvider>
                <SensorProvider>
                    <SimulationProvider>{children}</SimulationProvider>
                </SensorProvider>
            </TwinProvider>
        </UserProvider>
    );
}

export default Provider;
