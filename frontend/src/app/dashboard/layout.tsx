'use client';

import { useEffect, useState } from 'react';
import DashboardNavbar from '@/components/DashboardNavbar';
import DashboardSidebar from '@/components/DashboardSidebar';
import { CreateTwinModalProps } from '@/components/modals/CreateTwinModal';
import { Breadcrumb } from 'flowbite-react';
import { HiHome } from 'react-icons/hi';
import { usePathname, useRouter } from 'next/navigation';
import Icon from '@mdi/react';
import { mdiFullscreen, mdiFullscreenExit } from '@mdi/js';
import dynamic from 'next/dynamic';
import Image from 'next/image';
import { router } from 'next/client';
import { it } from 'node:test';

export default function DashboardLayout({
    children,
}: Readonly<{
    children: React.ReactNode;
}>) {
    const [isCreateTwinModalOpen, setIsCreateTwinModalOpen] = useState(false);
    const [sidebarVisible, setSidebarVisible] = useState(true);
    const pathname = usePathname();
    const router = useRouter();

    const CreateTwinModalImport = dynamic<CreateTwinModalProps>(
        () => import('@/components/modals/CreateTwinModal'),
        { ssr: false }
    );

    return (
        <div className='flex flex-col h-screen'>
            <DashboardNavbar openCreateTwinModal={() => setIsCreateTwinModalOpen(true)} />
            <CreateTwinModalImport
                isCreateTwinModalOpen={isCreateTwinModalOpen}
                closeCreateTwinModal={() => setIsCreateTwinModalOpen(false)}
            ></CreateTwinModalImport>
            <div className='flex flex-row grow'>
                {sidebarVisible && <DashboardSidebar />}{' '}
                <div className='px-12 py-3 grow h-full flex flex-col'>
                    <div className='my-4 flex flex-row'>
                        <button onClick={() => setSidebarVisible(!sidebarVisible)}>
                            {sidebarVisible ? (
                                <Image
                                    src='/icons/sidebar-close.svg'
                                    width={18}
                                    height={18}
                                    alt='Logo'
                                    style={{ marginRight: '5px' }}
                                />
                            ) : (
                                <Image
                                    src='/icons/sidebar-expand.svg'
                                    width={18}
                                    height={18}
                                    alt='Logo'
                                    style={{ marginRight: '5px' }}
                                />
                            )}
                        </button>

                        <Breadcrumb aria-label='Default breadcrumb example'>
                            {pathname
                                .slice(1, pathname.length)
                                .split('/')
                                .map(item =>
                                    item === 'dashboard' ? (
                                        <Breadcrumb.Item
                                            key={item}
                                            href={'#' + item}
                                            onClick={() => {
                                                router.replace('/' + item);
                                            }}
                                            icon={HiHome}
                                        ></Breadcrumb.Item>
                                    ) : (
                                        <Breadcrumb.Item
                                            key={item}
                                            href={'#'}
                                            onClick={() => {
                                                router.replace(pathname.split(item)[0] + item);
                                            }}
                                        >
                                            {item}
                                        </Breadcrumb.Item>
                                    )
                                )}
                        </Breadcrumb>
                    </div>
                    {children}
                </div>
            </div>
        </div>
    );
}
