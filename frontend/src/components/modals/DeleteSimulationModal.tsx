'use client';

import {Alert, Button, Modal} from 'flowbite-react';
import {Simulation} from "@/proto/simulation/frontend";


interface DeleteSimulationModalProps {
    isModalOpen: boolean;
    simulation?: Simulation;
    confirm: (simulation: Simulation) => void;
    cancel: () => void;
}

function DeleteSimulationModal({isModalOpen, simulation, cancel, confirm}: DeleteSimulationModalProps) {
    if (!simulation) {
        return <></>;
    }

    return (
        <Modal show={isModalOpen} onClose={cancel}>
            <Modal.Header>Delete Simulation &quot;{simulation.name}&quot;</Modal.Header>
            <Modal.Body>
                <Alert color='warning' rounded>
                    <span>
                        Warning: all data associated with this simulation will be permanently lost.
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
                <Button color='warning' onClick={() => confirm(simulation)}>
                    Delete
                </Button>
            </Modal.Footer>
        </Modal>
    );
}

export default DeleteSimulationModal;
