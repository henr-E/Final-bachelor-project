'use client';
import { Button, Spinner, Tooltip } from 'flowbite-react';
import { MdOutlineDeleteOutline } from 'react-icons/md';
import { useContext, useState } from 'react';
import { useRouter } from 'next/navigation';
import CreateSimulationModal from '@/components/modals/CreateSimulationModal';
import { mdiCheck, mdiClose } from '@mdi/js';
import Icon from '@mdi/react';
import { BackendDeleteSimulation, BackendGetSimulations } from '@/api/simulation/crud';
import { Simulation } from '@/proto/simulation/frontend';
import ToastNotification from '@/components/notification/ToastNotification';
import DeleteMultipleSimulationsModal from '@/components/modals/DeleteMultipleSimulationsModal';
import { TwinContext } from '@/store/twins';

function SimulationOverviewPage() {
    const [twinState, dispatchTwin] = useContext(TwinContext);
    const router = useRouter();
    const [isCreateSimulationModalOpen, setIsCreateSimulationModalOpen] = useState(false);
    const [isDeleteMultipleSimulationsModalOpen, setIsDeleteMultipleSimulationsModalOpen] =
        useState(false);
    const [simulationsToDelete, setSimulationsToDelete] = useState<Simulation[]>([]);

    const handleClick = (id: number) => {
        router.push('simulation/' + id);
    };

    const handleDeleteSelectedSimulations = async () => {
        if (!simulationsToDelete) {
            return;
        }
        try {
            await Promise.all(
                simulationsToDelete.map(async simulation => {
                    await BackendDeleteSimulation(simulation.id);
                })
            );

            simulationsToDelete.map(simulation => {
                dispatchTwin({ type: 'delete_simulation', simulationName: simulation.name });
            });

            setSimulationsToDelete([]);
            ToastNotification('success', `Simulations are deleted`);
        } catch {
            ToastNotification('error', `Something went wrong while deleting simulations.`);
        }
    };

    const handleCancelSelectedSimulationsDelete = async () => {
        setSimulationsToDelete([]);
        setIsDeleteMultipleSimulationsModalOpen(false);
    };

    return (
        <>
            {!twinState.current && <div>Please select a Twin.</div>}
            {twinState.current && (
                <div className='container space-y-4'>
                    <div className='space-y-4 flex flex-col w-auto'>
                        <div className='flex flex-row space-x-2'>
                            <Button
                                onClick={() => {
                                    setIsCreateSimulationModalOpen(true);
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
                            {twinState.current.simulations.length != 0 && (
                                <Button
                                    color='indigo'
                                    theme={{
                                        color: {
                                            indigo: 'bg-indigo-600 text-white ring-indigo-600',
                                        },
                                    }}
                                    onClick={() => {
                                        if (twinState.current) {
                                            if (simulationsToDelete.length == 0) {
                                                ToastNotification(
                                                    'info',
                                                    'No simulations selected.'
                                                );
                                            } else {
                                                setIsDeleteMultipleSimulationsModalOpen(true);
                                            }
                                        } else {
                                            ToastNotification(
                                                'error',
                                                'Twin not selected. Try again.'
                                            );
                                        }
                                    }}
                                >
                                    Delete selected simulations
                                </Button>
                            )}
                        </div>

                        {twinState.current.simulations?.length == 0 && (
                            <div>There are no simulations for this twin.</div>
                        )}
                        {twinState.current.simulations?.length !== 0 && (
                            <div className='shadow-md sm:rounded-lg bg-white p-2 w-full min-h-96 relative'>
                                <table className='text-sm text-left rtl:text-right text-gray-500 w-full table-auto'>
                                    <thead className='border-gray-600 text-xs text-gray-700 uppercase bg-gray-50 dark:bg-gray-700 dark:text-gray-400'>
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
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {twinState.current.simulations?.map(simulation => (
                                            <tr
                                                key={simulation.id}
                                                className='my-6'
                                                style={{ cursor: 'pointer' }}
                                            >
                                                <th scope='row' className='px-3 py-3 w-8'>
                                                    <div className='flex items-center'>
                                                        <input
                                                            id='checkbox-all-search'
                                                            checked={simulationsToDelete.includes(
                                                                simulation
                                                            )}
                                                            onChange={e => {
                                                                if (
                                                                    simulationsToDelete.includes(
                                                                        simulation
                                                                    )
                                                                ) {
                                                                    setSimulationsToDelete(
                                                                        simulationsToDelete.filter(
                                                                            s => s !== simulation
                                                                        )
                                                                    );
                                                                } else {
                                                                    setSimulationsToDelete(
                                                                        simulationsToDelete.concat([
                                                                            simulation,
                                                                        ])
                                                                    );
                                                                }
                                                            }}
                                                            type='checkbox'
                                                            className='w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded focus:ring-blue-500 focus:ring-2 dark:bg-gray-700'
                                                        />
                                                    </div>
                                                </th>
                                                <td
                                                    scope='row'
                                                    className='p-3 px-3'
                                                    onClick={() => handleClick(simulation.id)}
                                                >
                                                    {simulation.name}
                                                </td>
                                                <td
                                                    className='p-3 px-3'
                                                    onClick={() => handleClick(simulation.id)}
                                                >
                                                    {new Date(
                                                        +simulation.startDateTime * 1000
                                                    ).toLocaleString()}
                                                </td>
                                                <td
                                                    className='p-3 px-3'
                                                    onClick={() => handleClick(simulation.id)}
                                                >
                                                    {new Date(
                                                        +simulation.endDateTime * 1000
                                                    ).toLocaleString()}
                                                </td>
                                                <td
                                                    className='p-3 px-3'
                                                    onClick={() => handleClick(simulation.id)}
                                                >
                                                    {new Date(
                                                        +simulation.creationDateTime * 1000
                                                    ).toLocaleString()}
                                                </td>
                                                <td
                                                    className='p-3 px-3'
                                                    onClick={() => handleClick(simulation.id)}
                                                >
                                                    {simulation.framesLoaded}
                                                </td>
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
                                                        <Tooltip content={simulation.statusInfo}>
                                                            <Icon
                                                                path={mdiClose}
                                                                color='red'
                                                                size={1}
                                                                className='content-center'
                                                            />
                                                        </Tooltip>
                                                    )}
                                                </td>
                                            </tr>
                                        ))}
                                    </tbody>
                                </table>
                            </div>
                        )}
                    </div>
                    <div></div>
                    <CreateSimulationModal
                        isModalOpen={isCreateSimulationModalOpen}
                        closeModal={() => {
                            setIsCreateSimulationModalOpen(false);
                        }}
                    ></CreateSimulationModal>
                    <DeleteMultipleSimulationsModal
                        isModalOpen={isDeleteMultipleSimulationsModalOpen}
                        simulations={simulationsToDelete}
                        confirm={handleDeleteSelectedSimulations}
                        closeModal={handleCancelSelectedSimulationsDelete}
                    />
                </div>
            )}
        </>
    );
}

export default SimulationOverviewPage;
