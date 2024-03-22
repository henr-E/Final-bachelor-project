'use client';
import dynamic from 'next/dynamic';
import { Quantity, Unit } from '@/store/sensor';
import { createChannel, createClient } from 'nice-grpc-web';
import CreateSensorModal from '@/components/modals/CreateSensorModal';
import { Breadcrumb, Button, RangeSlider, Spinner } from 'flowbite-react';
import { HiHome } from 'react-icons/hi';
import { MdOpenInNew, MdOutlineDeleteOutline } from 'react-icons/md';
import { useCallback, useContext, useEffect, useState } from 'react';
import { useRouter } from 'next/navigation';
import CreateSimulationModal from '@/components/modals/CreateSimulationModal';
import {
    SimulationInterfaceServiceDefinition,
    TwinId,
    Simulations,
} from '@/proto/simulation/frontend';
import { TwinContext } from '@/store/twins';
import { mdiFullscreen, mdiCheck, mdiClose, mdiAnimationOutline } from '@mdi/js';
import Icon from '@mdi/react';
import { uiBackendServiceUrl } from '@/api/urls';

function SimulationOverviewPage() {
    const router = useRouter();
    const [twinState, dispatch] = useContext(TwinContext);
    const [isCreateSensorModalOpen, setIsSensorModalOpen] = useState(false);
    const [simulations, setSimulations] = useState<Simulations>();

    const loadSimulations = useCallback(() => {
        const channel = createChannel(uiBackendServiceUrl);
        const client = createClient(SimulationInterfaceServiceDefinition, channel);

        client.getAllSimulations({ twinId: twinState.current?.id }).then((r: Simulations) => {
            setSimulations(r);
        });
    }, [twinState]);

    useEffect(() => {
        loadSimulations();
    }, [loadSimulations]);

    return (
        <div className='flex flex-col space-y-4 w-full h-full'>
            <div className='flex flex-col grow space-y-4 h-full w-full'>
                <div className='flex flex-row'>
                    <div>
                        <Button
                            onClick={() => setIsSensorModalOpen(true)}
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
                    <div className='shadow-md sm:rounded-lg bg-white p-2 w-full min-h-96 relative'>
                        <table className='text-sm text-left rtl:text-right text-gray-500 w-full table-auto'>
                            <thead className='border-gray-600 text-xs text-gray-700 uppercase bg-gray-50 dark:bg-gray-700 dark:text-gray-400'>
                                <tr>
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
                                <tr>
                                    <td>
                                        {!simulations && (
                                            <Spinner
                                                aria-label='Medium sized spinner example'
                                                size='md'
                                            />
                                        )}
                                    </td>
                                </tr>
                                {simulations?.item.map(item => (
                                    <tr
                                        key={item.id}
                                        className='hover:bg-gray-50'
                                        onClick={() => {
                                            router.push('simulation/' + item.id);
                                        }}
                                        style={{ cursor: 'pointer' }}
                                    >
                                        <td scope='row' className='p-3 px-3'>
                                            {item.name}
                                        </td>
                                        <td className='p-3 px-3'>
                                            {new Date(+item.startDateTime * 1000).toLocaleString()}
                                        </td>
                                        <td className='p-3 px-3'>
                                            {new Date(+item.endDateTime * 1000).toLocaleString()}
                                        </td>
                                        <td className='p-3 px-3'>
                                            {new Date(
                                                +item.creationDateTime * 1000
                                            ).toLocaleString()}
                                        </td>
                                        <td className='p-3 px-3'>{item.framesLoaded}</td>
                                        <td className='p-3 px-3 text-center items-center flex justify-center w-20'>
                                            {item.status == 0 ? (
                                                <div>
                                                    <Spinner
                                                        aria-label='Medium sized spinner example'
                                                        size='md'
                                                    />
                                                </div>
                                            ) : item.status == 1 ? (
                                                <Spinner
                                                    color='success'
                                                    aria-label='Medium sized spinner example'
                                                    size='md'
                                                />
                                            ) : (
                                                <Icon
                                                    path={mdiCheck}
                                                    color='green'
                                                    size={1}
                                                    className='content-center'
                                                />
                                            )}
                                        </td>
                                        <td className='p-3 px-3 w-16'>
                                            <div className='flex flex-row space-x-2 justify-center '>
                                                <button>
                                                    <MdOutlineDeleteOutline
                                                        size={24}
                                                        onClick={() => {
                                                            alert('remove item');
                                                        }}
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
            <CreateSimulationModal
                isModalOpen={isCreateSensorModalOpen}
                closeModal={() => {
                    setIsSensorModalOpen(false);
                    loadSimulations();
                }}
            ></CreateSimulationModal>
        </div>
    );
}

export default SimulationOverviewPage;
