'use client';

import { useContext, useEffect, useState } from "react";
import DashboardNavbar from "@/components/DashboardNavbar";
import DashboardSidebar from "@/components/DashboardSidebar";
import { UserContext } from "@/store/user";
import { CityContext } from "@/store/city";

export default function DashboardLayout({
    children,
}: Readonly<{
    children: React.ReactNode;
}>) {
    const [userState, dispatchUser] = useContext(UserContext);
    const [cityState, dispatchCity] = useContext(CityContext);

    useEffect(() => {
        if (userState.token) { }
    }, [userState.token]);

    return (
        <div className="flex flex-col h-screen">
            <DashboardNavbar />
            <div className="flex grow">
                <DashboardSidebar />
                <div className="grow px-12 py-8">
                    {children}
                </div>
            </div>
        </div>
    );
}
