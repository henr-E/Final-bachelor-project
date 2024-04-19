'use client';

import { UserContext } from '@/store/user';
import {
    Navbar,
    NavbarBrand,
    NavbarLink,
    NavbarToggle,
    NavbarCollapse,
    Button,
} from 'flowbite-react';
import { jwtDecode } from 'jwt-decode';
import { useRouter } from 'next/navigation';
import { useContext } from 'react';

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
        <Navbar fluid rounded>
            <NavbarBrand>
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
    );
}

export default MainNavbar;
