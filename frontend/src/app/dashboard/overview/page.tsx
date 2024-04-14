'use client'
import React, {useContext, useEffect, useReducer, useState} from 'react';
import {TwinContext} from "@/store/twins";
import {MdOutlineDeleteOutline} from "react-icons/md";
import {useRouter} from "next/navigation";
import ToastNotification from "@/components/notification/ToastNotification";
import {Button, Modal} from "flowbite-react";
import {LatLngBoundsExpression} from "leaflet";
import dynamic from "next/dynamic";
import {CustomMapContainerProps} from "@/components/maps/CustomMapContainer";
import {BackendGetTwins} from "@/api/twins/crud";
import {twinObject} from "@/proto/twins/twin";

const CustomMapContainerImport = dynamic<CustomMapContainerProps>(() => import("@/components/maps/CustomMapContainer"), {ssr: false});

function OverviewPage() {
    const [twinState, dispatch] = useContext(TwinContext);
    const router = useRouter();
    const [isConfirmModalOpen, setIsConfirmModalOpen] = useState(false);
    const [twinDeleteIndex, setTwinDeleteIndex] = useState(-1);
    const [position, setPosition] = useState<[number, number]>([51.505, -0.09]);
    const [bounds, setBounds] = useState<LatLngBoundsExpression>([[0, 0], [0, 0]]);
    const [state, dispatchTwin] = useContext(TwinContext);


    useEffect(() => {
        async function getTwins() {
            let response = await BackendGetTwins();
            if (response) {
                let twinsFromBackend = response.twins.map((twinItem: twinObject) => ({
                    id: twinItem.id,
                    name: twinItem.name,
                    longitude: twinItem.longitude,
                    latitude: twinItem.latitude,
                    radius: Number(twinItem.radius),
                    sensors: [],
                    simulations: [],
                    creation_date_time: twinItem.creationDateTime,
                    simulation_amount: twinItem.simulationAmount
                }));

                if (twinsFromBackend.length > 0) {
                    // Load all twins into the state
                    dispatchTwin({type: 'load_twins', twins: twinsFromBackend});
                    ToastNotification("info", "All twins are being loaded.")
                } else {
                    // Optionally handle the case where no twins are returned
                    ToastNotification("info", "No twins found.");
                }
            }
        }
        getTwins();
    }, [dispatchTwin, router]);

    const handleOpenModal = (index: number) => {
        setTwinDeleteIndex(index);
        setIsConfirmModalOpen(true);
        setPosition([twinState.twins[index]?.latitude, twinState.twins[index]?.longitude]);
        const positionTemp = [twinState.twins[index]?.latitude, twinState.twins[index]?.longitude];

        let radius_km = twinState.twins[index]?.radius / 1000.0;
        // Calculate degrees of latitude per kilometer
        let delta_lat = radius_km / 111.0;
        // Calculate degrees of longitude per kilometer at the given latitude
        // Make sure to specify the type for numerical literals and perform operations on the correct type
        let pi = Math.PI;
        let latitude_in_radians = positionTemp[0] * (pi / 180);
        let delta_lon = radius_km / (111.0 * Math.cos(latitude_in_radians));

        // Calculate bounds for the rectangle
        const bounds: LatLngBoundsExpression = [
            [positionTemp[0] - delta_lat, positionTemp[1] - delta_lon], // Southwest corner
            [positionTemp[0] + delta_lat, positionTemp[1] + delta_lon], // Northeast corner
        ];

        setBounds(bounds);
    };

    const resetAll = () => {
        setIsConfirmModalOpen(false);
        setTwinDeleteIndex(-1);
        setPosition([51.505, -0.09]);
        setBounds([[0, 0], [0, 0]]);
    };

    const handleDeleteTwin = async () => {
        try {
            ToastNotification("error", "Deleting twin is not yet implemented");
        } catch (error) {
            console.log(error);
            ToastNotification("error", "There was a problem deleting the twin.");
        }
    };

    return (
        <>
            <div className='flex flex-col space-y-4 w-full h-full'>
                <div className='flex flex-col grow space-y-4 h-full w-full'>
                    <div className='flex flex-row'>
                        <div></div>
                        <div className='shadow-md sm:rounded-lg bg-white p-2 w-full min-h-96 relative'>
                            <table className='text-sm text-left rtl:text-right text-gray-500 w-full table-auto'>
                                <thead
                                    className='border-gray-600 text-xs text-gray-700 uppercase bg-gray-50 dark:bg-gray-700 dark:text-gray-400'>
                                <tr>
                                    <th scope='col' className='p-3 px-3 py-3 text-center'>
                                        id
                                    </th>
                                    <th scope='col' className='p-3 px-3 text-center'>
                                        creation date & time
                                    </th>
                                    <th scope='col' className='p-3 px-3 text-center'>
                                        name
                                    </th>
                                    <th scope='col' className='p-3 px-3 text-center'>
                                        latitude, longitude
                                    </th>
                                    <th scope='col' className='p-3 px-3 text-center'>
                                        radius
                                    </th>
                                    <th scope='col' className='p-3 px-3 text-center'>
                                        amount of active simulations
                                    </th>
                                    <th scope='col' className='p-3 px-3  text-center'>
                                        Delete
                                    </th>
                                </tr>
                                </thead>
                                <tbody>
                                {twinState.twins?.map((overviewTwinItem, index) => (
                                    <tr
                                        key={index}
                                    >
                                        <td
                                            style={{cursor: 'pointer'}}
                                            scope='row'
                                            className='hover:bg-gray-100 p-3 px-3 text-center'
                                            onClick={() => {
                                                dispatch({type: 'switch_twin', twin: overviewTwinItem});
                                                ToastNotification("info", `Switching to twin ${overviewTwinItem.name}!`)
                                                router.push('editor/');
                                            }}
                                        >{overviewTwinItem.id}</td>
                                        <td
                                            style={{cursor: 'not-allowed'}}
                                            className='p-3 px-3 text-center'
                                        >
                                            {new Date(+overviewTwinItem.creation_date_time * 1000).toLocaleString()}
                                        </td>
                                        <td
                                            style={{cursor: 'pointer'}}
                                            className='hover:bg-gray-100 p-3 px-3 text-center'
                                            onClick={() => {
                                                dispatch({type: 'switch_twin', twin: overviewTwinItem});
                                                ToastNotification("info", `Switching to twin ${overviewTwinItem.name}!`)
                                                router.push('editor/');
                                            }}
                                        >{overviewTwinItem.name}</td>
                                        <td
                                            style={{cursor: 'not-allowed'}}
                                            className='p-3 px-3 text-center'
                                        >{overviewTwinItem.latitude}, {overviewTwinItem.longitude}</td>
                                        <td
                                            style={{cursor: 'not-allowed'}}
                                            className='p-3 px-3 text-center'
                                        >{overviewTwinItem.radius}</td>
                                        <td
                                            style={{cursor: 'pointer'}}
                                            className='hover:bg-gray-100 p-3 px-3 text-center'
                                            onClick={() => {
                                                dispatch({type: 'switch_twin', twin: overviewTwinItem});
                                                ToastNotification("info", `Switching to twin ${overviewTwinItem.name}!`)
                                                router.push('simulation/');
                                            }}
                                        >{overviewTwinItem.simulation_amount}</td>
                                        <td
                                            style={{cursor: 'pointer'}}
                                            className='hover:bg-gray-100 p-3 px-3 text-centerw-16'
                                            onClick={() => handleOpenModal(index)}
                                        >
                                            <div className='flex flex-row space-x-2 justify-center'>
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
                    </div>
                    <div className='grid grid-cols-3'>
                        <div></div>
                    </div>
                </div>
            </div>

            <Modal
                show={isConfirmModalOpen}
                style={{
                    transition: 'filter 0.3s ease',
                    zIndex: 2000,
                }}
                onClose={() => {
                    setIsConfirmModalOpen(false);
                }}
            >
                <Modal.Header>Confirm Deletion</Modal.Header>
                <Modal.Body>
                    <b>Are you sure you want to delete this twin?</b>
                    <h1>Custom Name: {twinState.twins[twinDeleteIndex]?.name}</h1>
                    <div>
                        Position: {twinState.twins[twinDeleteIndex]?.latitude} , {twinState.twins[twinDeleteIndex]?.longitude}{' '}
                    </div>
                    <div>Radius: {twinState.twins[twinDeleteIndex]?.radius} meters</div>
                    <div style={{height: '400px', width: '100%', marginTop: '20px'}}>
                        <CustomMapContainerImport position={position} bounds={bounds}></CustomMapContainerImport>
                    </div>
                </Modal.Body>
                <Modal.Footer className="flex flex-row w-100">
                    <Button
                        outline
                        color="indigo"
                        theme={{color: {indigo: 'bg-indigo-600 text-white ring-indigo-600'}}}
                        onClick={resetAll}
                    >
                        Back
                    </Button>
                    <div className="grow"></div>
                    <Button
                        color="indigo"
                        theme={{color: {indigo: 'bg-indigo-600 text-white ring-indigo-600'}}}
                        onClick={handleDeleteTwin}
                    >
                        Delete
                    </Button>
                </Modal.Footer>
            </Modal>
        </>
    );
}

export default OverviewPage;

