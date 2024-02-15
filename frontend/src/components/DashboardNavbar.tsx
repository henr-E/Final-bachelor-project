'use client';

import { Twin, TwinContext } from "@/store/twins";
import { Navbar, Dropdown, DropdownItem } from "flowbite-react";
import { useContext } from "react";

interface DashboardNavbarProps { }

function DashboardNavbar({ }: DashboardNavbarProps) {
    const [twinState, dispatch] = useContext(TwinContext);

    const onTwinSelect = (twin: Twin) => {
        localStorage.setItem("selectedTwinID", twin.id);
        dispatch({ type: 'switch_twin', twin: twin });
    }

    return <Navbar fluid rounded className="shadow-md">
        <div className="flex">
            <span className="w-full whitespace-nowrap text-2xl font-semibold dark:text-white">
                Digital Twin
            </span>
        </div>
        <Dropdown label={twinState.current?.name ?? 'Select Twin'} dismissOnClick={false}>
            {twinState.twins.map(twin => <DropdownItem key={twin.id} onClick={() => onTwinSelect(twin)}>{twin.name}</DropdownItem>)}
        </Dropdown>
    </Navbar >
}

export default DashboardNavbar;

