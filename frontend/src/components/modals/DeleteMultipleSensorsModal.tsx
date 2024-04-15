'use client';


import {Alert, Button, Modal} from 'flowbite-react';
import {Sensor} from "@/proto/sensor/sensor-crud";

interface DeleteMultipleSensorsModalProps {
    isModalOpen: boolean;
    sensors: Sensor[];
    confirm: () => void;
    closeModal: () => void;
}

function DeleteMultipleSensorsModal({isModalOpen, sensors, closeModal, confirm}: DeleteMultipleSensorsModalProps) {
    const handleConfirmButtonClick = () => {
        confirm();
        closeModal();
    };

    return (
        <Modal show={isModalOpen} onClose={closeModal}>
            <Modal.Header>Delete Sensors ({sensors.length})</Modal.Header>
            <Modal.Body>
                <div className='mb-2 flex flex-col space-y-2'>
                    <span>You are about to delete the following sensors:</span>
                    <div>
                        <ul className='max-w-md space-y-1 text-gray-600 list-disc list-inside'>
                            {sensors.map((sensor, index) => (
                                <li key={index} className='text-sm'>
                                    {sensor.name}
                                </li>
                            ))}
                        </ul>
                    </div>
                </div>
                <Alert color='warning' rounded>
                    <span>
                        Warning: all data associated with these sensors will be permanently lost.
                    </span>
                </Alert>
            </Modal.Body>
            <Modal.Footer>
                <Button
                    outline
                    color='indigo'
                    theme={{
                        color: {
                            indigo: 'bg-indigo-600 text-white ring-indigo-600',
                        },
                    }}
                    onClick={closeModal}
                >
                    Cancel
                </Button>
                <div className='grow'></div>
                <Button color='warning' onClick={handleConfirmButtonClick}>
                    Delete
                </Button>
            </Modal.Footer>
        </Modal>
    );
}

export default DeleteMultipleSensorsModal;
