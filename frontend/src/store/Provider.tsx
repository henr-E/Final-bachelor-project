'use client';

import React from 'react';
import { UserProvider } from './user';
import { TwinProvider } from './twins';

function Provider({ children }: { children: React.ReactNode }) {
    return (
        <UserProvider>
            <TwinProvider>{children}</TwinProvider>
        </UserProvider>
    );
}

export default Provider;
