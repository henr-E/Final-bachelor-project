'use client';

import React from 'react';
import { CityProvider } from './city';
import { UserProvider } from './user';

function Provider({ children }: { children: React.ReactNode }) {
    return (
        <UserProvider>
            <CityProvider>
                {children}
            </CityProvider>
        </UserProvider>
    );
}

export default Provider;
