'use client';

import {Button} from 'flowbite-react';
import {useContext, useEffect, useState} from 'react';
import {MdModeEditOutline, MdOutlineDeleteOutline} from 'react-icons/md';
import {BsDatabaseFillAdd} from 'react-icons/bs';
import CreateSensorModal from '@/components/modals/CreateSensorModal';
import DeleteSensorModal from '@/components/modals/DeleteSensorModal';
import DeleteMultipleSensorsModal from "@/components/modals/DeleteMultipleSensorsModal";
import {BackendCreateSensor, BackendDeleteSensor, BackendGetSensors} from '@/api/sensor/crud';
import {TwinContext} from "@/store/twins";
import {Sensor} from "@/proto/sensor/sensor-crud";
import ToastNotification from "@/components/notification/ToastNotification";
import {BackendGetSimulations} from "@/api/simulation/crud";

function SensorPage() {
    const [twinState, dispatchTwin] = useContext(TwinContext);
    const [selectedSensors, setSelectedSensors] = useState<Sensor[]>([]);
    const [isCreateSensorModalOpen, setIsCreateSensorModalOpen] = useState(false);
    const [isDeleteSensorModalOpen, setIsDeleteSensorModalOpen] = useState(false);
    const [isDeleteMultipleSensorsModalOpen, setIsDeleteMultipleSensorsModalOpen] = useState(false);
    const [sensorToDelete, setSensorToDelete] = useState<Sensor>();

    const handleClick = (id: string) => {
        alert('opening the sensor.');
    }

    const handleCreateSensor = async (sensor: Sensor) => {
        let response = await BackendCreateSensor(sensor);
        if (response) {
            ToastNotification('success', `Sensor is created`);
            if(twinState.current){
                let sensors = await BackendGetSensors(twinState.current?.id);
                dispatchTwin({type: "load_sensors", sensors: sensors})
            }
        }
    };

    const handleConfirmSensorDelete = async () => {
        if(sensorToDelete?.id) {
            let response = await BackendDeleteSensor(sensorToDelete?.id);
            if (response) {
                ToastNotification('success', `Sensor is deleted`);
                if (twinState.current) {
                    let sensors = await BackendGetSensors(twinState.current?.id);
                    dispatchTwin({type: "load_sensors", sensors: sensors})
                }
            }
        }
    };


    const handleDeleteSelectedSensors = async () => {
        //todo
    };

    const handleCancelSelectedSensorsDelete = () => {
        //todo
    };

    return (
        <>
            {!twinState.current && (
                <div>Please select a Twin.</div>
            )}
            {twinState.current && (
                <div className='container space-y-4'>
                    <div className='space-y-4 flex flex-col w-auto'>
                        <div className='flex flex-row space-x-2'>
                            <Button
                                color='indigo'
                                theme={{
                                    color: {
                                        indigo: 'bg-indigo-600 text-white ring-indigo-600',
                                    },
                                }}
                                onClick={() => {
                                    if (twinState.current) {
                                        setIsCreateSensorModalOpen(true)
                                    } else {
                                        ToastNotification('error', 'Twin not selected. Try again.')
                                    }
                                }}
                            >
                                Create Sensor
                            </Button>
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
                                            Entries
                                        </th>
                                        <th scope='col' className='p-3 px-3'>
                                            Location
                                        </th>
                                        <th scope='col' className='p-3 px-3 text-center w-20'>
                                            Delete
                                        </th>
                                    </tr>
                                    </thead>
                                    <tbody>
                                    {twinState.current?.sensors.map(sensor => (
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
                                            <td className='p-3 px-3' onClick={() => handleClick(sensor.id)}>{sensor.name}</td>
                                            <td className='p-3 px-3' onClick={() => handleClick(sensor.id)}>{sensor.description}</td>
                                            <td className='p-3 px-3' onClick={() => handleClick(sensor.id)}>{sensor.signals.length}</td>
                                            <td className='p-3 px-3' onClick={() => handleClick(sensor.id)}>
                                                <span> {new Date().toLocaleDateString()}</span>
                                            </td>
                                            <td className='p-3 px-3' onClick={() => handleClick(sensor.id)}>0</td>
                                            <td className='p-3 px-3' onClick={() => handleClick(sensor.id)}>
                                                <a href='#'>
                                                    {sensor.latitude},{sensor.longitude}
                                                </a>
                                            </td>
                                            <td className='p-3 px-3'
                                                onClick={() => {
                                                    setSensorToDelete(sensor)
                                                    setIsDeleteSensorModalOpen(true);
                                                }}
                                            >
                                                <div className='flex flex-row space-x-3'>
                                                    <button>
                                                        <MdOutlineDeleteOutline
                                                            size={24}

                                                        />
                                                    </button>
                                                </div>
                                            </td>
                                        </tr>
                                    ))}
                                    </tbody>
                                </table>
                            </div>
                        )}
                        <div></div>
                    </div>
                    <CreateSensorModal
                        isModalOpen={isCreateSensorModalOpen}
                        handleCreateSensor={handleCreateSensor}
                        closeModal={() => setIsCreateSensorModalOpen(false)}
                    />
                    <DeleteSensorModal
                        isModalOpen={isDeleteSensorModalOpen}
                        sensor={sensorToDelete}
                        confirm={() => {
                            handleConfirmSensorDelete()
                            setSensorToDelete(undefined);
                            setIsDeleteSensorModalOpen(false);
                        }}
                        cancel={() => {
                            setSensorToDelete(undefined);
                            setIsDeleteSensorModalOpen(false);
                        }}
                    />
                    {/*todo delete MultipleSensorModal can only be used when it is linked to the backend */}
                    <DeleteMultipleSensorsModal
                        isModalOpen={isDeleteMultipleSensorsModalOpen}
                        sensors={selectedSensors}
                        confirm={handleDeleteSelectedSensors}
                        closeModal={handleCancelSelectedSensorsDelete}
                    />
                </div>
            )}
        </>
    );

}

export default SensorPage;
