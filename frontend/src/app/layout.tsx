import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";
import AppWrapper from "@/components/AppWrapper";
import Provider from "@/store/Provider";

const inter = Inter({ subsets: ["latin"] });

export const metadata: Metadata = {
    title: "Digital Twin",
    description: "Gain realtime insight into how your city uses energy.",
};

export default function RootLayout({
    children,
}: Readonly<{
    children: React.ReactNode;
}>) {
    return (
        <html lang="en">
            <body className={inter.className}>
                <Provider>
                    <AppWrapper>
                        {children}
                    </AppWrapper>
                </Provider>
            </body>
        </html>
    );
}
