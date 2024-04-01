'use client';

import { Button } from 'flowbite-react';
import { Breadcrumb } from 'flowbite-react';
import { HiHome } from 'react-icons/hi';
import { useContext, useState } from 'react';
import { SensorContext, Sensor } from '@/store/sensor';
import { MdModeEditOutline, MdOutlineDeleteOutline } from 'react-icons/md';
import { BsDatabaseFillAdd } from 'react-icons/bs';
import CreateSensorModal from '@/components/modals/CreateSensorModal';
import DeleteSensorModal from '@/components/modals/DeleteSensorModal';
import DeleteMultipleSensorsModal from '@/components/modals/DeleteMultipleSensorsModal';
import { createSensor, deleteSensor } from '@/api/sensor/crud';

function SensorPage() {
    // TODO: turn overview page into dashboard root

    const [{ isLoading, state: sensorState }, sensorDispatch] = useContext(SensorContext);

    const [selectedSensors, setSelectedSensors] = useState<Sensor[]>([]);
    const [isCreateSensorModalOpen, setIsCreateSensorModalOpen] = useState(false);

    const [sensorToDelete, setSensorToDelete] = useState<Sensor>();
    const [sensorToIngest, setSensorToIngest] = useState<Sensor>();

    const [isDeleteMultipleSensorsModalOpen, setIsDeleteMultipleSensorsModalOpen] = useState(false);

    const handleCreateSensor = async (sensor: Sensor) => {
        await createSensor(sensorDispatch, sensor);
    };

    const handleEditButtonClick = (sensor: Sensor) => { };

    const handleConfirmSensorDelete = async (sensor: Sensor) => {
        await deleteSensor(sensorDispatch, sensor.id);
        setSensorToDelete(undefined);
    };

    const handleDeleteSelectedSensors = async () => {
        await Promise.all(
            selectedSensors.map(async sensor => {
                await deleteSensor(sensorDispatch, sensor.id);
            })
        );
        setSelectedSensors([]);
    };

    const handleCancelSelectedSensorsDelete = () => {
        setIsDeleteMultipleSensorsModalOpen(false);
        setSelectedSensors([]);
    };

    const handleCancelSensorDelete = () => {
        setSensorToDelete(undefined);
    };

    return (
        <div className='container space-y-4'>
            <div className='my-4 flex flex-row'>
                <Breadcrumb>
                    <Breadcrumb.Item href='/dashboard' icon={HiHome}></Breadcrumb.Item>
                    <Breadcrumb.Item>Sensors</Breadcrumb.Item>
                </Breadcrumb>
            </div>
            <div className='space-y-4 flex flex-col w-auto'>
                <div className='flex flex-row space-x-2'>
                    <Button
                        color='indigo'
                        theme={{
                            color: {
                                indigo: 'bg-indigo-600 text-white ring-indigo-600',
                            },
                        }}
                        onClick={() => setIsCreateSensorModalOpen(true)}
                    >
                        Create Sensor
                    </Button>
                    <Button
                        color='indigo'
                        theme={{
                            color: {
                                indigo: 'bg-indigo-600 text-white ring-indigo-600',
                            },
                        }}
                        onClick={() => setIsDeleteMultipleSensorsModalOpen(true)}
                        disabled={selectedSensors.length === 0}
                    >
                        Delete {selectedSensors.length > 0 ? `(${selectedSensors.length})` : ''}
                    </Button>
                </div>
                <div className='shadow-md sm:rounded-lg bg-white p-2 w-full min-h-96 relative'>
                    {sensorState.sensors.length === 0 && (
                        <div className='absolute flex items-center justify-center w-full h-full'>
                            <span className='text-sm text-gray-500'>
                                {isLoading ? <>Loading...</> : <>No Sensors</>}
                            </span>
                        </div>
                    )}
                    <table className='text-sm text-left rtl:text-right text-gray-500 w-full table-auto'>
                        <thead className='border-gray-600 text-xs text-gray-700 uppercase'>
                            <tr>
                                <th scope='col' className='px-3 py-3 w-8'></th>
                                <th scope='col' className='px-3 py-3 w-32'>
                                    Name
                                </th>
                                <th scope='col' className='px-3 w-48'>
                                    Description
                                </th>
                                <th scope='col' className='px-3 w-24'>
                                    Signal Count
                                </th>
                                <th scope='col' className='px-3 w-24'>
                                    Last Updated
                                </th>
                                <th scope='col' className='px-3 w-24'>
                                    Entries
                                </th>
                                <th scope='col' className='px-3 w-24'>
                                    Location
                                </th>
                                <th scope='col' className='px-3 w-24'>
                                    Actions
                                </th>
                            </tr>
                        </thead>
                        <tbody>
                            {sensorState.sensors.map(sensor => (
                                <tr key={sensor.id} className='my-6'>
                                    <th scope='row' className='px-3 py-3 w-8'>
                                        <div className='flex items-center'>
                                            <input
                                                id='checkbox-all-search'
                                                checked={selectedSensors.includes(sensor)}
                                                onChange={e => {
                                                    if (selectedSensors.includes(sensor)) {
                                                        setSelectedSensors(
                                                            selectedSensors.filter(
                                                                s => s !== sensor
                                                            )
                                                        );
                                                    } else {
                                                        setSelectedSensors(
                                                            selectedSensors.concat([sensor])
                                                        );
                                                    }
                                                }}
                                                type='checkbox'
                                                className='w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded focus:ring-blue-500 focus:ring-2 dark:bg-gray-700'
                                            />
                                        </div>
                                    </th>
                                    <td className='px-3 max-w-32 truncate'>{sensor.name}</td>
                                    <td className='px-3 max-w-48 truncate'>{sensor.description}</td>
                                    <td className='px-3 w-24'>{sensor.signals.length}</td>
                                    <td className='px-3 w-24'>
                                        <span> {new Date().toLocaleDateString()}</span>
                                    </td>
                                    <td className='px-3 w-24'>0</td>
                                    <td className='px-3 w-24'>
                                        <a href='#'>
                                            {sensor.location.lat},{sensor.location.lng}
                                        </a>
                                    </td>
                                    <td className='px-3 w-24'>
                                        <div className='flex flex-row space-x-3'>
                                            <button
                                                disabled
                                                onClick={() => handleEditButtonClick(sensor)}
                                            >
                                                <MdModeEditOutline size={24} />
                                            </button>
                                            <button onClick={() => setSensorToIngest(sensor)}>
                                                <BsDatabaseFillAdd size={24} />
                                            </button>
                                            <button
                                                className='text-red-300'
                                                onClick={() => setSensorToDelete(sensor)}
                                            >
                                                <MdOutlineDeleteOutline size={24} />
                                            </button>
                                        </div>
                                    </td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                </div>
                <div></div>
            </div>
            <CreateSensorModal
                isModalOpen={isCreateSensorModalOpen}
                handleCreateSensor={handleCreateSensor}
                closeModal={() => setIsCreateSensorModalOpen(false)}
            />
            <DeleteSensorModal
                sensor={sensorToDelete}
                confirm={handleConfirmSensorDelete}
                cancel={() => handleCancelSensorDelete()}
            />
            <DeleteMultipleSensorsModal
                isModalOpen={isDeleteMultipleSensorsModalOpen}
                sensors={selectedSensors}
                confirm={handleDeleteSelectedSensors}
                closeModal={handleCancelSelectedSensorsDelete}
            />
        </div>
    );
}

export default SensorPage;
