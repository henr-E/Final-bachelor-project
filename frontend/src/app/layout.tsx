import type { Metadata } from 'next';
import './globals.css';
import AppWrapper from '@/components/AppWrapper';
import Provider from '@/store/Provider';
import { ToastContainer } from '@/components/notification/ToastNotification';
import React from 'react';

export const metadata: Metadata = {
    title: 'Digital Twin',
    description: 'Gain realtime insight into how your city uses energy.',
};

export default function RootLayout({
    children,
}: Readonly<{
    children: React.ReactNode;
}>) {
    return (
        <html lang='en'>
            <head>
                <link rel='icon' href='/favicons/favicon.ico' />
                <title>Digital Twin</title>
                <meta
                    name='description'
                    content='Gain realtime insight into how your city uses energy.'
                />
            </head>
            <body>
                <Provider>
                    <AppWrapper>{children}</AppWrapper>
                </Provider>
            </body>
        </html>
    );
}
