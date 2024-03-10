'use client';

import { Sensor } from '@/store/sensor';
import {
    Button,
    Modal,
    Alert
} from 'flowbite-react';

interface DeleteSensorModalProps {
    sensor?: Sensor;
    confirm: (sensor: Sensor) => void;
    cancel: () => void;
}

function DeleteSensorModal({ sensor, cancel, confirm }: DeleteSensorModalProps) {
    if (!sensor) {
        return <></>
    }

    return <Modal show={sensor !== undefined} onClose={cancel}>
        <Modal.Header>Delete Sensor "{sensor.name}"</Modal.Header>
        <Modal.Body>
            <Alert color="warning" rounded><span>Warning: all data associated with this sensor will be permanently lost.</span></Alert>
        </Modal.Body>
        <Modal.Footer>
            <Button outline color="indigo" theme={{ color: { indigo: 'bg-indigo-600 text-white ring-indigo-600' } }} onClick={cancel}>Cancel</Button>
            <div className="grow"></div>
            <Button color="warning" onClick={() => confirm(sensor)}>Delete</Button>
        </Modal.Footer>
    </Modal>
}

export default DeleteSensorModal;

