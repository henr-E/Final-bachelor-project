'use client';

import { TwinFromProvider, TwinContext } from '@/store/twins';
import { Navbar, Dropdown, DropdownItem, Button, NavbarBrand } from 'flowbite-react';
import { useContext } from 'react';
import { useRouter } from 'next/navigation';
import ToastNotification from '@/components/notification/ToastNotification';
import Image from 'next/image';
import { TourControlContext } from '@/store/tour';

interface DashboardNavbarProps {
    openCreateTwinModal?: () => void;
}

function DashboardNavbar({ openCreateTwinModal }: DashboardNavbarProps) {
    const tourController = useContext(TourControlContext);
    const [twinState, dispatch] = useContext(TwinContext);
    const router = useRouter();

    const onTwinSelect = (twin: TwinFromProvider) => {
        if (twinState.current?.id != twin.id) {
            localStorage.setItem('selectedTwinID', String(twin.id));
            dispatch({ type: 'switch_twin', twin: twin });
        } else {
            ToastNotification('info', 'This twin is already selected.');
        }
    };

    const handleCreateTwinButtonClick = () => {
        if (openCreateTwinModal) {
            openCreateTwinModal();
        }
    };

    return (
        <Navbar fluid rounded className='shadow-md'>
            <NavbarBrand
                onClick={() => {
                    router.push('/');
                }}
                className={'cursor-pointer'}
            >
                <Image src='/favicons/favicon.ico' width={36} height={36} alt='Logo' />
                <span className='self-center whitespace-nowrap text-xl font-semibold dark:text-white'>
                    Digital Twin
                </span>
            </NavbarBrand>
            <div
                className={'tour-step-0-overview'}
                onClick={() => {
                    tourController?.customGoToNextTourStep(1);
                }}
            >
                <Dropdown
                    pill
                    color='indigo'
                    theme={{
                        floating: {
                            target: 'enabled:hover:bg-indigo-700 bg-indigo-600 text-white',
                        },
                    }}
                    label={twinState.current?.name ?? 'Select Twin'}
                    dismissOnClick
                >
                    {twinState.twins.map(twin => (
                        <DropdownItem key={twin.id} onClick={() => onTwinSelect(twin)}>
                            {twin.name}
                        </DropdownItem>
                    ))}
                    {
                        <div
                            className={'tour-step-1-overview'}
                            onClick={() => {
                                if (tourController?.isOpen) {
                                    //no need to check which steps the tour is doing because the classnames are all overview steps
                                    tourController?.customCloseTourAndStartAtStep(2);
                                }
                            }}
                        >
                            <DropdownItem
                                onClick={handleCreateTwinButtonClick}
                                key='create-twin'
                                style={{
                                    display: 'block',
                                    width: '100%',
                                    padding: '0.5rem 1rem',
                                    textAlign: 'center',
                                    backgroundColor: 'transparent',
                                    color: '#6366f1',
                                    borderRadius: '0.375rem',
                                    borderWidth: '1px',
                                    borderColor: '#6366f1',
                                    cursor: 'pointer',
                                    outline: 'none',
                                }}
                            >
                                Create Twin
                            </DropdownItem>
                        </div>
                    }
                </Dropdown>
            </div>
        </Navbar>
    );
}

export default DashboardNavbar;
