'use client';
import dynamic from "next/dynamic";
import {RangeSlider, Card, Button, Spinner, Label, TextInput, Select, Tooltip} from 'flowbite-react';
import {useCallback, useContext, useEffect, useRef, useState} from "react";
import {useParams, useRouter} from 'next/navigation'
import {TwinContext} from "@/store/twins";
import {PredictionMapProps} from "@/components/maps/PredictionMap";
import {mdiPlay, mdiPause, mdiCogOutline, mdiCheck} from '@mdi/js';
import Icon from '@mdi/react';
import {
    ClientMiddlewareCall,
    createChannel,
    createClient,
    createClientFactory,
    WebsocketTransport
} from "nice-grpc-web";
import {
    SimulationFrameRequest,
    SimulationId
} from "@/proto/simulation/simulation-manager"
import {
    SimulationInterfaceServiceDefinition,
    Simulation,
    DeepPartial
} from "@/proto/simulation/frontend";
import ToastNotification from '@/components/notification/ToastNotification';
import {uiBackendServiceUrl} from "@/api/urls";
import {LineItem, MapItems, MapItemType, NodeItem} from "@/components/maps/MapItem";
import { JsonToTable } from "react-json-to-table";
import {CallOptions} from "nice-grpc-common";
import {LatLngExpression} from "leaflet"
import {BackendGetSimulationDetails} from "@/api/simulation/crud";

const PredictionMap = dynamic<PredictionMapProps>(() => import("@/components/maps/PredictionMap"), {ssr: false});
interface SimulationSettings{
    playBackSpeed: number;
    bufferSize: number;
}
const PredictionMapImport = dynamic<PredictionMapProps>(() => import("@/components/maps/PredictionMap"), {ssr: false});

