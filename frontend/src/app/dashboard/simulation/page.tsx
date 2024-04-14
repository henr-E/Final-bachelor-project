'use client';
import {Button, Spinner} from 'flowbite-react';
import {MdOutlineDeleteOutline} from "react-icons/md";
import {useContext, useEffect, useState} from "react";
import {useRouter} from "next/navigation";
import CreateSimulationModal from "@/components/modals/CreateSimulationModal";
import {TwinContext} from "@/store/twins";
import {mdiCheck, mdiClose} from "@mdi/js";
import Icon from "@mdi/react";
import {BackendGetSimulations, BackendDeleteSimulation} from "@/api/simulation/crud";
import {Simulation} from "@/proto/simulation/frontend";
import ToastNotification from "@/components/notification/ToastNotification";
import DeleteSimulationModal from "@/components/modals/DeleteSimulationModal";
import DeleteMultipleSimulationsModal from "@/components/modals/DeleteMultipleSimulationsModal";

function SimulationOverviewPage() {
    const router = useRouter();
    const [twinState, dispatchTwin] = useContext(TwinContext);
    const [isCreateSimulationModalOpen, setIsCreateSimulationModalOpen] = useState(false);
    const [isDeleteSimulationModalOpen, setIsDeleteSimulationModalOpen] = useState(false);
    const [isDeleteMultipleSimulationsModalOpen, setIsDeleteMultipleSimulationsModalOpen] = useState(false);
    const [selectedSimulations, setSelectedSimulations] = useState<Simulation[]>([]);
    const [simulationToDelete, setSimulationToDelete] = useState<Simulation>();

    const handleClick = (id: string) => {
        router.push('simulation/' + id);
    }

    const handleConfirmSimulationDelete = async () => {
        if(simulationToDelete?.id){
            let response = await BackendDeleteSimulation(simulationToDelete?.id);
            if (response) {
                // ToastNotification('success', `Simulation is deleted`);
                ToastNotification("error", "Not yet implemented")
                if(twinState.current){
                    let simulations = await BackendGetSimulations(String(twinState.current?.id));
                    dispatchTwin({type: "load_simulations", simulations: simulations.item})
                }
            }
        }
    };

    const handleDeleteSelectedSimulations = async () => {
        //todo
    };

    const handleCancelSelectedSimulationsDelete = async () => {
        //todo
    };


    return (
        <>
            {!twinState.current && (
                <div>Please select a Twin.</div>
            )}
            {twinState.current && (
                <div className='flex flex-col space-y-4 w-full h-full'>
                    <div className='flex flex-col grow space-y-4 h-full w-full'>
                        <div className='flex flex-row'>
                            <div>
                                <Button
                                    onClick={() => {
                                        setIsCreateSimulationModalOpen(true)
                                    }}
                                    color='indigo'
                                    theme={{
                                        color: {
                                            indigo: 'bg-indigo-600 text-white ring-indigo-600',
                                        },
                                    }}
                                >
                                    Create Simulation
                                </Button>
                            </div>
                        </div>
                        <div className='flex flex-row'>
                            <div></div>
                            {twinState.current.simulations?.length == 0 && (
                                <div>There are no simulations for this twin.</div>
                            )}
                            {twinState.current.simulations?.length !== 0 && (
                                <div className='shadow-md sm:rounded-lg bg-white p-2 w-full min-h-96 relative'>
                                    <table className='text-sm text-left rtl:text-right text-gray-500 w-full table-auto'>
                                        <thead
                                            className='border-gray-600 text-xs text-gray-700 uppercase bg-gray-50 dark:bg-gray-700 dark:text-gray-400'>
                                        <tr>
                                            <th scope='col' className='px-3 py-3 w-8'></th>
                                            <th scope='col' className='p-3 px-3 py-3'>
                                                Name
                                            </th>
                                            <th scope='col' className='p-3 px-3'>
                                                Simulation start
                                            </th>
                                            <th scope='col' className='p-3 px-3'>
                                                Simulation end
                                            </th>
                                            <th scope='col' className='p-3 px-3'>
                                                Creation date
                                            </th>
                                            <th scope='col' className='p-3 px-3'>
                                                Loaded frames
                                            </th>
                                            <th scope='col' className='p-3 px-1 text-center w-20'>
                                                Status
                                            </th>
                                            <th scope='col' className='p-3 px-3 text-center w-20'>
                                                Delete
                                            </th>
                                        </tr>
                                        </thead>
                                        <tbody>
                                        {twinState.current.simulations?.map(simulation => (

                                            <tr
                                                key={simulation.id}
                                                className='my-6'
                                                style={{cursor: 'pointer'}}
                                            >

                                                <th scope='row' className='px-3 py-3 w-8'>
                                                    <div className='flex items-center'>
                                                        <input
                                                            id='checkbox-all-search'
                                                            checked={selectedSimulations.includes(simulation)}
                                                            onChange={e => {
                                                                if (selectedSimulations.includes(simulation)) {
                                                                    setSelectedSimulations(
                                                                        selectedSimulations.filter(
                                                                            s => s !== simulation
                                                                        )
                                                                    );
                                                                } else {
                                                                    setSelectedSimulations(
                                                                        selectedSimulations.concat([simulation])
                                                                    );
                                                                }
                                                            }}
                                                            type='checkbox'
                                                            className='w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded focus:ring-blue-500 focus:ring-2 dark:bg-gray-700'
                                                        />
                                                    </div>
                                                </th>
                                                <td scope='row' className='p-3 px-3' onClick={() => handleClick(simulation.id)}>
                                                    {simulation.name}
                                                </td>
                                                <td className='p-3 px-3' onClick={() => handleClick(simulation.id)}>
                                                    {new Date(+simulation.startDateTime * 1000).toLocaleString()}
                                                </td>
                                                <td className='p-3 px-3' onClick={() => handleClick(simulation.id)}>
                                                    {new Date(+simulation.endDateTime * 1000).toLocaleString()}
                                                </td>
                                                <td className='p-3 px-3' onClick={() => handleClick(simulation.id)}>
                                                    {new Date(
                                                        +simulation.creationDateTime * 1000
                                                    ).toLocaleString()}
                                                </td>
                                                <td className='p-3 px-3' onClick={() => handleClick(simulation.id)}>{simulation.framesLoaded}</td>
                                                <td className='p-3 px-3 text-center items-center flex justify-center w-20'>
                                                    {simulation.status == 0 ? (
                                                        <div>
                                                            <Spinner
                                                                aria-label='Medium sized spinner example'
                                                                size='md'
                                                            />
                                                        </div>
                                                    ) : simulation.status == 1 ? (
                                                        <Spinner
                                                            color='success'
                                                            aria-label='Medium sized spinner example'
                                                            size='md'
                                                        />
                                                    ) : simulation.status == 2 ? (
                                                        <Icon
                                                            path={mdiCheck}
                                                            color='green'
                                                            size={1}
                                                            className='content-center'
                                                        />
                                                    ) : (
                                                        <Icon
                                                            path={mdiClose}
                                                            color='red'
                                                            size={1}
                                                            className='content-center'
                                                        />
                                                    )
                                                    }
                                                </td>
                                                <td
                                                    className='p-3 px-3 w-16'
                                                    onClick={() => {
                                                        setSimulationToDelete(simulation)
                                                        setIsDeleteSimulationModalOpen(true);
                                                    }}
                                                >
                                                    <div className='flex flex-row space-x-2 justify-center '>
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
                        </div>
                        <div className='grid grid-cols-3'>
                            <div></div>
                        </div>
                    </div>
                    <CreateSimulationModal
                        isModalOpen={isCreateSimulationModalOpen}
                        closeModal={() => {
                            setIsCreateSimulationModalOpen(false);
                        }}
                    ></CreateSimulationModal>
                    <DeleteSimulationModal
                        isModalOpen={isDeleteSimulationModalOpen}
                        simulation={simulationToDelete}
                        confirm={() => {
                            handleConfirmSimulationDelete()
                            setSimulationToDelete(undefined);
                            setIsDeleteSimulationModalOpen(false);
                        }}
                        cancel={() => {
                            setSimulationToDelete(undefined);
                            setIsDeleteSimulationModalOpen(false);
                        }}
                    />
                    {/*todo delete MultipleSimulationModal can only be used when it is linked to the backend */}
                    <DeleteMultipleSimulationsModal
                        isModalOpen={isDeleteMultipleSimulationsModalOpen}
                        simulations={selectedSimulations}
                        confirm={handleDeleteSelectedSimulations}
                        closeModal={handleCancelSelectedSimulationsDelete}
                    />
                </div>
            )}
        </>
    );
}

export default SimulationOverviewPage;
