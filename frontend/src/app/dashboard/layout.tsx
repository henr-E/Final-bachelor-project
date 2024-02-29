'use client';

import { useContext, useEffect, useState } from "react";
import DashboardNavbar from "@/components/DashboardNavbar";
import DashboardSidebar from "@/components/DashboardSidebar";
import { UserContext } from "@/store/user";
import { TwinContext } from "@/store/twins";
import CreateTwinModal from "@/components/modals/CreateTwinModal";

export default function DashboardLayout({
    children,
}: Readonly<{
    children: React.ReactNode;
}>) {
    const [userState, dispatchUser] = useContext(UserContext);
    const [twinState, dispatchTwin] = useContext(TwinContext);
    const [isCreateTwinModalOpen, setIsCreateTwinModalOpen] = useState(false);

    useEffect(() => {
        if (userState.token && twinState.twins.length > 0 && !twinState.current) {
            dispatchTwin({ type: 'switch_twin', twin: twinState.twins[0] });
        }
    }, [userState.token, twinState.twins, twinState.current]);

    return (
        <div className="flex flex-col h-screen">
            <DashboardNavbar openCreateTwinModal={() => setIsCreateTwinModalOpen(true)}/>
            <CreateTwinModal isCreateTwinModalOpen={isCreateTwinModalOpen} closeCreateTwinModal={() => setIsCreateTwinModalOpen(false)} />
            <div className="flex flex-row grow">
                <DashboardSidebar />
                <div className="px-12 py-8 grow">
                    {children}
                </div>
            </div>
        </div>
    );
}
