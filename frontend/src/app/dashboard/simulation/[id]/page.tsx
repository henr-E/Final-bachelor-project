'use client';
import dynamic from 'next/dynamic';
import { RangeSlider, Card, Spinner, Label, TextInput, Tooltip, Badge } from 'flowbite-react';
import { useCallback, useContext, useEffect, useRef, useState } from 'react';
import { useParams, useRouter } from 'next/navigation';
import { TwinContext } from '@/store/twins';
import { PredictionMapProps } from '@/components/maps/PredictionMap';
import { mdiPlay, mdiPause, mdiCogOutline, mdiCheck, mdiSourceBranchPlus } from '@mdi/js';
import Icon from '@mdi/react';
import { createChannel, WebsocketTransport } from 'nice-grpc-web';
import {
    SimulationFrame,
    SimulationFrameRequest,
    SimulationId,
} from '@/proto/simulation/simulation-manager';
import {
    SimulationInterfaceServiceDefinition,
    Simulation,
    DeepPartial,
} from '@/proto/simulation/frontend';
import { uiBackendServiceUrl } from '@/api/urls';
import MapFrame, { FrameToMapInformation } from '@/components/maps/MapFrame';
import ToastNotification from '@/components/notification/ToastNotification';
import CreateSimulationModal from '@/components/modals/CreateSimulationModal';
import { BackendGetSimulationDetails } from '@/api/simulation/crud';
import { clientAuthLayer } from '@/api/protecteRequestFactory';

const PredictionMap = dynamic<PredictionMapProps>(() => import('@/components/maps/PredictionMap'), {
    ssr: false,
});
interface SimulationSettings {
    playBackSpeed: number;
    bufferSize: number;
}

