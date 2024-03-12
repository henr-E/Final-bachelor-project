'use client';

import { Twin, TwinContext } from "@/store/twins";
import { Navbar, Dropdown, DropdownItem, Button } from "flowbite-react";
import { useContext } from "react";

interface DashboardNavbarProps {
    openCreateTwinModal: () => void;
 }

function DashboardNavbar({openCreateTwinModal }: DashboardNavbarProps) {
    const [twinState, dispatch] = useContext(TwinContext);

    const onTwinSelect = (twin: Twin) => {
        localStorage.setItem("selectedTwinID", twin.id);
        dispatch({ type: 'switch_twin', twin: twin });
    }

    const handleCreateTwinButtonClick = () => {
        openCreateTwinModal();
    }

    return <Navbar fluid rounded className="shadow-md">
        <div className="flex">
            <span className="w-full whitespace-nowrap text-2xl font-semibold dark:text-white">
                Digital Twin
            </span>
        </div>
        <Dropdown
            pill color="indigo"
            theme={{ floating: { target: 'enabled:hover:bg-indigo-700 bg-indigo-600 text-white' }}}
            label={twinState.current?.name ?? 'Select Twin'}
            dismissOnClick
        >
            {twinState.twins.map(twin => <DropdownItem key={twin.id} onClick={() => onTwinSelect(twin)}>{twin.name}</DropdownItem>)}
            {<DropdownItem>
                <Button pill color="indigo" onClick={() => handleCreateTwinButtonClick()}>create twin</Button>
            </DropdownItem>}
        </Dropdown>
    </Navbar >
}

export default DashboardNavbar;
