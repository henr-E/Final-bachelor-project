'use client';

import { Button } from "flowbite-react";
import { Breadcrumb } from 'flowbite-react';
import { HiHome } from 'react-icons/hi';
import { useContext, useState } from "react";
import { SensorContext, Sensor, Quantity, Unit } from "@/store/sensor";
import { MdModeEditOutline, MdOutlineDeleteOutline } from "react-icons/md";
import CreateSensorModal from '../../../components/modals/CreateSensorModal';

function SensorPage() {
    // TODO: turn overview page into dashboard root

    const [sensorState, sensorDispatch] = useContext(SensorContext);

    const [selectedSensors, setSelectedSensors] = useState<Sensor[]>([]);
    const [isCreateSensorModalOpen, setIsSensorModalOpen] = useState(false);

    const handleEditButtonClick = (sensor: Sensor) => {

    }

    const handleDeleteButtonClick = (sensor: Sensor) => {
        // TODO: backend request
        sensorDispatch({ type: 'delete_sensor', sensorId: sensor.id });
    }

    return (
        <div className="flex flex-col space-y-4 w-full h-full">
            <div className="my-4 flex flex-row">
                <Breadcrumb aria-label="Default breadcrumb example">
                    <Breadcrumb.Item href="/dashboard" icon={HiHome}></Breadcrumb.Item>
                    <Breadcrumb.Item>Sensors</Breadcrumb.Item>
                </Breadcrumb>
                <div className="grow"></div>
            </div>
            <div className="flex flex-col grow space-y-4 h-full">
                <div className="flex flex-row">
                    <div>
                        <Button
                            color="indigo"
                            theme={{ color: { indigo: 'bg-indigo-600 text-white ring-indigo-600' } }}
                            onClick={() => setIsSensorModalOpen(true)}
                        >
                            Create Sensor
                        </Button>
                    </div>
                </div>
                <div className="flex flex-row h-96">
                    <div>
                    </div>
                    <div className="overflow-x-auto shadow-md sm:rounded-lg bg-white p-2">
                        <table className="text-sm text-left rtl:text-right text-gray-500 table-auto">
                            <thead className="border-gray-600 text-xs text-gray-700 uppercase bg-gray-50 dark:bg-gray-700 dark:text-gray-400">
                                <tr>
                                    <th></th>
                                    <th scope="col" className="px-3 py-3">
                                        Name
                                    </th>
                                    <th scope="col" className="px-3">
                                        Description
                                    </th>
                                    <th scope="col" className="px-3">
                                        Quantity
                                    </th>
                                    <th scope="col" className="px-3">
                                        Unit
                                    </th>
                                    <th scope="col" className="px-3">
                                        Most Recent Value
                                    </th>
                                    <th scope="col" className="px-3">
                                        Entries
                                    </th>
                                    <th scope="col" className="px-3">
                                        Location
                                    </th>
                                    <th scope="col" className="px-3">
                                        Actions
                                    </th>
                                </tr>
                            </thead>
                            <tbody className="">
                                {
                                    sensorState.sensors.map(sensor =>
                                        <tr key={sensor.id} className="bg-white my-6">
                                            <td className="px-3 py-3">
                                                <div className="flex items-center">
                                                    <input
                                                        id="checkbox-all-search"
                                                        checked={selectedSensors.includes(sensor)}
                                                        onChange={(e) => {
                                                            if (selectedSensors.includes(sensor)) {
                                                                setSelectedSensors(selectedSensors.filter(s => s !== sensor))
                                                            } else {
                                                                setSelectedSensors(selectedSensors.concat([sensor]))
                                                            }
                                                        }}
                                                        type="checkbox"
                                                        className="w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded focus:ring-blue-500 dark:focus:ring-blue-600 dark:ring-offset-gray-800 dark:focus:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600" />
                                                </div>
                                            </td>
                                            <td scope="row" className="px-3 w-24">
                                                {sensor.name}
                                            </td>
                                            <td className="px-3 w-64">
                                                {sensor.description}
                                            </td>
                                            <td className="px-3 w-24">
                                                {Quantity[sensor.quantity]}
                                            </td>
                                            <td className="px-3 w-24">
                                                {Unit[sensor.unit]}
                                            </td>
                                            <td className="px-3 w-24">
                                                -
                                            </td>
                                            <td className="px-3 w-24">
                                                -
                                            </td>
                                            <td className="px-3 w-24">
                                                <a href="#">
                                                    {sensor.location.lat},{sensor.location.lng}
                                                </a>
                                            </td>
                                            <td className="px-3 w-16">
                                                <div className="flex flex-row space-x-2">
                                                    <button disabled onClick={() => handleEditButtonClick(sensor)}><MdModeEditOutline size={24} /></button>
                                                    <button disabled onClick={() => handleDeleteButtonClick(sensor)}><MdOutlineDeleteOutline size={24} /></button>
                                                </div>
                                            </td>
                                        </tr>
                                    )
                                }
                            </tbody>
                        </table>
                    </div>
                </div>
                <div className="grid grid-cols-3">
                    <div>
                    </div>
                </div>
            </div>
            <CreateSensorModal isModalOpen={isCreateSensorModalOpen} closeModal={() => setIsSensorModalOpen(false)} ></CreateSensorModal>
        </div>
    );
}

export default SensorPage;