function SimulationPage() {
    const params = useParams<{ id: string }>();
    const [twinState, dispatch] = useContext(TwinContext);
    const [currentTime, setCurrentTime] = useState(0);
    const [intervalItem, setIntervalItem] = useState<NodeJS.Timeout>();
    const currentTimeRef = useRef(currentTime);
    const [maxAvailableFrames, setMaxAvailableFrames] = useState<number>(0);
    const intervalItemRef = useRef(intervalItem);
    const [settingMenu, setSettingMenu] = useState<boolean>(false);
    const [settings, setSettings] = useState<SimulationSettings>({
        playBackSpeed: 500,
        bufferSize: 20,
    });
    const [sliding, setSliding] = useState<boolean>(false);
    const slidingRef = useRef(sliding);
    const settingsRef = useRef(settings);
    const [isCreateSimulationModalOpen, setIsCreateSimulationModalOpen] = useState(false);
    const [frames, setFrames] = useState<Array<SimulationFrame>>(
        Array(maxAvailableFrames).fill(undefined)
    );
    const framesRef = useRef(frames);
    const [clickedItem, setClickedItem] = useState<number>(0);
    const [simulation, setSimulation] = useState<Simulation>();
    const router = useRouter();

    const loadSimulations = useCallback(() => {
        /**
         * AsyncIterable to load frames via stream
         */
        async function* framePlayer(): AsyncIterable<DeepPartial<SimulationFrameRequest>> {
            let loadedFrames = new Set<number>();
            let loadFrame: number = 0;
            while (true) {
                //Check if frame is loaded when not, ask frame on server
                if (!loadedFrames.has(loadFrame)) {
                    loadedFrames.add(loadFrame);
                    yield SimulationFrameRequest.create({
                        simulationId: SimulationId.create({
                            uuid: params.id,
                        }),
                        frameNr: loadFrame,
                    });
                }

                loadFrame++;

                //Wait for user to pres start or to navigate on scroll bar
                const oldFrameNumber = currentTimeRef.current;
                while (
                    (oldFrameNumber == currentTimeRef.current &&
                        currentTimeRef.current + settingsRef.current.bufferSize < loadFrame) ||
                    slidingRef.current
                ) {
                    await new Promise(resolve => {
                        setTimeout(resolve, 250);
                    });
                }

                //If user clicks somewhere random on the slider
                if (oldFrameNumber !== currentTimeRef.current) {
                    loadFrame = currentTimeRef.current;
                }
            }
        }

        /**
         * Process frames using framePlayer, wait for new frame and adds to frame timeline
         */
        async function loadGraph() {
            let serverUrl = uiBackendServiceUrl;
            if (uiBackendServiceUrl.slice(0, 4) !== 'http') {
                serverUrl = window.location.origin;
            }
            const channel = createChannel(serverUrl, WebsocketTransport());
            const client = clientAuthLayer.create(SimulationInterfaceServiceDefinition, channel);

            for await (const response of client.getSimulationFrames(framePlayer())) {
                if (!response.request) return;

                const cIndex = response.request.frameNr;

                let state = response as SimulationFrame;
                setFrames(tempFrames => [
                    ...tempFrames.slice(0, cIndex),
                    state,
                    ...tempFrames.slice(cIndex + 1),
                ]);
            }
        }

        async function loadSimulations() {
            try {
                await BackendGetSimulationDetails(params.id).then(r => {
                    setMaxAvailableFrames(r.framesLoaded);
                    setFrames(Array(r.framesLoaded).fill(undefined));
                    setSimulation(r);
                    let _ = loadGraph(); //Start loading frames
                });
            } catch (error) {
                console.error('Failed to load simulation details');
                ToastNotification('error', 'Failed to load simulation details');
                router.push('/dashboard/simulation/');
            }
        }

        let _ = loadSimulations();
    }, [params.id, router]);

    useEffect(() => {
        loadSimulations();
    }, [loadSimulations]);

    useEffect(() => {
        currentTimeRef.current = currentTime;
        intervalItemRef.current = intervalItem;
        slidingRef.current = sliding;
        settingsRef.current = settings;
        framesRef.current = frames;
    }, [currentTime, intervalItem, sliding, settings, frames]);

    const isMounted = useRef(false);

    useEffect(() => {
        console.log('check', isMounted.current, twinState.current?.id);

        //When loaded from url, this will be not set initially and thus return
        if (!twinState.current?.id) {
            return;
        }
        //When there is a twin set, set isMounted to detect if twin changes
        if (!isMounted.current) {
            isMounted.current = true;
            return;
        }
        //Twin changed, return back to simulation overview
        router.push('/dashboard/simulation/');
        // eslint-disable-next-line
    }, [twinState.current?.id]);

    const showItem = (id: number) => {
        setClickedItem(id);
    };

    if (!twinState.current) {
        return <h1>Please select a Twin</h1>;
    }

    /**
     * Start or stop autoplay
     */
    const startStopSimulation = () => {
        //stop time when already running
        if (intervalItem) {
            clearInterval(intervalItem);
            setIntervalItem(undefined);
            return;
        }

        //When end reached go back to time 0
        if (currentTimeRef.current >= maxAvailableFrames - 1) {
            setCurrentTime(0);
        }

        //Create a interval to change the time
        setIntervalItem(
            setInterval(function () {
                if (currentTimeRef.current >= maxAvailableFrames - 1) {
                    clearInterval(intervalItemRef.current);
                    setIntervalItem(undefined);
                    return;
                }
                if (!framesRef.current[currentTimeRef.current]) {
                    clearInterval(intervalItemRef.current);
                    setIntervalItem(undefined);
                    return;
                }
                setCurrentTime(currentTimeRef.current + 1);
            }, settings.playBackSpeed)
        );
    };

    const createNewSimulation = () => {
        setIsCreateSimulationModalOpen(true);
        console.log('new simulation');
    };

    return (
        <div className='flex flex-col h-full'>
            <Card className='flex-auto'>
                <h1 className=''>{simulation?.name}</h1>
                <div className='flex flex-wrap gap-2'>
                    <Badge color='gray'>
                        Start time:{' '}
                        {new Date((simulation?.startDateTime || 0) * 1000).toLocaleString()}
                    </Badge>
                    <Badge color='gray'>
                        End time: {new Date((simulation?.endDateTime || 0) * 1000).toLocaleString()}
                    </Badge>
                    <Badge color='gray'>Status: {simulation?.status}</Badge>
                </div>
                <MapFrame
                    twin={twinState.current}
                    frame={frames[currentTime]}
                    frameNr={currentTime}
                ></MapFrame>
            </Card>
            <Card className='mt-1'>
                <div className='flex'>
                    <div className='flex'>
                        <a onClick={startStopSimulation} style={{ cursor: 'pointer' }}>
                            {intervalItem ? (
                                <Icon path={mdiPause} size={1.2} />
                            ) : (
                                <Icon path={mdiPlay} size={1.2} />
                            )}
                        </a>
                        <div className='px-3 relative'>
                            {settingMenu && (
                                <Card className='absolute z-30 bottom-[40px] -left-5 w-[200px]'>
                                    <h1 className='text-lg'>Settings</h1>
                                    <div className='flex flex-row w-full space-x-3 pt-1'>
                                        <div>
                                            <div className='mb-2 block'>
                                                <Label
                                                    htmlFor='playspeed'
                                                    value='Time between frames (ms)'
                                                />
                                            </div>
                                            <TextInput
                                                id='playspeed'
                                                type='number'
                                                value={settings.playBackSpeed}
                                                maxLength={5000}
                                                required
                                                onChange={e =>
                                                    setSettings({
                                                        playBackSpeed: parseInt(e.target.value),
                                                        bufferSize: settings.bufferSize,
                                                    })
                                                }
                                            />
                                        </div>
                                    </div>
                                    <div className='flex flex-row w-full space-x-3 pt-1'>
                                        <div>
                                            <div className='mb-2 block'>
                                                <Label htmlFor='buffersizer' value='Buffer size' />
                                            </div>
                                            <TextInput
                                                id='endtime'
                                                type='number'
                                                value={settings.bufferSize}
                                                maxLength={5000}
                                                required
                                                minLength={5}
                                                onChange={e => {
                                                    setSettings({
                                                        playBackSpeed: settings.playBackSpeed,
                                                        bufferSize: parseInt(e.target.value),
                                                    });
                                                }}
                                            />
                                        </div>
                                    </div>
                                </Card>
                            )}
                            <Tooltip content='Player settings'>
                                <a
                                    style={{ cursor: 'pointer' }}
                                    onClick={() => {
                                        setSettingMenu(!settingMenu);
                                        if (intervalItem) {
                                            startStopSimulation();
                                        }
                                    }}
                                >
                                    <Icon
                                        path={mdiCogOutline}
                                        size={1.2}
                                        rotate={settingMenu ? 0 : -150}
                                    />
                                </a>
                            </Tooltip>
                        </div>
                        <Tooltip content='Create a new simulation from this frame'>
                            <a href='#' className='' onClick={createNewSimulation}>
                                {<Icon path={mdiSourceBranchPlus} size={1.2} />}
                            </a>
                        </Tooltip>
                        <p className='text-center p-1 px-3 w-28'>
                            {currentTime}/{maxAvailableFrames - 1}
                        </p>
                    </div>
                    <div className='flex-auto'>
                        <RangeSlider
                            id='default-range'
                            className='object-fill'
                            min={0}
                            max={maxAvailableFrames - 1}
                            onMouseDown={e => {
                                setSliding(true);
                                console.log('disable server loading');
                            }}
                            onMouseUp={e => {
                                setSliding(false);
                                console.log('enable server loading');
                            }}
                            onChange={e => {
                                setCurrentTime(parseFloat(e.target.value));
                            }}
                            value={currentTime}
                        />
                    </div>
                    <div className='flex'>
                        {frames
                            .slice(currentTime, currentTime + settings.bufferSize)
                            .filter(function (value) {
                                return value !== undefined;
                            }).length == settings.bufferSize ? (
                            <Tooltip
                                content={`Buffer of size ${settings.bufferSize} frames completely loaded`}
                            >
                                <Icon
                                    path={mdiCheck}
                                    color='green'
                                    size={1}
                                    className='content-center'
                                />
                            </Tooltip>
                        ) : (
                            <Tooltip
                                content={`Buffer loading: ${
                                    frames
                                        .slice(currentTime, currentTime + settings.bufferSize)
                                        .filter(function (value) {
                                            return value !== undefined;
                                        }).length
                                }/${settings.bufferSize} frames completely loaded`}
                            >
                                <Spinner aria-label='Default status example' className='ml-2' />
                            </Tooltip>
                        )}
                    </div>
                </div>
            </Card>
            {frames[currentTime] && (
                <CreateSimulationModal
                    globalComponents={JSON.stringify(frames[currentTime].state?.globalComponents)}
                    title={'Branch/' + simulation?.name + '/' + currentTime}
                    initialNodes={FrameToMapInformation(frames[currentTime])[0]}
                    initialEdges={FrameToMapInformation(frames[currentTime])[1]}
                    timeStepDelta={Math.round(
                        ((simulation?.endDateTime || 60) - (simulation?.startDateTime || 0)) /
                            (simulation?.framesLoaded || 1)
                    )}
                    startDate={
                        simulation?.startDateTime
                            ? new Date(
                                  simulation?.startDateTime * 1000 +
                                      1000 *
                                          currentTime *
                                          (((simulation?.endDateTime || 60) -
                                              simulation?.startDateTime) /
                                              (simulation?.framesLoaded || 1))
                              )
                            : undefined
                    }
                    endDate={
                        simulation?.endDateTime
                            ? new Date(simulation?.endDateTime * 1000)
                            : undefined
                    }
                    startTime={
                        simulation?.startDateTime
                            ? new Date(
                                  simulation?.startDateTime * 1000 +
                                      1000 *
                                          currentTime *
                                          (((simulation?.endDateTime || 60) -
                                              simulation?.startDateTime) /
                                              (simulation?.framesLoaded || 1))
                              ).toLocaleTimeString('nl-BE')
                            : undefined
                    }
                    endTime={
                        simulation?.endDateTime
                            ? new Date(simulation?.endDateTime * 1000).toLocaleTimeString('nl-BE')
                            : undefined
                    }
                    isModalOpen={isCreateSimulationModalOpen}
                    closeModal={() => {
                        setIsCreateSimulationModalOpen(false);
                    }}
                    parent={[simulation?.id || 0, simulation?.name || '', currentTime]}
                ></CreateSimulationModal>
            )}
        </div>
    );
}
export default SimulationPage;
