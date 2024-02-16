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
        if (userState.token) {
            fetch('/api/v1/city', {
                headers: {
                    'Content-Type': 'application/json',
                    authorization: userState.token
                }
            })
                .then(resp => resp.json())
                .then(data => dispatchCity({ type: 'load_cities', cities: data }))
                .catch(err => {
                    // only executed when the request 'failed', such as when the server couldn't be reached
                    // not executed when the backend returns a HTTP status code other than 200
                    console.log(err.message);
                    // redirect to error page or dispay a notification
                });
        }
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
