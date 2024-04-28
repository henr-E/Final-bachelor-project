'use client';

import { Button, Tooltip } from 'flowbite-react';
import Link from 'next/link';
import { usePathname, useRouter } from 'next/navigation';
import { PropsWithChildren, useContext } from 'react';
import { MdSensors, MdHome, MdEdit } from 'react-icons/md';
import { FaChartArea, FaArrowRight, FaBolt } from 'react-icons/fa';
import { CiLogout } from 'react-icons/ci';
import { UserContext } from '@/store/user';

// TODO: avoid hard refresh (to={Link} does not appear to work on Sidebar.Item)
function DashboardSidebar() {
    const pathName = usePathname();
    const router = useRouter();
    const [userState, dispatchUser] = useContext(UserContext);

    const SidebarItem = ({
        children,
        href,
        label,
    }: PropsWithChildren<{ href: string; label: string }>) => {
        const wrapperActiveStyles = pathName === href ? 'bg-indigo-600 text-white' : '';
        const iconActiveStyles = pathName === href ? 'text-white-800' : '';
        return (
            <Link href={href} replace>
                <div
                    className={`my-2 hover:bg-indigo-700 hover:text-white ${wrapperActiveStyles} flex items-center p-2 text-gray-900 rounded-lg hover:bg-gray-100 group`}
                >
                    <div
                        className={`w-5 h-5 transition duration-75 group-hover:text-white ${iconActiveStyles}`}
                    >
                        {children}
                    </div>
                    <span className='ms-3'>{label}</span>
                </div>
            </Link>
        );
    };

    const handleLogoutButtonClick = () => {
        localStorage.removeItem('authToken');
        dispatchUser({ type: 'logout' });
        router.back();
    };

    return (
        <div className='h-100 w-64 shadow-md z-40 bg-white flex flex-col'>
            <div className='px-3 py-4 grow'>
                <span className='ms-2 text-gray-500'>General</span>
                <SidebarItem href='/dashboard/overview' label='Overview'>
                    <MdHome size={20} />
                </SidebarItem>
                <SidebarItem href='/dashboard/editor' label='Editor'>
                    <MdEdit size={20} />
                </SidebarItem>
                <SidebarItem href='/dashboard/realtime' label='RealTime'>
                    <FaChartArea size={20} />
                </SidebarItem>
                <SidebarItem href='/dashboard/sensors' label='Sensors'>
                    <MdSensors size={20} />
                </SidebarItem>
                <SidebarItem href='/dashboard/simulation' label='Simulation'>
                    <MdSensors size={20} />
                </SidebarItem>
            </div>
            <hr />
            <div className='m-2 flex flex-row'>
                <Tooltip content='Logout'>
                    <Button
                        className='bg-transparent focus:ring-indigo-600 ring-indigo-600 enabled:hover:ring-indigo-600 enabled:hover:bg-indigo-600'
                        outline
                        onClick={handleLogoutButtonClick}
                    >
                        <CiLogout
                            size={20}
                            className='mx-4 group-hover:fill-white group-hover:border-white'
                        />
                    </Button>
                </Tooltip>
                <Button
                    fullSized
                    className='bg-transparent focus:ring-indigo-600 ring-indigo-600 enabled:hover:ring-indigo-600 enabled:hover:bg-indigo-600'
                    outline
                    onClick={() => router.replace('/dashboard/account')}
                >
                    My Account
                </Button>
            </div>
        </div>
    );
}

export default DashboardSidebar;
