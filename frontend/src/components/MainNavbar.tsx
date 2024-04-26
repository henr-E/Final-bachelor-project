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
import { useRouter } from 'next/navigation';
import { useContext } from 'react';
import Image from 'next/image';

interface MainNavbarProps {
    openLoginModal: () => void;
    openRegisterModal: () => void;
}

function MainNavbar({ openLoginModal, openRegisterModal }: MainNavbarProps) {
    const [userState, dispatch] = useContext(UserContext);
    const router = useRouter();

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
                <NavbarBrand>
                    <Image src='/favicons/favicon.ico' width={36} height={36} alt='Logo' />
                    <span className='self-center whitespace-nowrap text-xl font-semibold dark:text-white'>
                        Digital Twin
                    </span>
                </NavbarBrand>
                <div className='flex md:order-2'>
                    {!userState.user && (
                        <Button onClick={handleRegisterButtonClick} color='green'>
                            Register
                        </Button>
                    )}
                    <div className='ml-2'>
                        <Button color='indigo' onClick={handleGetStartedButtonClick}>
                            {userState.user ? 'Dashboard' : 'Login'}
                        </Button>
                        <NavbarToggle />
                    </div>
                </div>
                <NavbarCollapse>
                    <NavbarLink href='#' active>
                        Home
                    </NavbarLink>
                    <NavbarLink href='#'>About</NavbarLink>
                    <NavbarLink href='#'>Docs</NavbarLink>
                    <NavbarLink href='#'>Contact</NavbarLink>
                </NavbarCollapse>
            </Navbar>
            <div className='flex flex-col justify-center items-center h-screen'>
                <Image src='/favicons/favicon.ico' width={405} height={509} alt='Logo' />
                <h1 className='text-indigo-700 text-3xl font-bold mt-4'>DIGITAL TWIN</h1>
            </div>
        </>
    );
}

export default MainNavbar;
