'use client';

import React from 'react';
import { CityProvider } from './city';
import { UserProvider } from './user';
import { TwinProvider } from './twins';
import { SensorProvider } from './sensor';

function Provider({ children }: { children: React.ReactNode }) {
    return (
        <UserProvider>
            <CityProvider>
                <TwinProvider>
                    <SensorProvider>
                        {children}
                    </SensorProvider>
                </TwinProvider>
            </CityProvider>
        </UserProvider>
    );
}

export default Provider;
