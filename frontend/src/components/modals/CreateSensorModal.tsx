'use client';

import { useContext, useState, useRef } from 'react';
import { Sensor, SensorContext, Quantity, quantityBaseUnits } from '@/store/sensor';
import { v4 as uuidv4 } from 'uuid';
import {
    Button,
    Modal,
    Label,
    TextInput,
    Select
} from 'flowbite-react';

interface CreateSensorModalProps {
    isModalOpen: boolean;
    closeModal: () => void;
}

function CreateSensorModal({ isModalOpen, closeModal }: CreateSensorModalProps) {
    const [sensorState, dispatchSensor] = useContext(SensorContext);

    const [name, setName] = useState<string>("");
    const [description, setDescription] = useState<string>("");
    const [quantity, setQuantity] = useState<Quantity>(Quantity.TEMPERATURE);

    const formRef = useRef<HTMLFormElement>(null);

    const handleSensorCreateButtonClick = () => {
        if (!formRef.current?.checkValidity()) {
            formRef.current?.reportValidity();
            return;
        }

        const sensor: Sensor = {
            id: uuidv4(),
            name: name,
            description: description,
            quantity: quantity,
            unit: quantityBaseUnits[quantity],
            location: { lat: 52, lng: 4 },
        }

        // TODO: backend request

        dispatchSensor({ type: 'create_sensor', sensor });
        closeModal();
    }

    const handleCancelButtonClick = () => {
        setName("");
        setDescription("");
        setQuantity(Quantity.TEMPERATURE);

        closeModal();
    }

    return (
        <>
            <Modal show={isModalOpen} onClose={closeModal}>
                <Modal.Header>Create Sensor</Modal.Header>
                <Modal.Body>
                    <form ref={formRef}>
                        <div>
                            <div className="mb-2 block">
                                <Label htmlFor="name" value="Name" />
                            </div>
                            <TextInput id="name" type="text" value={name} placeholder="name" required maxLength={50} onChange={(e) => setName(e.target.value)} style={{ marginBottom: '10px' }} />
                        </div>
                        <div>
                            <div className="mb-2 block">
                                <Label htmlFor="description" value="Description" />
                            </div>
                            <TextInput id="description" type="text" value={description} placeholder={"description"} maxLength={200} required onChange={(e) => setDescription(e.target.value)} />
                        </div>
                        <div>
                            <div className="mb-2 block">
                                <Label htmlFor="quantity" value="Quantity" />
                            </div>
                            <Select id="quantity" value={quantity} onChange={e => setQuantity(parseInt(e.target.value))} required>
                                {
                                    Object.keys(Quantity).filter(q => isNaN(parseInt(q))).map((q, i) => <option key={i} value={i}>{q}</option>)
                                }
                            </Select>
                        </div>
                    </form>
                </Modal.Body>
                <Modal.Footer>
                    <Button color="indigo" theme={{ color: { indigo: 'bg-indigo-600 text-white ring-indigo-600' } }} onClick={() => handleSensorCreateButtonClick()}>Create</Button>
                    <Button outline color="indigo" theme={{ color: { indigo: 'bg-indigo-600 text-white ring-indigo-600' } }} onClick={handleCancelButtonClick}>Cancel</Button>
                </Modal.Footer>
            </Modal>
        </>
    );
}

export default CreateSensorModal;

