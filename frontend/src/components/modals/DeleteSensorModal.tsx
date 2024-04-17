'use client';

import { Alert, Button, Modal } from 'flowbite-react';
import { Sensor } from '@/proto/sensor/sensor-crud';

interface DeleteSensorModalProps {
    isModalOpen: boolean;
    sensor?: Sensor;
    confirm: (sensor: Sensor) => void;
    cancel: () => void;
}

function DeleteSensorModal({ isModalOpen, sensor, cancel, confirm }: DeleteSensorModalProps) {
    if (!sensor) {
        return <></>;
    }

    return (
        <Modal show={isModalOpen} onClose={cancel}>
            <Modal.Header>Delete Sensor &quot;{sensor.name}&quot;</Modal.Header>
            <Modal.Body>
                <Alert color='warning' rounded>
                    <span>
                        Warning: all data associated with this sensor will be permanently lost.
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
                    onClick={cancel}
                >
                    Cancel
                </Button>
                <div className='grow'></div>
                <Button color='warning' onClick={() => confirm(sensor)}>
                    Delete
                </Button>
            </Modal.Footer>
        </Modal>
    );
}

export default DeleteSensorModal;
