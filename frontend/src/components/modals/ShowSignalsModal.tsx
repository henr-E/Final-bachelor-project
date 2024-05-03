'use client';
import React, { useContext, useEffect, useState } from 'react';
import 'leaflet/dist/leaflet.css';
import { Button, Modal } from 'flowbite-react';
import { Sensor, Signal } from '@/proto/sensor/sensor-crud';
import { TwinContext } from '@/store/twins';
import { updateSensor } from '@/api/sensor/crud';
import ToastNotification from '@/components/notification/ToastNotification';
import UpdateSensorModal from '@/components/modals/UpdateSensorModal';
import { toast } from 'react-toastify';
import { bigIntToExponent, prefixes, prefixExponents, prefixMap } from '@/store/sensor';
import { BigDecimal } from '@/proto/sensor/bigdecimal';

interface ShowSignalsModalProps {
    isModalOpen: boolean;
    sensor?: Sensor;
    closeModal: () => void;
}

/**
 *
 * @param isModalOpen
 * @param sensor
 * @param closeModal
 * @constructor
 */
function ShowSignalsModal({ isModalOpen, sensor, closeModal }: ShowSignalsModalProps) {
    const [isUpdateSensorModalOpen, setIsUpdateSensorModalOpen] = useState(false);
    const [twinState, dispatchTwin] = useContext(TwinContext);
    const [showSignalModal, setShowSignalModal] = useState<boolean>();

    const [currentSensor, setCurrentSensor] = useState<Sensor>();

    useEffect(() => {
        setCurrentSensor(sensor);
    }, [sensor]);

    useEffect(() => {
        setShowSignalModal(isModalOpen);
    }, [isModalOpen]);

    const handleUpdateSensor = async (updatedSensor: Sensor) => {
        let result = await updateSensor(updatedSensor);
        if (result) {
            dispatchTwin({ type: 'update_sensor', sensor: updatedSensor });
            ToastNotification('success', 'Sensor updated succesfully.');
        } else {
            ToastNotification('error', 'Something went wrong when updating the sensor.');
        }

        setCurrentSensor(updatedSensor);
    };

    return (
        <>
            <Modal
                show={showSignalModal}
                onClose={() => {
                    closeModal();
                }}
                style={{
                    maxWidth: '100%',
                    maxHeight: '100%',
                    zIndex: 2000,
                }}
            >
                <Modal.Header>Signals for Sensor {currentSensor?.name} </Modal.Header>
                <Modal.Body>
                    <div className='shadow-md sm:rounded-lg bg-white p-2 w-full min-h-96 relative'>
                        <table className='text-sm text-left rtl:text-right text-gray-500 w-full table-auto'>
                            <thead className='border-gray-600 text-xs text-gray-700 uppercase bg-gray-50 dark:bg-gray-700 dark:text-gray-400'>
                                <tr>
                                    <th scope='col' className='p-3 px-3' style={{ width: '20%' }}>
                                        Alias
                                    </th>
                                    <th scope='col' className='p-3 px-3' style={{ width: '20%' }}>
                                        Unit
                                    </th>
                                    <th scope='col' className='p-3 px-3' style={{ width: '20%' }}>
                                        Quantity
                                    </th>
                                    <th scope='col' className='p-3 px-3' style={{ width: '20%' }}>
                                        Unit
                                    </th>
                                    <th scope='col' className='p-3 px-3' style={{ width: '20%' }}>
                                        Prefix
                                    </th>
                                </tr>
                            </thead>
                            <tbody>
                                {currentSensor?.signals.map((signal, index) => (
                                    <tr key={index} className='my-6' style={{ cursor: 'pointer' }}>
                                        <td className='p-3 px-3'>{signal.alias}</td>
                                        <td className='p-3 px-3'>{signal.unit}</td>
                                        <td className='p-3 px-3'>{signal.quantity}</td>
                                        <td className='p-3 px-3'>{signal.ingestionUnit}</td>
                                        <td className='p-3 px-3'>
                                            {prefixMap.get(
                                                bigIntToExponent(
                                                    signal.prefix as BigDecimal
                                                ) as number
                                            )}
                                        </td>
                                    </tr>
                                ))}
                            </tbody>
                        </table>
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
                        onClick={() => {
                            closeModal();
                        }}
                    >
                        Back
                    </Button>
                    <Button
                        outline
                        color='indigo'
                        theme={{
                            color: {
                                indigo: 'bg-indigo-600 text-white ring-indigo-600',
                            },
                        }}
                        onClick={() => {
                            setIsUpdateSensorModalOpen(true);
                            setShowSignalModal(false);
                        }}
                    >
                        Update Sensor
                    </Button>
                </Modal.Footer>
            </Modal>
            {currentSensor && (
                <UpdateSensorModal
                    isModalOpen={isUpdateSensorModalOpen}
                    selectedBuildingId={currentSensor.buildingId || null}
                    handleUpdateSensor={handleUpdateSensor}
                    closeModal={() => {
                        setIsUpdateSensorModalOpen(false);
                        setShowSignalModal(true);
                    }}
                    sensor={currentSensor}
                />
            )}
        </>
    );
}

export default ShowSignalsModal;
