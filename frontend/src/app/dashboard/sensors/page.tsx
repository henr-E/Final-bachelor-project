'use client';

import { Button } from 'flowbite-react';
import { useContext, useState } from 'react';
import CreateSensorModal from '@/components/modals/CreateSensorModal';
import ShowSignalsModal from '@/components/modals/ShowSignalsModal';
import DeleteMultipleSensorsModal from '@/components/modals/DeleteMultipleSensorsModal';
import { BackendCreateSensor, BackendDeleteSensor, BackendGetSensors } from '@/api/sensor/crud';
import { TwinContext } from '@/store/twins';
import ToastNotification from '@/components/notification/ToastNotification';
import { Sensor } from '@/proto/sensor/sensor-crud';
import { TourControlContext } from '@/store/tour';

interface renderSensorTableProps {
    sensors: Sensor[];
    handleClick: (sensor: Sensor) => void;
    sensorsToDelete: Sensor[];
    setSensorsToDelete: (sensors: Sensor[]) => void;
    global: boolean;
    customGoToNextTourStep: (timeout: number) => void;
    createdSensorDuringTutorial?: number;
}

function renderSensorTable({
    sensors,
    handleClick,
    sensorsToDelete,
    setSensorsToDelete,
    global,
    customGoToNextTourStep,
    createdSensorDuringTutorial,
}: renderSensorTableProps) {
    return (
        <tbody>
            {sensors.map((sensor, index) => (
                <tr
                    key={sensor.id}
                    className={global ? 'bg-indigo-300 ring-indigo-300 my-6' : 'my-6'}
                    style={{ cursor: 'pointer' }}
                >
                    <th
                        scope='row'
                        className={
                            index == 0 ? 'tour-step-16-sensors px-3 py-3 w-8' : 'px-3 py-3 w-8'
                        }
                    >
                        <div
                            className='flex items-center'
                            onClick={() => {
                                customGoToNextTourStep(1);
                            }}
                        >
                            <input
                                id='checkbox-all-search'
                                checked={sensorsToDelete.includes(sensor)}
                                onChange={e => {
                                    if (sensorsToDelete.includes(sensor)) {
                                        setSensorsToDelete(
                                            sensorsToDelete.filter(s => s !== sensor)
                                        );
                                    } else {
                                        setSensorsToDelete(sensorsToDelete.concat([sensor]));
                                    }
                                }}
                                type='checkbox'
                                className='w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded focus:ring-blue-500 focus:ring-2 dark:bg-gray-700'
                            />
                        </div>
                    </th>
                    <td
                        className='tour-step-7-sensors p-3 px-3'
                        onClick={() => {
                            handleClick(sensor);
                            customGoToNextTourStep(1);
                        }}
                    >
                        {sensor.name}
                    </td>
                    <td className='p-3 px-3' onClick={() => handleClick(sensor)}>
                        {sensor.description}
                    </td>
                    <td className='p-3 px-3' onClick={() => handleClick(sensor)}>
                        {sensor.signals.length}
                    </td>
                    <td className='p-3 px-3' onClick={() => handleClick(sensor)}>
                        <span>{new Date().toLocaleDateString()}</span>
                    </td>
                    <td className='p-3 px-3' onClick={() => handleClick(sensor)}>
                        <a href='#'>
                            {global && 'global'}
                            {!global && (
                                <div>
                                    {sensor.latitude},{sensor.longitude}
                                </div>
                            )}
                        </a>
                    </td>
                    <td className='p-3 px-3' onClick={() => handleClick(sensor)}>
                        {sensor.buildingId || 'Global Sensor'}
                    </td>
                </tr>
            ))}
        </tbody>
    );
}

