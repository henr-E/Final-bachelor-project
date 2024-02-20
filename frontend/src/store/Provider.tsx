'use client';

import React from 'react';
import { CityProvider } from './city';
import { UserProvider } from './user';
import { TwinProvider } from './twins';

function Provider({ children }: { children: React.ReactNode }) {
    return (
        <UserProvider>
            <CityProvider>
                <TwinProvider>
                    {children}
                </TwinProvider>
            </CityProvider>
        </UserProvider>
    );
}

export default Provider;
