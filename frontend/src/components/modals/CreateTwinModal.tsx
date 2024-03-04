'use client';

import {Twin, TwinContext} from '@/store/twins';

import { City, CityContext, CityProvider } from '@/store/city';
import {
    Button,
    Modal,
    Label,
    TextInput,
    Dropdown
} from 'flowbite-react'
import { useContext, useState, useRef } from 'react';
import { useRouter } from 'next/navigation';

interface CreateTwinProps {
    isCreateTwinModalOpen : boolean;
    closeCreateTwinModal: () => void;
}


function CreateTwinModalProps({ isCreateTwinModalOpen, closeCreateTwinModal}: CreateTwinProps) {
    /**
     * This is function is responsible for creating a new twin
     * The user can select from a list of available cities.
     */

    const [TwinState, dispatchTwin] = useContext(TwinContext);
    const [cityState, dispatchCity] = useContext(CityContext);

    const [name, setName] = useState("");
    const [city, setCity] = useState<City>();



    const formRef = useRef<HTMLFormElement>(null);


    const handleSelectButton = (city: City) => {
        /**
         * When a city is being selected it will call dispatchCity to switch the cityState
         * to the new city
         */
        setCity(city)



        const cityTwin = city.name + " Twin" // adding Twin manually, because it's being selected from City, so Antwerp Twin vs Antwerp
        setName(cityTwin)
        dispatchCity({type: 'switch_city', city})
    }

    const handleCreateTwinButton = () => {
        /**
         * This is will first perform a basic check if all field are fill out
         * and will create a new twin based on the selected city
         */
        if (!formRef.current?.checkValidity()) {
            formRef.current?.reportValidity();
            return;
        }

        if(!city){
            return;
        }

        // for now ID is hardcoded as it will be decided by the database
        const twin:Twin = {id: "0", name: name, city:city};


        dispatchTwin({type: 'create_twin', twin})

        closeCreateTwinModal();
    }


    const handleCancelButtonClick = () => {
        closeCreateTwinModal();
    }

    return (
        <Modal show={isCreateTwinModalOpen} onClose={closeCreateTwinModal} style={{zIndex: 100}}>
            <Modal.Header>Create Twin</Modal.Header>
            <Modal.Body>
                <form ref={formRef}>
                <div>
                <Dropdown pill color="indigo" theme={{ floating: { target: 'enabled:hover:bg-indigo-700 bg-indigo-600 text-white' } }} label={cityState.current?.name ?? 'Select city'} dismissOnClick>
                    {cityState.cities.map((city, index) => (
                    <Dropdown.Item key={index} onClick={() => handleSelectButton(city)}>{city.name}</Dropdown.Item>
                    ))}
                </Dropdown>
                </div>
                </form>
            </Modal.Body>
            <Modal.Footer>
                    <Button pill color="indigo" onClick={handleCreateTwinButton}>Create</Button>
                    <Button color="gray" onClick={handleCancelButtonClick}>Cancel</Button>
                </Modal.Footer>
        </Modal>
    )
}

export default CreateTwinModalProps;
