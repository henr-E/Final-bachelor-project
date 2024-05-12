'use client';
import React, { useCallback, useEffect, useRef, useState } from 'react';
import MainNavbar from '@/components/MainNavbar';
import LoginModal from '@/components/modals/LoginModal';
import RegisterModal from '@/components/modals/RegisterModal';
import { MdArrowUpward, MdDeleteOutline, MdEdit, MdSettings } from 'react-icons/md';
import ContentSection from '@/app/docs/content';
import { CgProfile } from 'react-icons/cg';
import { GrUpdate } from 'react-icons/gr';
import { TbPresentation } from 'react-icons/tb';
import { FaMagnifyingGlass } from 'react-icons/fa6';
import { HiOutlineSwitchHorizontal } from 'react-icons/hi';

interface SidebarItemProps {
    label: string;
    elementRef: React.RefObject<HTMLDivElement>;
    children: React.ReactNode;
}

const SidebarItem: React.FC<SidebarItemProps> = ({ children, label, elementRef }) => {
    const scrollToSection = useCallback((elementRef: React.RefObject<HTMLDivElement>) => {
        elementRef.current?.scrollIntoView({ behavior: 'smooth', block: 'start' });
    }, []);

    return (
        <div
            className='my-2 cursor-pointer hover:bg-indigo-700 hover:text-white flex items-center p-2 text-gray-900 rounded-lg transition-colors duration-200 ease-in-out'
            onClick={() => scrollToSection(elementRef)}
        >
            <div className='w-5 h-5 mr-3 text-indigo-500 group-hover:text-white'>{children}</div>
            <span>{label}</span>
        </div>
    );
};

interface SectionRefs {
    [key: string]: React.RefObject<HTMLDivElement>;
}

interface LabelMapping {
    [key: string]: string;
}

