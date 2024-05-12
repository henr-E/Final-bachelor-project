'use client';
import React, { useContext, useEffect, useState } from 'react';
import { TwinContext, TwinFromProvider } from '@/store/twins';
import { useRouter } from 'next/navigation';
import ToastNotification from '@/components/notification/ToastNotification';
import { Button } from 'flowbite-react';
import { LatLngBoundsExpression } from 'leaflet';
import dynamic from 'next/dynamic';
import { CustomMapContainerProps } from '@/components/maps/CustomMapContainer';
import { BackendDeleteTwin, BackendGetTwins } from '@/api/twins/crud';
import { twinObject } from '@/proto/twins/twin';
import DeleteMultipleTwinsModal from '@/components/modals/DeleteMultipleTwinsModal';
import { TourControlContext } from '@/store/tour';

const CustomMapContainerImport = dynamic<CustomMapContainerProps>(
    () => import('@/components/maps/CustomMapContainer'),
    { ssr: false }
);

function OverviewPage() {
    const tourController = useContext(TourControlContext);
    const [twinState, dispatchTwin] = useContext(TwinContext);
    const router = useRouter();
    const [isDeleteMultipleTwinsModalOpen, setIsDeleteMultipleTwinsModalOpen] = useState(false);
    const [twinsToDelete, setTwinsToDelete] = useState<TwinFromProvider[]>([]);

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
                    simulation_amount: twinItem.simulationAmount,
                }));

                if (twinsFromBackend.length > 0) {
                    // Load all twins into the state
                    dispatchTwin({ type: 'load_twins', twins: twinsFromBackend });
                    ToastNotification('info', 'All twins are being loaded.');
                } else {
                    // Optionally handle the case where no twins are returned
                    ToastNotification('info', 'No twins found.');
                }
            }
        }

        getTwins();
    }, [dispatchTwin, router]);

    const handleDeleteSelectedTwins = async () => {
        if (!twinsToDelete) {
            return;
        }
        try {
            await Promise.all(
                twinsToDelete.map(async twin => {
                    await BackendDeleteTwin(twin.id);
                })
            );

            twinsToDelete.map(twin => {
                dispatchTwin({ type: 'delete_twin', twinId: twin.id });
            });

            setTwinsToDelete([]);
            ToastNotification('success', `Twins are deleted`);
        } catch {
            ToastNotification('error', `Something went wrong while deleting twins.`);
        }
        tourController?.setIsOpen(false);
    };

    const handleCancelSelectedTwinsDelete = () => {
        setTwinsToDelete([]);
        setIsDeleteMultipleTwinsModalOpen(false);
    };

    return (
        <>
            {!twinState.current && <div>Please select a Twin.</div>}
            {twinState.current && (
                <div className='flex flex-col space-y-4 w-full h-full'>
                    <div className='flex flex-col grow space-y-4 h-full w-full'>
                        <div className='flex flex-row'>
                            <Button
                                className={'tour-step-9-overview'}
                                color='indigo'
                                theme={{
                                    color: {
                                        indigo: 'bg-indigo-600 text-white ring-indigo-600',
                                    },
                                }}
                                onClick={() => {
                                    if (twinState.current) {
                                        if (twinsToDelete?.length == 0) {
                                            ToastNotification('info', 'No twins selected.');
                                        } else {
                                            setIsDeleteMultipleTwinsModalOpen(true);
                                        }
                                    } else {
                                        ToastNotification('error', 'Twin not selected. Try again.');
                                    }
                                    tourController?.customGoToNextTourStep(1);
                                }}
                            >
                                Delete selected twins
                            </Button>
                        </div>
                        <div className='flex flex-row'>
                            <div className='shadow-md sm:rounded-lg bg-white p-2 w-full min-h-96 relative'>
                                <table className='text-sm text-left rtl:text-right text-gray-500 w-full table-auto'>
                                    <thead className='border-gray-600 text-xs text-gray-700 uppercase bg-gray-50 dark:bg-gray-700 dark:text-gray-400'>
                                        <tr>
                                            <th scope='col' className='px-3 py-3 w-8'></th>
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
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {twinState.twins?.map((overviewTwinItem, index) => (
                                            <tr
                                                key={index}
                                                className='my-6'
                                                style={{ cursor: 'pointer' }}
                                            >
                                                <th
                                                    scope='row'
                                                    className={
                                                        index == twinState.twins?.length - 1
                                                            ? 'tour-step-8-overview px-3 py-3 w-8'
                                                            : 'px-3 py-3 w-8'
                                                    }
                                                >
                                                    <div
                                                        onClick={() => {
                                                            tourController?.customGoToNextTourStep(
                                                                1
                                                            );
                                                        }}
                                                        className='flex items-center'
                                                    >
                                                        <input
                                                            id='checkbox-all-search'
                                                            checked={twinsToDelete.includes(
                                                                overviewTwinItem
                                                            )}
                                                            onChange={e => {
                                                                if (
                                                                    twinsToDelete.includes(
                                                                        overviewTwinItem
                                                                    )
                                                                ) {
                                                                    setTwinsToDelete(
                                                                        twinsToDelete.filter(
                                                                            s =>
                                                                                s !==
                                                                                overviewTwinItem
                                                                        )
                                                                    );
                                                                } else {
                                                                    setTwinsToDelete(
                                                                        twinsToDelete.concat([
                                                                            overviewTwinItem,
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
                                                    style={{ cursor: 'pointer' }}
                                                    scope='row'
                                                    className='hover:bg-gray-100 p-3 px-3 text-center'
                                                    onClick={() => {
                                                        dispatchTwin({
                                                            type: 'switch_twin',
                                                            twin: overviewTwinItem,
                                                        });
                                                        ToastNotification(
                                                            'info',
                                                            `Switching to twin ${overviewTwinItem.name}!`
                                                        );
                                                        router.push('editor/');
                                                    }}
                                                >
                                                    {overviewTwinItem.id}
                                                </td>
                                                <td
                                                    style={{ cursor: 'not-allowed' }}
                                                    className='p-3 px-3 text-center'
                                                >
                                                    {new Date(
                                                        +overviewTwinItem.creation_date_time * 1000
                                                    ).toLocaleString()}
                                                </td>
                                                <td
                                                    style={{ cursor: 'pointer' }}
                                                    className='hover:bg-gray-100 p-3 px-3 text-center'
                                                    onClick={() => {
                                                        dispatchTwin({
                                                            type: 'switch_twin',
                                                            twin: overviewTwinItem,
                                                        });
                                                        ToastNotification(
                                                            'info',
                                                            `Switching to twin ${overviewTwinItem.name}!`
                                                        );
                                                        router.push('editor/');
                                                    }}
                                                >
                                                    {overviewTwinItem.name}
                                                </td>
                                                <td
                                                    style={{ cursor: 'not-allowed' }}
                                                    className='p-3 px-3 text-center'
                                                >
                                                    {overviewTwinItem.latitude},{' '}
                                                    {overviewTwinItem.longitude}
                                                </td>
                                                <td
                                                    style={{ cursor: 'not-allowed' }}
                                                    className='p-3 px-3 text-center'
                                                >
                                                    {overviewTwinItem.radius}
                                                </td>
                                                <td
                                                    style={{ cursor: 'pointer' }}
                                                    className='hover:bg-gray-100 p-3 px-3 text-center'
                                                    onClick={() => {
                                                        dispatchTwin({
                                                            type: 'switch_twin',
                                                            twin: overviewTwinItem,
                                                        });
                                                        ToastNotification(
                                                            'info',
                                                            `Switching to twin ${overviewTwinItem.name}!`
                                                        );
                                                        router.push('simulation/');
                                                    }}
                                                >
                                                    {overviewTwinItem.simulation_amount}
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
                    <DeleteMultipleTwinsModal
                        isModalOpen={isDeleteMultipleTwinsModalOpen}
                        twins={twinsToDelete}
                        confirm={handleDeleteSelectedTwins}
                        closeModal={handleCancelSelectedTwinsDelete}
                    />
                </div>
            )}
        </>
    );
}

export default OverviewPage;
