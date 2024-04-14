'use client';

import { TwinFromProvider, TwinContext } from '@/store/twins';
import { Navbar, Dropdown, DropdownItem, Button } from 'flowbite-react';
import { useContext } from 'react';
import {useRouter} from "next/navigation";
import ToastNotification from "@/components/notification/ToastNotification";

interface DashboardNavbarProps {
    openCreateTwinModal: () => void;
}

function DashboardNavbar({ openCreateTwinModal }: DashboardNavbarProps) {
    const [twinState, dispatch] = useContext(TwinContext);
    const router = useRouter();

    const onTwinSelect = (twin: TwinFromProvider) => {
        if(twinState.current?.id != twin.id){
            localStorage.setItem("selectedTwinID", String(twin.id));
            dispatch({ type: 'switch_twin', twin: twin });
        }
        else{
            ToastNotification("info", "This twin is already selected.")
        }
    };

    const handleCreateTwinButtonClick = () => {
        openCreateTwinModal();
    };

    return (
        <Navbar fluid rounded className='shadow-md'>
            <div className='flex'>
                <span className='w-full whitespace-nowrap text-2xl font-semibold dark:text-white'>
                    Digital Twin
                </span>
            </div>
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
                    <DropdownItem
                        onClick={handleCreateTwinButtonClick}
                        key="create-twin"
                        style={{
                            display: "block",
                            width: "100%",
                            padding: "0.5rem 1rem",
                            textAlign: "center",
                            backgroundColor: "transparent",
                            color: "#6366f1",
                            borderRadius: "0.375rem",
                            borderWidth: "1px",
                            borderColor: "#6366f1",
                            cursor: "pointer",
                            outline: "none",
                        }}
                    >
                        Create Twin
                    </DropdownItem>
                }
            </Dropdown>
        </Navbar>
    );
}

export default DashboardNavbar;
