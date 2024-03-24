'use client';
import dynamic from "next/dynamic";
import {RangeSlider, Card, Button, Spinner} from 'flowbite-react';
import {useCallback, useContext, useEffect, useRef, useState} from "react";
import {useParams} from 'next/navigation'
import {TwinContext} from "@/store/twins";
import {PredictionMapProps} from "@/components/maps/PredictionMap";
import {mdiPlay, mdiPause} from '@mdi/js';
import Icon from '@mdi/react';
import {createChannel, createClient} from "nice-grpc-web";
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

const PredictionMap = dynamic<PredictionMapProps>(() => import("@/components/maps/PredictionMap"), {ssr: false});

function SimulationPage() {
    const params = useParams<{ id: string }>();
    const [simulation, setSimulation] = useState<Simulation>();
    const [twinState, dispatch] = useContext(TwinContext);
    const [currentTime, setCurrentTime] = useState(0);
    const [intervalItem, setIntervalItem] = useState<NodeJS.Timeout>();
    const currentTimeRef = useRef(currentTime);
    const [maxAvailableFrames, setMaxAvailableFrames] = useState<number>(0);
    const intervalItemRef = useRef(intervalItem);

    //TODO refactor to use simlatiuon frame from prot file
    const [mapItemsTimeLine, setMapItemsTimeLine] = useState<Array<Map<number, MapItemType>>>(Array(maxAvailableFrames).fill(undefined));
    const [globalComponentsTimeLine, setGlobalComponentsTimeLine] = useState<{
        [p: string]: any
    }[]>(Array(maxAvailableFrames).fill({"No vars set": ""}));

    const mapItemsTimeLineRef = useRef(mapItemsTimeLine);
    const [clickedItem, setClickedItem] = useState<number>(0);

    const loadSimulations = useCallback(() => {
        async function* framePlayer(): AsyncIterable<DeepPartial<SimulationFrameRequest>> {
            for (let i = currentTimeRef.current; i < maxAvailableFrames; i++) {
                //Check if person skips
                if (currentTimeRef.current > i)
                    i = currentTimeRef.current;

                //Check if frame is loaded
                if (mapItemsTimeLineRef.current[i] !== undefined)
                    continue

                //Ask frame server
                yield SimulationFrameRequest.create({
                    simulationId: SimulationId.create({
                        uuid: params.id
                    }),
                    frameNr: i
                });
            }
        }
        async function loadGraph() {
            for await (const response of client.getSimulationFrames(framePlayer())) {
                if (!response.request)
                    return
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

        const channel = createChannel(uiBackendServiceUrl);
        const client = createClient(SimulationInterfaceServiceDefinition, channel);
        client.getSimulation({uuid: params.id}).then((r: Simulation) => {
            setSimulation(r);
            setMaxAvailableFrames(r.framesLoaded);
            setMapItemsTimeLine(Array(maxAvailableFrames).fill(undefined));
            let _ = loadGraph();
        });
    }, [params.id, maxAvailableFrames]);


    useEffect(() => {
        loadSimulations();
    }, [loadSimulations]);


    useEffect(() => {
        currentTimeRef.current = currentTime;
        mapItemsTimeLineRef.current = mapItemsTimeLine;
        intervalItemRef.current = intervalItem;
    }, [currentTime, mapItemsTimeLine, intervalItem]);

    const showItem = (id: number) => {
        setClickedItem(id);
    }

    const channel = createChannel(uiBackendServiceUrl);
    const client = createClient(SimulationInterfaceServiceDefinition, channel);


    if (!twinState.current) {
        return <h1>Please select a Twin</h1>;
    }
    const startStopSimulation = () => {
        if (intervalItem) {
            clearInterval(intervalItem);
            setIntervalItem(undefined);
            return;
        }
        setIntervalItem(setInterval(function () {
            if(currentTimeRef.current >= maxAvailableFrames - 1 || !mapItemsTimeLineRef.current[currentTimeRef.current]){
                clearInterval(intervalItemRef.current);
                setIntervalItem(undefined);
                return
            }
            setCurrentTime(currentTimeRef.current + 1);

        }, 500));
    }

    return (
        <div className="flex flex-col h-full">
            <Card className="flex-auto">
                <div className="flex h-full grid grid-cols-12">
                    <div className={`h-full col-span-9 ${!mapItemsTimeLine[currentTime] ? 'blur-sm' : ''}`}>
                        <PredictionMap twin={twinState.current} mapItems={mapItemsTimeLine[currentTime]? Array.from(mapItemsTimeLine[currentTime].values()): []}/>
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
                                    {globalComponentsTimeLine[currentTime] && <JsonToTable json={globalComponentsTimeLine[currentTime]} />}
                                </Card>
                                <Card className="overflow-x-scroll">
                                    <h1>Vars of selected component</h1>
                                    {mapItemsTimeLineRef.current[currentTime] && ((mapItemsTimeLineRef.current[currentTime]).get(clickedItem))?.components && <JsonToTable json={(mapItemsTimeLineRef.current[currentTime].get(clickedItem)?.components)} />}
                                </Card>
                            </div> :
                            <div className="col-span-3 mx-6">
                                <Card className="text-center items-center">
                                    <Spinner aria-label="Medium sized spinner example" size="md" />
                                    <h1>Loading frame</h1>
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
                        <p className="text-center p-1 px-3">frame: {currentTime}/{maxAvailableFrames - 1}</p>
                    </div>
                    <div className="flex-auto">
                        <RangeSlider id="default-range" className="object-fill"
                                     min={0}
                                     max={maxAvailableFrames - 1}
                                     onChange={(e) => {setCurrentTime(parseFloat(e.target.value));}}
                                     value={currentTime}
                        />
                    </div>
                </div>
            </Card>
        </div>
    );
}

export default SimulationPage;