function SimulationPage() {
    const params = useParams<{ id: string }>();
    const [twinState, dispatch] = useContext(TwinContext);
    const [currentTime, setCurrentTime] = useState(0);
    const [intervalItem, setIntervalItem] = useState<NodeJS.Timeout>();
    const currentTimeRef = useRef(currentTime);
    const [maxAvailableFrames, setMaxAvailableFrames] = useState<number>(0);
    const intervalItemRef = useRef(intervalItem);
    const [settingMenu, setSettingMenu] = useState<boolean>(false);
    const [settings, setSettings] = useState<SimulationSettings>({playBackSpeed: 500, bufferSize: 20});
    const [sliding, setSliding] = useState<boolean>(false);
    const slidingRef = useRef(sliding);
    const settingsRef = useRef(settings);
    const router = useRouter();

    //TODO refactor to use simulation frame from proto file (other issue)
    const [mapItemsTimeLine, setMapItemsTimeLine] = useState<Array<Map<number, MapItemType>>>(Array(maxAvailableFrames).fill(undefined));
    const [globalComponentsTimeLine, setGlobalComponentsTimeLine] = useState<{
        [p: string]: any
    }[]>(Array(maxAvailableFrames).fill({"No vars set": ""}));

    const mapItemsTimeLineRef = useRef(mapItemsTimeLine);
    const [clickedItem, setClickedItem] = useState<number>(0);

    const loadSimulations = useCallback(() => {
        /**
         * AsyncIterable to load frames via stream
         */
        async function* framePlayer(): AsyncIterable<DeepPartial<SimulationFrameRequest>> {
            let loadedFrames = new Set<number>();
            let loadFrame: number = 0;
            while (true) {
                //Check if frame is loaded when not, ask frame on server
                if (!loadedFrames.has(loadFrame)){
                    loadedFrames.add(loadFrame)
                    yield SimulationFrameRequest.create({
                        simulationId: SimulationId.create({
                            uuid: params.id
                        }),
                        frameNr: loadFrame
                    });
                }

                loadFrame++;

                //Wait for user to pres start or to navigate on scroll bar
                const oldFrameNumber = currentTimeRef.current;
                while ((oldFrameNumber == currentTimeRef.current && currentTimeRef.current + settingsRef.current.bufferSize < loadFrame) || slidingRef.current) {
                    await new Promise((resolve) => {
                        setTimeout(resolve, 250)
                    });
                }

                //If user clicks somewhere random on the slider
                if(oldFrameNumber !== currentTimeRef.current){
                    loadFrame = currentTimeRef.current;
                }
            }
        }

        /**
         * Process frames using framePlayer, wait for new frame and adds to frame timeline
         */
        async function loadGraph() {
            let serverUrl = uiBackendServiceUrl;
            if(uiBackendServiceUrl.slice(0,4) !== "http"){
                serverUrl = window.location.origin;
            }
            const channel = createChannel(serverUrl, WebsocketTransport());
            const client = createClient(SimulationInterfaceServiceDefinition, channel);

            for await (const response of client.getSimulationFrames(framePlayer())) {
                if (!response.request)
                    return

                //Convert proto frame to map items (will be changed in #182)
                const currentFrame = response.request.frameNr;
                let nodeItems = response.state?.graph?.nodes.map((item) => {
                    const newItem: NodeItem = {
                        components: item.components,
                        inactive: false,
                        location: [item.latitude, item.longitude],
                        name: item.id.toString(),
                        id: item.id,
                        type: MapItems.TransformerHouse,
                        eventHandler: {click: (e) => showItem(item.id)}
                    }
                    return newItem;
                });

                if (!nodeItems)
                    return
                let mapItems = new Map(nodeItems.map(i => [i.id, i]));

                let lineItems = response.state?.graph?.edge.map(item => {
                    let connectingItems = [mapItems.get(item.from) as NodeItem, mapItems.get(item.to) as NodeItem];
                    const newItem: LineItem = {
                        name: item.componentType,
                        id: item.id,
                        components: item.componentData,
                        items: connectingItems,
                        type: MapItems.Line,
                        eventHandler: {click: (e) => showItem(item.id)}
                    }
                    return newItem;
                });

                if (!lineItems)
                    return
                let lineItemsMap = new Map(lineItems.map(i => [i.id, i]));

                // @ts-ignore
                let result: Map<number, MapItemType> = new Map([...mapItems, ...lineItemsMap]);

                const cIndex = response.request.frameNr;
                setMapItemsTimeLine(tesmpMapItemTimeLine => [
                    ...tesmpMapItemTimeLine.slice(0, cIndex),
                    result,
                    ...tesmpMapItemTimeLine.slice(cIndex + 1)
                ]);

                const globalVarItem = response.state?.globalComponents || {"No vars set": ""};
                setGlobalComponentsTimeLine(globalComponentsTimeLine => [
                    ...globalComponentsTimeLine.slice(0, cIndex),
                    globalVarItem,
                    ...globalComponentsTimeLine.slice(cIndex + 1)
                ]);
            }
        }

        async function loadSimulations() {
            try {
                await BackendGetSimulationDetails(params.id).then(r => {
                    setMaxAvailableFrames(r.framesLoaded);
                    setMapItemsTimeLine(Array(maxAvailableFrames).fill(undefined));
                    let _ = loadGraph();
                });
            } catch (error) {
                console.error("Failed to load simulation details");
            }
        }

        let _ = loadSimulations();
    }, [params.id, maxAvailableFrames]);

    useEffect(() => {
        loadSimulations();
    }, [loadSimulations]);

    useEffect(() => {
        currentTimeRef.current = currentTime;
        mapItemsTimeLineRef.current = mapItemsTimeLine;
        intervalItemRef.current = intervalItem;
        slidingRef.current = sliding;
        settingsRef.current = settings;
    }, [currentTime, mapItemsTimeLine, intervalItem, sliding, settings]);

    const isMounted = useRef(false);

    useEffect(() => {
        if (!isMounted.current) {
            isMounted.current = true;
            return;
        }
        router.push('/dashboard/simulation/');
        // eslint-disable-next-line
    }, [twinState.current?.id]);


    const showItem = (id: number) => {
        setClickedItem(id);
    }

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
        if(currentTimeRef.current >= maxAvailableFrames - 1){
            setCurrentTime(0);
        }

        //Create a interval to change the time
        setIntervalItem(setInterval(function () {
            if(currentTimeRef.current >= maxAvailableFrames - 1){
                clearInterval(intervalItemRef.current);
                setIntervalItem(undefined);
                return
            }
            if(!mapItemsTimeLineRef.current[currentTimeRef.current]){
                clearInterval(intervalItemRef.current);
                setIntervalItem(undefined);
                return
            }
            setCurrentTime(currentTimeRef.current + 1);

        }, settings.playBackSpeed));
    }


    return (
        <div className="flex flex-col h-full">
            <Card className="flex-auto">
                <div className="flex h-full grid grid-cols-12">
                    <div className={`h-full col-span-9 ${!mapItemsTimeLine[currentTime] ? 'blur-sm' : ''}`}>
                        <PredictionMapImport twin={twinState.current}
                                             mapItems={mapItemsTimeLine[currentTime] ? Array.from(mapItemsTimeLine[currentTime].values()) : []}/>
                    </div>

                    <div className="col-span-3 mx-6">
                        {mapItemsTimeLine[currentTime] ?
                            <div className="h-full space-y-1">
                                <Button
                                    onClick={() => {
                                        ToastNotification("warning", "Currently not implemented")
                                    }}
                                    className="w-full"
                                    color="indigo"
                                    theme={{color: {indigo: 'bg-indigo-600 text-white ring-indigo-600'}}}
                                >
                                    Create new simulation from here
                                </Button>
                                <Card className="overflow-x-scroll">
                                    <h1>Global variable</h1>
                                    {globalComponentsTimeLine[currentTime] &&
                                        <JsonToTable json={globalComponentsTimeLine[currentTime]}/>}
                                </Card>
                                <Card className="overflow-x-scroll">
                                    <h1>Vars of selected component</h1>
                                    {mapItemsTimeLineRef.current[currentTime] && ((mapItemsTimeLineRef.current[currentTime]).get(clickedItem))?.components &&
                                        <JsonToTable
                                            json={(mapItemsTimeLineRef.current[currentTime].get(clickedItem)?.components)}/>}
                                </Card>
                            </div> :
                            <div className="col-span-3 mx-6">
                                <Card className="text-center items-center">
                                    <Spinner aria-label="Medium sized spinner example" size="md"/>
                                    {sliding ?
                                        <div><h3 className="text-lg">Not loaded into buffer</h3><h4>leave timeline to
                                            load frame: {currentTime}</h4></div> : <h1>Loading frame</h1>}
                                </Card>
                            </div>
                        }

                    </div>
                </div>
            </Card>
            <Card className="mt-1">
                <div className="flex">
                    <div className="flex">
                        <a href="#" onClick={startStopSimulation}>
                            {(intervalItem) ? <Icon path={mdiPause} size={1.2}/> : <Icon path={mdiPlay} size={1.2}/>}
                        </a>

                        <div className="px-3 relative">
                            {
                                settingMenu && <Card className="absolute z-30 bottom-[40px] -left-5 w-[200px]">
                                    <h1 className="text-lg">Settings</h1>
                                    <div className="flex flex-row w-full space-x-3 pt-1">
                                        <div>
                                            <div className="mb-2 block">
                                                <Label htmlFor="playspeed" value="Time between frames (ms)"/>
                                            </div>
                                            <TextInput id="playspeed" type="number" value={settings.playBackSpeed}
                                                       maxLength={5000} required
                                                       onChange={(e) => setSettings({
                                                           playBackSpeed: parseInt(e.target.value),
                                                           bufferSize: settings.bufferSize
                                                       })}/>
                                        </div>
                                    </div>
                                    <div className="flex flex-row w-full space-x-3 pt-1">
                                        <div>
                                            <div className="mb-2 block">
                                                <Label htmlFor="buffersizer" value="Buffer size"/>
                                            </div>
                                            <TextInput id="endtime" type="number" value={settings.bufferSize}
                                                       maxLength={5000} required
                                                       minLength={5}
                                                       onChange={(e) => {setSettings({playBackSpeed: settings.playBackSpeed, bufferSize: parseInt(e.target.value)})}}/>
                                        </div>
                                    </div>
                                </Card>
                            }
                            <a href="#" onClick={() => {setSettingMenu(!settingMenu); if(intervalItem) {startStopSimulation()} }}>
                                <Icon path={mdiCogOutline} size={1.2} rotate={settingMenu ? 0 : -150}/>
                            </a>
                        </div>
                        <p className="text-center p-1 px-3 w-28">{currentTime}/{maxAvailableFrames - 1}</p>
                    </div>
                    <div className="flex-auto">
                        <RangeSlider id="default-range" className="object-fill"
                                     min={0}
                                     max={maxAvailableFrames - 1}
                                     onMouseDown={(e) => {
                                         setSliding(true);
                                         console.log("disable server loading")
                                     }}
                                     onMouseUp={(e) => {
                                         setSliding(false);
                                         console.log("enable server loading")
                                     }}
                                     onChange={(e) => {
                                         setCurrentTime(parseFloat(e.target.value));
                                     }}
                                     value={currentTime}
                        />
                    </div>
                    <div className="flex">
                        {
                            mapItemsTimeLine.slice(currentTime, currentTime + settings.bufferSize).filter(function (value) {
                                return value !== undefined
                            }).length == settings.bufferSize ?
                                <Tooltip content={`Buffer of size ${settings.bufferSize} frames completely loaded`} >
                                    <Icon
                                        path={mdiCheck}
                                        color='green'
                                        size={1}
                                        className='content-center'
                                    />
                                </Tooltip>:
                                <Tooltip content={`Buffer loading: ${mapItemsTimeLine.slice(currentTime, currentTime + settings.bufferSize).filter(function (value) {
                                    return value !== undefined
                                }).length}/${settings.bufferSize} frames completely loaded`} >
                                    <Spinner aria-label="Default status example" className="ml-2"/>
                                </Tooltip>
                        }
                    </div>
                </div>
            </Card>
        </div>
    );
}
export default SimulationPage;
