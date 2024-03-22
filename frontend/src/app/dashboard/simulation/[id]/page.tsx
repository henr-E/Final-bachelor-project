'use client';
import dynamic from 'next/dynamic';
import { RangeSlider, Card, Button } from 'flowbite-react';
import { useContext, useEffect, useRef, useState } from 'react';
import { useParams } from 'next/navigation';
import { TwinContext } from '@/store/twins';
import { PredictionMapProps } from '@/components/maps/PredictionMap';
import { mdiPlay, mdiPause } from '@mdi/js';
import Icon from '@mdi/react';
import { Sensor } from '@/store/sensor';
import { createChannel, createClient } from 'nice-grpc-web';
import {
    CreateSimulationResponse,
    SimulationInterfaceServiceDefinition,
} from '@/proto/simulation/frontend';
import {
    SimulationManagerClient,
    SimulationManagerDefinition,
    SimulationFrameRequest,
} from '@/proto/simulation/simulation-manager';
import * as Stream from 'stream';
import ToastNotification from '@/components/notification/ToastNotification';

const PredictionMap = dynamic<PredictionMapProps>(() => import('@/components/maps/PredictionMap'), {
    ssr: false,
});
function SimulationPage() {
    const params = useParams<{ id: string }>();
    const [twinState, dispatch] = useContext(TwinContext);
    const [currentTime, setCurrentTime] = useState(0);
    const [intervalItem, setIntervalItem] = useState<NodeJS.Timeout>();
    const selectedItemsRef = useRef(currentTime);

    //Update the reference when state changes
    useEffect(() => {
        selectedItemsRef.current = currentTime;
    }, [currentTime]);

    if (!twinState.current) {
        return <h1>Please select a Twin</h1>;
    }
    const startStopSimulation = () => {
        if (intervalItem) {
            clearInterval(intervalItem);
            setIntervalItem(undefined);
            return;
        }
        setIntervalItem(
            setInterval(function () {
                setCurrentTime(selectedItemsRef.current + 1);
            }, 500)
        );
    };

    return (
        <div className='flex flex-col h-full'>
            <Card className='flex-auto'>
                <div className='flex h-full grid grid-cols-12'>
                    <div className='h-full col-span-9'>
                        <PredictionMap twin={twinState.current} />
                    </div>

                    <div className='col-span-3 mx-6'>
                        <div className='h-full space-y-1'>
                            <Button
                                onClick={() => {
                                    ToastNotification('warning', 'Currently not implemented');
                                }}
                                className='w-full'
                                color='indigo'
                                theme={{
                                    color: {
                                        indigo: 'bg-indigo-600 text-white ring-indigo-600',
                                    },
                                }}
                            >
                                Create new simulation from here
                            </Button>
                            <Card>
                                <h1>hello world</h1>
                            </Card>
                            <Card>
                                <h1>hello world</h1>
                            </Card>
                            <Card>
                                <h1>hello world</h1>
                            </Card>
                        </div>
                    </div>
                </div>
            </Card>

            <Card className='mt-1'>
                <div className='flex'>
                    <div className='flex'>
                        <a href='#' onClick={startStopSimulation}>
                            {intervalItem ? (
                                <Icon path={mdiPause} size={1.2} />
                            ) : (
                                <Icon path={mdiPlay} size={1.2} />
                            )}
                        </a>
                        <p className='text-center p-1 px-3'> 10/03/2022 15:23:10 {currentTime}</p>
                    </div>
                    <div className='flex-auto'>
                        <RangeSlider
                            id='default-range'
                            className='object-fill'
                            onChange={e => setCurrentTime(parseFloat(e.target.value))}
                            value={currentTime.toString()}
                        />
                    </div>
                </div>
            </Card>
        </div>
    );
}

export default SimulationPage;
