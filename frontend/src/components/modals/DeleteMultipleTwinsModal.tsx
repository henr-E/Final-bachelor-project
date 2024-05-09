'use client';

import { Alert, Button, Modal } from 'flowbite-react';
import { TwinFromProvider } from '@/store/twins';
import { useContext } from 'react';
import { TourControlContext } from '@/store/tour';

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

    const tourController = useContext(TourControlContext);

    return (
        <Modal show={isModalOpen} onClose={closeModal}>
            <Modal.Header>Delete Twins ({twins.length})</Modal.Header>
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
                <Button
                    className={'tour-step-10-overview'}
                    color={'red'}
                    onClick={() => {
                        handleConfirmButtonClick();
                        tourController?.setIsOpen(false);
                    }}
                >
                    Delete
                </Button>
            </Modal.Footer>
        </Modal>
    );
}

export default DeleteMultipleTwinsModal;
