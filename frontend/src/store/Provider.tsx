'use client';

import React from 'react';
import { UserProvider } from './user';
import { TwinProvider } from './twins';
import { SensorProvider } from '@/store/sensor';

function Provider({ children }: { children: React.ReactNode }) {
    return (
        <UserProvider>
            <TwinProvider>
                <SensorProvider>{children}</SensorProvider>
            </TwinProvider>
        </UserProvider>
    );
}

export default Provider;
