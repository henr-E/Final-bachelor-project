'use client';

import { City, CityContext } from "@/store/city";
import { Navbar, NavbarBrand, Dropdown, NavbarLink, NavbarToggle, NavbarCollapse, Button, DropdownItem } from "flowbite-react";
import { useContext } from "react";

interface DashboardNavbarProps {
}

function DashboardNavbar({ }: DashboardNavbarProps) {
    const [cityState, dispatch] = useContext(CityContext);

    const onCitySelect = (city: City) => {
        dispatch({ type: 'switch_city', city: city });
    }

    return <Navbar fluid rounded>
        <div className="flex">
            <NavbarBrand><span className="self-center whitespace-nowrap text-xl font-semibold dark:text-white">Digital Twin</span></NavbarBrand>
        </div>
        <Dropdown label={cityState.current?.name ?? 'Select City'} dismissOnClick={false}>
            {cityState.cities.map(city => <DropdownItem onClick={() => onCitySelect(city)}>{city.name}</DropdownItem>)}
        </Dropdown>
    </Navbar >
}

export default DashboardNavbar;