function SensorPage() {
    const [twinState, dispatchTwin] = useContext(TwinContext);
    const [isCreateSensorModalOpen, setIsCreateSensorModalOpen] = useState(false);
    const [isDeleteMultipleSensorsModalOpen, setIsDeleteMultipleSensorsModalOpen] = useState(false);
    const [sensorsToDelete, setSensorsToDelete] = useState<Sensor[]>([]);
    const [isShowSignalsModalOpen, setIsShowSignalsModalOpen] = useState(false);
    const [selectedSensor, setSelectedSensor] = useState<Sensor>();

    const handleClick = (sensor: Sensor) => {
        setIsShowSignalsModalOpen(true);
        setSelectedSensor(sensor);
    };

    const handleCreateSensor = async (sensor: Sensor) => {
        let success = await BackendCreateSensor(sensor);
        if (!success) {
            ToastNotification('error', 'Failed to create sensor');
            return;
        }

        if (twinState.current) {
            let sensors = await BackendGetSensors(twinState.current?.id);
            dispatchTwin({ type: 'load_sensors', sensors: sensors });
        }

        ToastNotification('success', `Sensor is created`);
    };

    const handleDeleteSelectedSensors = async () => {
        if (!sensorsToDelete) {
            return;
        }
        try {
            await Promise.all(
                sensorsToDelete.map(async sensor => {
                    await BackendDeleteSensor(sensor.id);
                })
            );

            sensorsToDelete.map(sensor => {
                dispatchTwin({ type: 'delete_sensor', sensorId: sensor.id });
            });

            setSensorsToDelete([]);
            ToastNotification('success', `Sensors are deleted`);
        } catch {
            ToastNotification('error', `Something went wrong while deleting sensors.`);
        }
    };

    const handleCancelSelectedSensorsDelete = () => {
        setSensorsToDelete([]);
        setIsDeleteMultipleSensorsModalOpen(false);
    };

    const tourController = useContext(TourControlContext);

    return (
        <>
            {!twinState.current && <div>Please select a Twin.</div>}
            {twinState.current && (
                <div className='container space-y-4'>
                    <div className='space-y-4 flex flex-col w-auto'>
                        <div className='flex flex-row space-x-2'>
                            <Button
                                className={'tour-step-0-sensors'}
                                color='indigo'
                                theme={{
                                    color: {
                                        indigo: 'bg-indigo-600 text-white ring-indigo-600',
                                    },
                                }}
                                onClick={() => {
                                    if (twinState.current) {
                                        let global_sensors = twinState.current.sensors.filter(
                                            sensor => !sensor.buildingId
                                        );
                                        if (global_sensors.length != 0) {
                                            ToastNotification(
                                                'warning',
                                                'There can be only one global sensor.'
                                            );
                                            if (tourController?.isOpen) {
                                                tourController.setIsOpen(false);
                                                ToastNotification(
                                                    'warning',
                                                    'The tutorial has been stopped because a global sensor already exists.'
                                                );
                                            }
                                        } else {
                                            //only go to the next tutorial window if there is no global tutorial
                                            tourController?.customGoToNextTourStep(1);
                                            setIsCreateSensorModalOpen(true);
                                        }
                                    } else {
                                        ToastNotification('error', 'Twin not selected. Try again.');
                                    }
                                }}
                            >
                                Create Sensor
                            </Button>
                            {twinState.current.sensors.length != 0 && (
                                <Button
                                    className={'tour-step-17-sensors'}
                                    color='indigo'
                                    theme={{
                                        color: {
                                            indigo: 'bg-indigo-600 text-white ring-indigo-600',
                                        },
                                    }}
                                    onClick={() => {
                                        if (twinState.current) {
                                            if (sensorsToDelete?.length == 0) {
                                                ToastNotification('info', 'No sensors selected.');
                                            } else {
                                                setIsDeleteMultipleSensorsModalOpen(true);
                                            }
                                        } else {
                                            ToastNotification(
                                                'error',
                                                'Twin not selected. Try again.'
                                            );
                                        }
                                        tourController?.customGoToNextTourStep(1);
                                    }}
                                >
                                    Delete selected sensors
                                </Button>
                            )}
                        </div>

                        {twinState.current && twinState.current.sensors?.length == 0 && (
                            <div>There are no sensors for this twin.</div>
                        )}
                        {twinState.current && twinState.current.sensors?.length !== 0 && (
                            <div className='shadow-md sm:rounded-lg bg-white p-2 w-full min-h-96 relative'>
                                <table className='text-sm text-left rtl:text-right text-gray-500 w-full table-auto'>
                                    <thead className='border-gray-600 text-xs text-gray-700 uppercase bg-gray-50 dark:bg-gray-700 dark:text-gray-400'>
                                        <tr>
                                            <th scope='col' className='px-3 py-3 w-8'></th>
                                            <th scope='col' className='p-3 px-3 py-3'>
                                                Name
                                            </th>
                                            <th scope='col' className='p-3 px-3'>
                                                Description
                                            </th>
                                            <th scope='col' className='p-3 px-3'>
                                                Signal Count
                                            </th>
                                            <th scope='col' className='p-3 px-3'>
                                                Last Updated
                                            </th>
                                            <th scope='col' className='p-3 px-3'>
                                                Location
                                            </th>
                                            <th scope='col' className='p-3 px-3'>
                                                Building Number
                                            </th>
                                        </tr>
                                    </thead>

                                    {/*table for global sensor*/}
                                    {renderSensorTable({
                                        sensors: twinState.current.sensors.filter(
                                            sensor => !sensor.buildingId
                                        ),
                                        handleClick,
                                        sensorsToDelete,
                                        setSensorsToDelete,
                                        global: true,
                                        customGoToNextTourStep:
                                            tourController?.customGoToNextTourStep ||
                                            ((timeout: number): void => {}),
                                    })}

                                    {/*table for building sensors*/}
                                    {renderSensorTable({
                                        sensors: twinState.current.sensors.filter(
                                            sensor => sensor.buildingId
                                        ),
                                        handleClick,
                                        sensorsToDelete,
                                        setSensorsToDelete,
                                        global: false,
                                        customGoToNextTourStep:
                                            tourController?.customGoToNextTourStep ||
                                            ((timeout: number): void => {}),
                                    })}
                                </table>
                            </div>
                        )}
                        <div></div>
                    </div>
                    <CreateSensorModal
                        isModalOpen={isCreateSensorModalOpen}
                        selectedBuildingId={null}
                        handleCreateSensor={handleCreateSensor}
                        closeModal={() => setIsCreateSensorModalOpen(false)}
                    />
                    <ShowSignalsModal
                        isModalOpen={isShowSignalsModalOpen}
                        sensor={selectedSensor}
                        closeModal={() => {
                            setIsShowSignalsModalOpen(false);
                        }}
                    />
                    <DeleteMultipleSensorsModal
                        isModalOpen={isDeleteMultipleSensorsModalOpen}
                        sensors={sensorsToDelete}
                        confirm={handleDeleteSelectedSensors}
                        closeModal={handleCancelSelectedSensorsDelete}
                    />
                </div>
            )}
        </>
    );
}

export default SensorPage;
