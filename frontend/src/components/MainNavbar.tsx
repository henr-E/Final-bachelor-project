'use client';
import { UserContext } from '@/store/user';
import {
    Button,
    Navbar,
    NavbarBrand,
    NavbarCollapse,
    NavbarLink,
    NavbarToggle,
} from 'flowbite-react';
import { usePathname, useRouter } from 'next/navigation';
import { useContext } from 'react';
import Image from 'next/image';
import { Tooltip } from 'react-tooltip';
import { simulationSteps } from '@/store/tour/steps/simulation';
import { TourControlContext } from '@/store/tour';
import { startupSteps } from '@/store/tour/steps/startup';
import ToastNotification from '@/components/notification/ToastNotification';

interface MainNavbarProps {
    openLoginModal: () => void;
    openRegisterModal: () => void;
}

function MainNavbar({ openLoginModal, openRegisterModal }: MainNavbarProps) {
    const [userState, dispatch] = useContext(UserContext);
    const router = useRouter();
    const pathName = usePathname();
    const tourController = useContext(TourControlContext);

    const handleGetStartedButtonClick = () => {
        const token = localStorage.getItem('authToken');

        if (token) {
            router.push('/dashboard');
        } else {
            openLoginModal();
        }
    };

    const handleRegisterButtonClick = () => {
        openRegisterModal();
    };

    return (
        <>
            <Navbar fluid rounded>
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
                <div className='flex md:order-2'>
                    {!userState.user && (
                        <Button
                            className={'tour-step-2-startup'}
                            onClick={() => {
                                handleRegisterButtonClick();
                                tourController?.customGoToNextTourStep(1);
                            }}
                            color='green'
                        >
                            Register
                        </Button>
                    )}
                    <div className='ml-2'>
                        {!userState.user && (
                            <Button
                                className={'tour-step-5-startup'}
                                color='indigo'
                                onClick={() => {
                                    handleGetStartedButtonClick();
                                    //login button
                                    tourController?.customGoToNextTourStep(1);
                                }}
                            >
                                Login
                            </Button>
                        )}
                        {userState.user && (
                            <Button
                                className={'tour-step-2-startup '}
                                color='indigo'
                                onClick={() => {
                                    handleGetStartedButtonClick();
                                    //dashboard button
                                    tourController?.customGoToNextTourStep(1);
                                    tourController?.setIsOpen(false);
                                }}
                            >
                                Dashboard
                            </Button>
                        )}
                        <NavbarToggle />
                    </div>
                </div>
                <NavbarCollapse>
                    <NavbarLink
                        className={
                            pathName === '/' ? 'underline decoration-solid text-indigo-600' : ''
                        }
                        href='/'
                    >
                        Home
                    </NavbarLink>

                    <NavbarLink
                        className={
                            pathName === '/docs'
                                ? 'tour-step-1-startup underline decoration-solid text-indigo-600'
                                : 'tour-step-1-startup'
                        }
                        href='/docs'
                    >
                        Docs
                    </NavbarLink>

                    <div
                        className={'tour-step-0-startup cursor-pointer'}
                        onClick={() => {
                            tourController?.setIsOpen(true);
                            tourController?.setCurrentStep(0);
                            if (tourController?.customSetSteps) {
                                tourController?.customSetSteps(startupSteps);
                            }
                            ToastNotification('info', 'Loading tutorial');
                        }}
                    >
                        Startup Tutorial
                    </div>
                </NavbarCollapse>
            </Navbar>
        </>
    );
}

export default MainNavbar;
