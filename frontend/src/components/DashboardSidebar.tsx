'use client';

import { Sidebar } from 'flowbite-react';
import { HiUser, HiViewBoards, HiSun, HiDesktopComputer } from 'react-icons/hi';

function DashboardSidebar() {
    return (
        <Sidebar className="h-full shadow-md">
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
                        <Sidebar.Item>Prediction</Sidebar.Item>
                        <Sidebar.Item>Risk Analysis</Sidebar.Item>
                        <Sidebar.Item>Optimization</Sidebar.Item>
                    </Sidebar.Collapse>
                </Sidebar.ItemGroup>
            </Sidebar.Items>
        </Sidebar>
    );
}

export default DashboardSidebar;

