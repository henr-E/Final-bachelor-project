'use client';

import { Sidebar } from 'flowbite-react';
import { HiUser, HiViewBoards, HiDesktopComputer } from 'react-icons/hi';

// TODO: avoid hard refresh (to={Link} does not appear to work on Sidebar.Item)
function DashboardSidebar() {
    return (
        <Sidebar className="h-100 shadow-md">
            <Sidebar.Items>
                <Sidebar.ItemGroup>
                    <Sidebar.Item icon={HiViewBoards} href="/dashboard/twins">
                        My Twins
                    </Sidebar.Item>
                    <Sidebar.Item icon={HiUser} href="/dashboard/account">
                        Account
                    </Sidebar.Item>
                </Sidebar.ItemGroup>
                <Sidebar.ItemGroup>
                    <Sidebar.Collapse icon={HiDesktopComputer} label="Simulations" open={true}>
                        <Sidebar.Item href="/dashboard/prediction">Prediction</Sidebar.Item>
                        <Sidebar.Item href="/dashboard/risk-analysis">Risk Analysis</Sidebar.Item>
                        <Sidebar.Item href="/dashboard/optimization">Optimization</Sidebar.Item>
                    </Sidebar.Collapse>
                </Sidebar.ItemGroup>
            </Sidebar.Items>
        </Sidebar>
    );
}

export default DashboardSidebar;

