'use client';

import { useContext, useEffect, useState } from 'react';
import DashboardNavbar from '@/components/DashboardNavbar';
import DashboardSidebar from '@/components/DashboardSidebar';
import { UserContext } from '@/store/user';
import { TwinContext } from '@/store/twins';
import CreateTwinModal from '@/components/modals/CreateTwinModal';
import { Breadcrumb } from 'flowbite-react';
import { HiHome } from 'react-icons/hi';
import { usePathname, useSearchParams } from 'next/navigation';
import Icon from '@mdi/react';
import { mdiFullscreen, mdiFullscreenExit } from '@mdi/js';
import dynamic from 'next/dynamic';
// import {PredictionMapProps} from "@/components/maps/PredictionMap";
import { CreateTwinModalProps } from '@/components/modals/CreateTwinModal';

export default function DashboardLayout({
    children,
}: Readonly<{
    children: React.ReactNode;
}>) {
    const [userState, dispatchUser] = useContext(UserContext);
    const [twinState, dispatchTwin] = useContext(TwinContext);
    const [isCreateTwinModalOpen, setIsCreateTwinModalOpen] = useState(false);
    const [fullscreenContent, setFullscreenContent] = useState(false);
    const pathname = usePathname();

    const CreateTwinModalImport = dynamic<CreateTwinModalProps>(
        () => import('@/components/modals/CreateTwinModal'),
        { ssr: false }
    );

    useEffect(() => {
        if (userState.token && twinState.twins.length > 0 && !twinState.current) {
            dispatchTwin({ type: 'switch_twin', twin: twinState.twins[0] });
        }
    }, [userState.token, twinState.twins, twinState, dispatchTwin]);

    if (fullscreenContent) {
        return (
            <div className='h-screen'>
                <a href='#' className='fixed' onClick={() => setFullscreenContent(false)}>
                    <Icon path={mdiFullscreenExit} size={1} className='mr-3' />
                </a>
                {children}
            </div>
        );
    }
    return (
        <div className='flex flex-col h-screen'>
            <DashboardNavbar openCreateTwinModal={() => setIsCreateTwinModalOpen(true)} />
            <CreateTwinModalImport
                isCreateTwinModalOpen={isCreateTwinModalOpen}
                closeCreateTwinModal={() => setIsCreateTwinModalOpen(false)}
            ></CreateTwinModalImport>
            <div className='flex flex-row grow'>
                <DashboardSidebar />
                <div className='px-12 py-3 grow h-full flex flex-col'>
                    <div className='my-4 flex flex-row'>
                        <a href='#' onClick={() => setFullscreenContent(true)}>
                            <Icon path={mdiFullscreen} size={1} className='mr-3' />
                        </a>

                        <Breadcrumb aria-label='Default breadcrumb example'>
                            {pathname
                                .slice(1, pathname.length)
                                .split('/')
                                .map(item =>
                                    item === 'dashboard' ? (
                                        <Breadcrumb.Item
                                            key={item}
                                            href={'/' + item}
                                            icon={HiHome}
                                        ></Breadcrumb.Item>
                                    ) : (
                                        <Breadcrumb.Item
                                            key={item}
                                            href={pathname.split(item)[0] + item}
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
