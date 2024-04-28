'use client';

import { Alert, Button, Modal } from 'flowbite-react';
import { twinObject } from '@/proto/twins/twin';
import { TwinFromProvider } from '@/store/twins';

interface DeleteMultipleTwinsModalProps {
    isModalOpen: boolean;
    twins: TwinFromProvider[];
    confirm: () => void;
    closeModal: () => void;
}

function DeleteMultipleTwinsModal({
    isModalOpen,
    twins,
    closeModal,
    confirm,
}: DeleteMultipleTwinsModalProps) {
    const handleConfirmButtonClick = () => {
        confirm();
        closeModal();
    };

    return (
        <Modal show={isModalOpen} onClose={closeModal}>
            <Modal.Header>Delete Simulations ({twins.length})</Modal.Header>
            <Modal.Body>
                <div className='mb-2 flex flex-col space-y-2'>
                    <span>You are about to delete the following twins:</span>
                    <div>
                        <ul className='max-w-md space-y-1 text-gray-600 list-disc list-inside'>
                            {twins.map((twin, index) => (
                                <li key={index} className='text-sm'>
                                    {twin.name}
                                </li>
                            ))}
                        </ul>
                    </div>
                </div>
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

export default DeleteMultipleTwinsModal;
