'use client';

import React, { useState } from 'react';
import MainNavbar from '@/components/MainNavbar';
import LoginModal from '@/components/modals/LoginModal';
import RegisterModal from '@/components/modals/RegisterModal';
import Image from 'next/image';

export default function Home() {
    const [isLoginModalOpen, setIsLoginModalOpen] = useState(false);
    const [isRegisterModalOpen, setIsRegisterModalOpen] = useState(false);

    return (
        <>
            <MainNavbar
                openLoginModal={() => setIsLoginModalOpen(true)}
                openRegisterModal={() => setIsRegisterModalOpen(true)}
            />
            <LoginModal
                isLoginModalOpen={isLoginModalOpen}
                closeLoginModal={() => setIsLoginModalOpen(false)}
            />
            <RegisterModal
                isRegisterModalOpen={isRegisterModalOpen}
                closeRegisterModal={() => setIsRegisterModalOpen(false)}
            />

            <div className='h-full flex items-center justify-center'>
                <Image src='/favicons/favicon.ico' width={405} height={509} alt='Logo' />
            </div>
        </>
    );
}
