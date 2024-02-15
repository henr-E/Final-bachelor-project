'use client';

import { Sidebar } from 'flowbite-react';
import { BiBuoy } from 'react-icons/bi';
import { HiArrowSmRight, HiChartPie, HiInbox, HiShoppingBag, HiTable, HiUser, HiViewBoards } from 'react-icons/hi';

function DashboardSidebar() {
    return (
        <Sidebar>
            <Sidebar.Items>
                <Sidebar.ItemGroup>
                    <Sidebar.Item href="#" icon={HiViewBoards}>
                        My Twins
                    </Sidebar.Item>
                    <Sidebar.Item href="#" icon={HiUser}>
                        Account
                    </Sidebar.Item>
                </Sidebar.ItemGroup>
                <Sidebar.ItemGroup>
                    <Sidebar.Item href="#">
                        Use case 1
                    </Sidebar.Item>
                    <Sidebar.Item href="#">
                        Use case 2
                    </Sidebar.Item>
                    <Sidebar.Item href="#">
                        Use case 3
                    </Sidebar.Item>
                    <Sidebar.Item href="#">
                        Use case 4
                    </Sidebar.Item>
                </Sidebar.ItemGroup>
            </Sidebar.Items>
        </Sidebar>
    );
}

export default DashboardSidebar;

