'use client';

import React from 'react';
import { UserProvider } from './user';
import { TwinProvider } from './twins';
import { SensorProvider } from '@/store/sensor';
import { ToastContainer } from '@/components/notification/ToastNotification';

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
