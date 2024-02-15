'use client';

import React, { useState } from 'react';
import MainNavbar from "@/components/MainNavbar";
import LoginModal from '@/components/modals/LoginModal';

export default function Home() {
    const [isLoginModalOpen, setIsLoginModalOpen] = useState(false);

    return (
        <>
            <MainNavbar openLoginModal={() => setIsLoginModalOpen(true)} />
            <LoginModal isLoginModalOpen={isLoginModalOpen} closeLoginModal={() => setIsLoginModalOpen(false)} />
        </>
    )
}
