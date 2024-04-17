import type { Metadata } from 'next';
import './globals.css';
import AppWrapper from '@/components/AppWrapper';
import Provider from '@/store/Provider';

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
            <head></head>
            <body>
                <Provider>
                    <AppWrapper>{children}</AppWrapper>
                </Provider>
            </body>
        </html>
    );
}