function DocsPage() {
    const [isLoginModalOpen, setIsLoginModalOpen] = useState(false);
    const [isRegisterModalOpen, setIsRegisterModalOpen] = useState(false);
    const [showScrollTopButton, setShowScrollTopButton] = useState(false);

    const sectionRefs: SectionRefs = {
        createAccount: useRef<HTMLDivElement>(null),

        createTwin: useRef<HTMLDivElement>(null),
        switchTwin: useRef<HTMLDivElement>(null),
        deleteTwin: useRef<HTMLDivElement>(null),

        realtime: useRef<HTMLDivElement>(null),

        createPreset: useRef<HTMLDivElement>(null),
        deleteBuilding: useRef<HTMLDivElement>(null),

        createBuildingSensor: useRef<HTMLDivElement>(null),
        createGlobalSensor: useRef<HTMLDivElement>(null),
        updateSensor: useRef<HTMLDivElement>(null),
        deleteSensor: useRef<HTMLDivElement>(null),

        createSimulation: useRef<HTMLDivElement>(null),
        openSimulation: useRef<HTMLDivElement>(null),
        deleteSimulation: useRef<HTMLDivElement>(null),

        time: useRef<HTMLDivElement>(null),
        weather: useRef<HTMLDivElement>(null),
        supplyAndDemand: useRef<HTMLDivElement>(null),
        loadflow: useRef<HTMLDivElement>(null),
        components: useRef<HTMLDivElement>(null),
    };

    const labelMapping: LabelMapping = {
        createAccount: 'create an account?',

        createTwin: 'create a twin?',
        switchTwin: 'switch to another twin?',
        deleteTwin: 'delete twins?',

        realtime: 'use the realtime tab?',

        createPreset: 'create a preset?',
        deleteBuilding: 'delete a building?',

        createBuildingSensor: 'create building sensors?',
        createGlobalSensor: 'create global sensors?',
        updateSensor: 'update sensors?',
        deleteSensor: 'delete sensors?',

        createSimulation: 'create simulations?',
        openSimulation: 'open simulations?',
        deleteSimulation: 'delete simulations?',

        time: 'time',
        weather: 'weather',
        supplyAndDemand: 'supply and demand',
        loadflow: 'loadflow',
        components: 'components',
    };

    return (
        <div className='h-screen flex flex-col'>
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
            <div className='flex overflow-hidden h-full'>
                <div className='shadow-md z-40 w-80 bg-white overflow-auto'>
                    <div className='px-3 py-4'>
                        <span className='text-gray-500 text-lg'>HOW TO ...</span>
                        <SidebarItem
                            label={labelMapping['createAccount']}
                            elementRef={sectionRefs['createAccount']}
                        >
                            <CgProfile size={20} />
                        </SidebarItem>

                        <hr className='my-2 border-t border-gray-300' />
                        <SidebarItem
                            label={labelMapping['createTwin']}
                            elementRef={sectionRefs['createTwin']}
                        >
                            <MdEdit size={20} />
                        </SidebarItem>
                        <SidebarItem
                            label={labelMapping['switchTwin']}
                            elementRef={sectionRefs['switchTwin']}
                        >
                            <HiOutlineSwitchHorizontal size={20} />
                        </SidebarItem>
                        <SidebarItem
                            label={labelMapping['deleteTwin']}
                            elementRef={sectionRefs['deleteTwin']}
                        >
                            <MdDeleteOutline size={20} />
                        </SidebarItem>
                        <hr className='my-2 border-t border-gray-300' />
                        <SidebarItem
                            label={labelMapping['realtime']}
                            elementRef={sectionRefs['realtime']}
                        >
                            <TbPresentation size={20} />
                        </SidebarItem>
                        <hr className='my-2 border-t border-gray-300' />

                        <SidebarItem
                            label={labelMapping['createPreset']}
                            elementRef={sectionRefs['createPreset']}
                        >
                            <MdEdit size={20} />
                        </SidebarItem>
                        <SidebarItem
                            label={labelMapping['deleteBuilding']}
                            elementRef={sectionRefs['deleteBuilding']}
                        >
                            <MdDeleteOutline size={20} />
                        </SidebarItem>

                        <hr className='my-2 border-t border-gray-300' />
                        <SidebarItem
                            label={labelMapping['createBuildingSensor']}
                            elementRef={sectionRefs['createBuildingSensor']}
                        >
                            <MdEdit size={20} />
                        </SidebarItem>
                        <SidebarItem
                            label={labelMapping['createGlobalSensor']}
                            elementRef={sectionRefs['createGlobalSensor']}
                        >
                            <MdEdit size={20} />
                        </SidebarItem>
                        <SidebarItem
                            label={labelMapping['updateSensor']}
                            elementRef={sectionRefs['updateSensor']}
                        >
                            <GrUpdate size={20} />
                        </SidebarItem>
                        <SidebarItem
                            label={labelMapping['deleteSensor']}
                            elementRef={sectionRefs['deleteSensor']}
                        >
                            <MdDeleteOutline size={20} />
                        </SidebarItem>

                        <hr className='my-2 border-t border-gray-300' />
                        <SidebarItem
                            label={labelMapping['createSimulation']}
                            elementRef={sectionRefs['createSimulation']}
                        >
                            <MdEdit size={20} />
                        </SidebarItem>
                        <SidebarItem
                            label={labelMapping['openSimulation']}
                            elementRef={sectionRefs['openSimulation']}
                        >
                            <FaMagnifyingGlass size={20} />
                        </SidebarItem>
                        <SidebarItem
                            label={labelMapping['deleteSimulation']}
                            elementRef={sectionRefs['deleteSimulation']}
                        >
                            <MdDeleteOutline size={20} />
                        </SidebarItem>
                        <hr className='my-2 border-t border-gray-300' />
                        <span className='text-gray-500 text-lg'>Simulators</span>
                        <SidebarItem label={labelMapping['time']} elementRef={sectionRefs['time']}>
                            <MdSettings size={20} />
                        </SidebarItem>
                        <SidebarItem
                            label={labelMapping['weather']}
                            elementRef={sectionRefs['weather']}
                        >
                            <MdSettings size={20} />
                        </SidebarItem>
                        <SidebarItem
                            label={labelMapping['supplyAndDemand']}
                            elementRef={sectionRefs['supplyAndDemand']}
                        >
                            <MdSettings size={20} />
                        </SidebarItem>
                        <SidebarItem
                            label={labelMapping['loadflow']}
                            elementRef={sectionRefs['loadflow']}
                        >
                            <MdSettings size={20} />
                        </SidebarItem>
                        <SidebarItem
                            label={labelMapping['components']}
                            elementRef={sectionRefs['components']}
                        >
                            <MdSettings size={20} />
                        </SidebarItem>
                    </div>
                </div>
                <div className='p-10 pt-5 overflow-y-scroll w-full'>
                    <ContentSection refs={sectionRefs} />
                </div>
            </div>
        </div>
    );
}

export default DocsPage;
