'use client';

import { Button, Dropdown, DropdownItem, Tooltip } from 'flowbite-react';
import Link from 'next/link';
import { usePathname, useRouter } from 'next/navigation';
import { PropsWithChildren, useContext } from 'react';
import { MdSensors, MdHome, MdEdit } from 'react-icons/md';
import { FaChartArea } from 'react-icons/fa';
import { CiLogout } from 'react-icons/ci';
import { UserContext } from '@/store/user';
import { overviewSteps } from '../store/tour/steps/overview';
import { editorSteps } from '../store/tour/steps/editor';
import { sensorsSteps } from '../store/tour/steps/sensors';
import { realtimeSteps } from '../store/tour/steps/realtime';
import { simulationSteps } from '../store/tour/steps/simulation';
import { TourControlContext } from '@/store/tour';
import ToastNotification from '@/components/notification/ToastNotification';

function DashboardSidebar() {
    const pathName = usePathname();
    const router = useRouter();
    const [userState, dispatchUser] = useContext(UserContext);

    const tourController = useContext(TourControlContext);

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

                <Dropdown
                    pill
                    color='indigo'
                    theme={{
                        floating: {
                            target: 'enabled:hover:bg-indigo-700 bg-indigo-600 text-white',
                        },
                    }}
                    label={'Tutorials'}
                    dismissOnClick
                >
                    <DropdownItem
                        onClick={() => {
                            router.push('/dashboard/overview');
                            tourController?.setIsOpen(true);
                            tourController?.setCurrentStep(0);
                            if (tourController?.customSetSteps) {
                                tourController?.customSetSteps(overviewSteps);
                            }
                            ToastNotification('info', 'Loading tutorial');
                        }}
                    >
                        Overview tutorial
                    </DropdownItem>
                    <DropdownItem
                        onClick={() => {
                            router.push('/dashboard/editor');
                            setTimeout(() => {
                                tourController?.setIsOpen(true);
                                tourController?.setCurrentStep(0);
                                if (tourController?.customSetSteps) {
                                    tourController?.customSetSteps(editorSteps);
                                }
                            }, 3000);
                            ToastNotification('info', 'Loading tutorial');
                        }}
                    >
                        Editor tutorial
                    </DropdownItem>
                    <DropdownItem
                        onClick={() => {
                            router.push('/dashboard/realtime');
                            setTimeout(() => {
                                tourController?.setIsOpen(true);
                                tourController?.setCurrentStep(0);
                                if (tourController?.customSetSteps) {
                                    tourController?.customSetSteps(realtimeSteps);
                                }
                            }, 3000);
                            ToastNotification('info', 'Loading tutorial');
                        }}
                    >
                        Realtime tutorial
                    </DropdownItem>
                    <DropdownItem
                        onClick={() => {
                            router.push('/dashboard/sensors');
                            setTimeout(() => {
                                tourController?.setIsOpen(true);
                                tourController?.setCurrentStep(0);
                                if (tourController?.customSetSteps) {
                                    tourController?.customSetSteps(sensorsSteps);
                                }
                            }, 3000);
                            ToastNotification('info', 'Loading tutorial');
                        }}
                    >
                        Sensors tutorial
                    </DropdownItem>
                    <DropdownItem
                        onClick={() => {
                            router.push('/dashboard/simulation');
                            setTimeout(() => {
                                tourController?.setIsOpen(true);
                                tourController?.setCurrentStep(0);
                                if (tourController?.customSetSteps) {
                                    tourController?.customSetSteps(simulationSteps);
                                }
                            }, 3000);
                            ToastNotification('info', 'Loading tutorial');
                        }}
                    >
                        Simulations tutorial
                    </DropdownItem>
                </Dropdown>
            </div>
        </div>
    );
}

export default DashboardSidebar;
