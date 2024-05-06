'use client';
import { Badge, Button } from 'flowbite-react';
import { Icon } from '@mdi/react';
import {
    mdiCursorPointer,
    mdiHomeLightningBoltOutline,
    mdiPlus,
    mdiTransitConnectionHorizontal,
    mdiWindTurbine,
} from '@mdi/js';
import { MutableRefObject, useContext, useEffect, useRef, useState } from 'react';
import { PredictionMapProps } from '@/components/maps/PredictionMap';
import dynamic from 'next/dynamic';
import { TwinContext } from '@/store/twins';
import { BuildingItem, LineItem, MapItems, MapItemType, NodeItem } from '@/components/maps/MapItem';
import ToastNotification from '@/components/notification/ToastNotification';
import { TwinServiceDefinition, presetObject } from '@/proto/twins/twin';
import { createChannel, createClient } from 'nice-grpc-web';
import { uiBackendServiceUrl } from '@/api/urls';
import { toast } from 'react-hot-toast';
import CreateIconsModal from '@/components/modals/CreateIconsModal';
import { BackendGetComponent, BackendGetSimulations } from '@/api/simulation/crud';
import { ComponentsInfo } from '@/proto/simulation/simulation-manager';

import { Sensor } from '@/proto/sensor/sensor-crud';
import { BackendCreateSensor, BackendGetSensors } from '@/api/sensor/crud';
import ShowSignalsModal from '@/components/modals/ShowSignalsModal';
import CreateSensorModal from '@/components/modals/CreateSensorModal';
import CustomJsonEditor from '@/components/CustomJsonEditor';
import {
    BackendDeleteBuilding,
    BackendGetAllPreset,
    BackendUndoDeleteBuilding,
} from '@/api/twins/crud';

/*
function addCursorState(name: string) {
    const len = Object.keys(CursorState).length;
    CursorState[name] = len;
}

const iconPaths = {
    [CursorState.PLACE_PRODUCER]: '/icons/home-lightning-bolt-outline.svg',
    [CursorState.PLACE_CONSUMER]: '/icons/wind-turbine.svg',
    [CursorState.CONNECT_ITEMS]: '/icons/transit-connection-horizontal.svg',
    [CursorState.MOVE_ITEMS]: '/icons/cursor-move.svg',
    [CursorState.GRAB]: 'hand',
    [CursorState.NONE]: 'hand',
};
*/
//TODO make variable

export interface MapEditorProps {
    nodeItemRef?: MutableRefObject<Map<number, NodeItem> | undefined>;
    edgeItemRef?: MutableRefObject<LineItem[] | undefined>;
    initialNodes?: Map<number, NodeItem>;
    initialEdges?: Array<LineItem>;
    noBuildings?: boolean;
}

const PredictionMapImport = dynamic<PredictionMapProps>(
    () => import('@/components/maps/PredictionMap'),
    { ssr: false }
);

function MapEditor({
    nodeItemRef,
    edgeItemRef,
    noBuildings,
    initialEdges,
    initialNodes,
}: MapEditorProps) {
    const [twinState, dispatchTwin] = useContext(TwinContext);
    const [currentPreset, setCurrentPreset] = useState<presetObject>();
    const [nodes, setNodes] = useState<Map<number, NodeItem>>(new Map<number, NodeItem>());
    const [edges, setEdges] = useState<Array<LineItem>>([]);
    const [selectedItems, setSelectedItems] = useState<Array<MapItemType>>([]);
    const nodesRef = useRef(nodes); //Use a reference because needed when called from eventHandlers
    const edgesRef = useRef(edges); //Use a reference because needed when called from eventHandlers
    const currentPresetRef = useRef(currentPreset); //Use a reference because needed when called from eventHandlers
    const selectedItemsRef = useRef(selectedItems); //Use a reference because needed when called from eventHandlers
    const [itemComponents, setItemComponents] = useState('{}');
    const [isCreateIconsModalOpen, setIsCreateIconModalOpen] = useState(false);
    const [selectedBuilding, setSelectedBuilding] = useState<BuildingItem | undefined>(undefined);

    const [isCreateSensorModalOpen, setIsCreateSensorModalOpen] = useState(false);
    const [isShowSignalsModalOpen, setIsShowSignalsModalOpen] = useState(false);
    const [selectedSensor, setSelectedSensor] = useState<Sensor>();
    const [presets, setPresets] = useState<Array<presetObject | undefined>>([]);
    const [typePreset, setTypePreset] = useState<Array<any>>([]);

    const getAllPreset = async () => {
        const response = await BackendGetAllPreset();
        if (response !== undefined) {
            setPresets(response);
        }
    };
    //Overwrite eventhandlers
    useEffect(() => {
        initialEdges?.map(item => {
            let newItem = item;
            newItem.eventHandler = { click: e => selectEdge(item.id) };
            setEdges(edges => [...edges, newItem]);
        });

        if (initialNodes)
            Array.from(initialNodes?.values())?.map(item => {
                let newItem = item;
                newItem.eventHandler = { click: e => selectEdge(item.id) };
                setNodes(map => new Map(map.set(item.id, newItem)));
            });
        getAllPreset().then();
    }, [initialNodes, initialEdges]);

    //Update the reference when state changes
    useEffect(() => {
        nodesRef.current = nodes;
        edgesRef.current = edges;
        currentPresetRef.current = currentPreset;
        selectedItemsRef.current = selectedItems;
        if (nodeItemRef) nodeItemRef.current = nodes;

        if (edgeItemRef) edgeItemRef.current = edges;
    }, [nodes, currentPreset, selectedItems, edges, nodeItemRef, edgeItemRef]);

    if (!twinState.current) {
        return <h1>Please select a Twin</h1>;
    }

    /**
     * Select an item on the map by id
     * @param index
     */
    function selectNode(index: number) {
        console.log('cursor', currentPresetRef.current);
        console.log(currentPresetRef.current?.name.substring(0, 5));
        if (currentPresetRef.current?.isEdge) {
            addLine(index);
            let item = nodesRef.current.get(index);
            if (item) setSelectedItems([...selectedItemsRef.current, item]);
            return;
        }
        let item = nodesRef.current.get(index);
        if (item) setSelectedItems([item]);
        if (nodesRef.current.get(index)?.components) {
            setItemComponents(JSON.stringify(nodesRef.current.get(index)?.components));
            return;
        }
        setItemComponents('{}');
    }

    /**
     * Select an item on the map by id
     * @param index
     */
    function selectEdge(index: number) {
        setSelectedBuilding(undefined);
        let item = edgesRef.current[index];
        if (item) setSelectedItems([item]);
        if (edgesRef.current[index]?.components) {
            setItemComponents(JSON.stringify(edgesRef.current[index]?.components));
            return;
        }
        setItemComponents('{}');
    }

    /**
     * Add a line between items
     * @param id the id of the item you want the line to pass
     */
    const addLine = (id: number) => {
        if (!nodesRef.current.get(id)) {
            return;
        }
        if (selectedItemsRef.current.length == 0) {
            const lineId = edgesRef.current.length;
            const newItem: LineItem = {
                name: 'item: ' + lineId,
                id: lineId,
                items: [nodesRef.current.get(id) as NodeItem],
                type: MapItems.Line,
                components: JSON.parse(currentPresetRef.current?.info || '{}'),
                eventHandler: { click: e => selectEdge(lineId) },
            };

            setEdges([...edgesRef.current, newItem]);
            return;
        }
        changePreset(undefined);
        console.log('node: ' + nodesRef.current.get(id)?.id);
        console.log(nodesRef.current.get(id));
        (edgesRef.current[edgesRef.current.length - 1] as LineItem).items.push(
            nodesRef.current.get(id) as NodeItem
        );
    };

    const changePreset = (preset: presetObject | undefined) => {
        setSelectedBuilding(undefined);
        setSelectedItems([]);
        setCurrentPreset(preset);
    };

    /**
     * Remove an item from the map
     * (currently set item to inactive)
     * @param id index id
     */
    const removeMapItem = (node: NodeItem | undefined) => {
        let removeItem = node;
        if (!removeItem) return;
        removeItem.inactive = true; //currently set item to inactive TODO
        setNodes(prevState => ({ ...prevState, id: removeItem }));
    };

    const onSelectBuilding = (building: BuildingItem) => {
        setSelectedBuilding(building);
        if (!currentPresetRef.current || currentPresetRef.current?.isEdge) {
            if (nodesRef.current.get(building.id)) selectNode(building.id);
            else setSelectedItems([]);
            return;
        }

        let id = building.id;
        let type = MapItems.TransformerHouse;
        console.log(typePreset, currentPresetRef.current);
        const newItem: NodeItem = {
            id: id,
            name: 'item: ' + id,
            location: building.location,
            type: type,
            components: JSON.parse(currentPresetRef.current?.info || '{}'),
            eventHandler: {},
        };
        if (newItem.components === undefined) {
            throw new Error('Unreachable: components should always be set.');
        }
        newItem.components['building'] = { building_id: building.id };

        console.log(newItem);
        building.nodeItem = newItem;

        setNodes(map => new Map(map.set(id, newItem)));
        selectNode(building.id);
        setItemComponents(JSON.stringify(newItem.components));
        setSelectedItems([newItem]);
    };

    const handleDeleteBuilding = async () => {
        if (selectedBuilding) {
            let response = await BackendDeleteBuilding(selectedBuilding?.id);
            if (response) {
                selectedBuilding.visible = false;
                ToastNotification('info', 'Building succesfully deleted.');
                setSelectedBuilding(undefined);
            }
        }
    };

    const handleUndoDeleteBuilding = async () => {
        if (selectedBuilding) {
            let response = await BackendUndoDeleteBuilding(selectedBuilding?.id);
            if (response) {
                selectedBuilding.visible = true;
                ToastNotification('info', 'Building succesfully restored.');
                setSelectedBuilding(undefined);
            }
        }
    };

    const handleCreateSensor = async (sensor: Sensor) => {
        let success = await BackendCreateSensor(sensor);
        if (!success) {
            ToastNotification('error', 'Failed to create sensor');
            return;
        }

        if (twinState.current) {
            let sensors = await BackendGetSensors(twinState.current?.id);
            dispatchTwin({ type: 'load_sensors', sensors: sensors });
        }

        ToastNotification('success', `Sensor is created`);
    };

    const handleClick = (sensor: Sensor) => {
        setIsShowSignalsModalOpen(true);
        setSelectedSensor(sensor);
    };

    const saveBuildingComponents = async (jsonString: string) => {
        if (!nodesRef?.current) {
            return;
        }
        try {
            if (selectedItemsRef.current[0].type == 2) {
                let edgeItem = edgesRef.current[(selectedItemsRef.current[0] as LineItem).id];
                if (edgeItem) {
                    edgeItem['components'] = JSON.parse(jsonString);
                    const edgesTemp = [...edgesRef.current];
                    edgesTemp[(selectedItemsRef.current[0] as LineItem).id] = edgeItem;
                    console.log(edgesTemp);
                    setEdges(edgesTemp);
                    ToastNotification('success', 'Vars set');
                }
            } else {
                let nodeItem = nodesRef.current.get((selectedItemsRef.current[0] as NodeItem).id);
                if (nodeItem) {
                    nodeItem['components'] = JSON.parse(jsonString);
                    setNodes(
                        map =>
                            new Map(map.set((selectedItemsRef.current[0] as NodeItem).id, nodeItem))
                    );
                    ToastNotification('success', 'Vars set');
                }
            }
        } catch (e) {
            ToastNotification('error', 'Not a valid json format');
        }
    };

    const OnClickPreset = async () => {
        setIsCreateIconModalOpen(true);
    };

    return (
        <>
            <div className='flex h-full grid grid-cols-12'>
                <div className='h-full col-span-8'>
                    <div style={{ height: '90%', width: '100%' }}>
                        <PredictionMapImport
                            twin={twinState.current}
                            nodes={nodes}
                            edges={edges}
                            onSelectBuilding={onSelectBuilding}
                        />
                    </div>
                    <div className='flex justify-start gap-2'>
                        <div className='bg-white gap-4 p-2 my-1 rounded-md flex justify-start max-w-100 overflow-x-auto'>
                            <Button
                                outline={!currentPreset}
                                onClick={(_: any) => changePreset(undefined)}
                                color='indigo'
                                theme={{
                                    color: {
                                        indigo: 'bg-indigo-600 text-white ring-indigo-600',
                                    },
                                }}
                            >
                                <span className='whitespace-nowrap text-xl font-semibold dark:text-white'>
                                    <Icon path={mdiCursorPointer} size={1} />
                                </span>
                            </Button>
                            <Button
                                color='indigo'
                                theme={{
                                    color: {
                                        indigo: 'bg-indigo-600 text-white ring-indigo-600',
                                    },
                                }}
                                //outline={cursor !== CursorState.PLACE_PRODUCER}
                                onClick={OnClickPreset}
                            >
                                <span className='whitespace-nowrap text-xl font-semibold dark:text-white'>
                                    <Icon path={mdiPlus} size={1.2} />
                                </span>
                            </Button>
                            {Array.from(presets.values()).map((preset, index) => (
                                <div key={index}>
                                    <Button
                                        outline={currentPreset?.name !== preset?.name}
                                        onClick={() => changePreset(preset)}
                                        color='indigo'
                                        theme={{
                                            color: {
                                                indigo: 'bg-indigo-600 text-white ring-indigo-600',
                                            },
                                        }}
                                    >
                                        {preset?.name}
                                    </Button>
                                </div>
                            ))}
                        </div>
                    </div>
                </div>
                <div className='col-span-4 mx-6 overflow-y-scroll'>
                    <div className='flex flex-col h-full'>
                        <div className='bg-white grid-cols-12 gap-4 my-1 rounded-md flex flex-col justify-start w-full p-3'>
                            {!selectedBuilding ? (
                                selectedItems.length != 1 && (
                                    <div className='text-gray-700 text-md mb-2'>
                                        Please select a building or edge.
                                    </div>
                                )
                            ) : (
                                <>
                                    <div
                                        className={`text-lg font-semibold mb-4 ${
                                            !selectedBuilding.visible ? 'blur-sm' : ''
                                        }`}
                                    >
                                        Selected Building Details:
                                    </div>
                                    <div
                                        className={`text-gray-700 text-md ${
                                            !selectedBuilding.visible ? 'blur-sm' : ''
                                        }`}
                                    >
                                        <div>
                                            <span className='font-semibold'>Building Number:</span>{' '}
                                            {selectedBuilding?.id}
                                        </div>
                                        <div>
                                            <span className='font-semibold'>City:</span>{' '}
                                            {selectedBuilding.city}
                                        </div>
                                        <div>
                                            <span className='font-semibold'>House Number:</span>{' '}
                                            {selectedBuilding.houseNumber}
                                        </div>
                                        <div>
                                            <span className='font-semibold'>Postcode:</span>{' '}
                                            {selectedBuilding.postcode}
                                        </div>
                                        <div>
                                            <span className='font-semibold'>Street:</span>{' '}
                                            {selectedBuilding.street}
                                        </div>
                                        <div>
                                            <div className='flex flex-wrap gap-2'>
                                                <span className='font-semibold'>Sensors:</span>
                                                {(() => {
                                                    const filteredSensors =
                                                        twinState.current.sensors.filter(
                                                            sensor =>
                                                                sensor.buildingId ===
                                                                selectedBuilding.id
                                                        );
                                                    return filteredSensors.map(buildingSensor => (
                                                        <Badge
                                                            key={buildingSensor.id}
                                                            color='gray'
                                                            style={{ cursor: 'pointer' }}
                                                            onClick={() =>
                                                                handleClick(buildingSensor)
                                                            }
                                                        >
                                                            {buildingSensor.name}
                                                        </Badge>
                                                    ));
                                                })()}
                                            </div>
                                        </div>
                                    </div>
                                    {selectedBuilding.visible ? (
                                        <Button
                                            color={'red'}
                                            onClick={() => handleDeleteBuilding()}
                                        >
                                            Delete building
                                        </Button>
                                    ) : (
                                        <Button
                                            color={'red'}
                                            onClick={() => handleUndoDeleteBuilding()}
                                        >
                                            Restore building
                                        </Button>
                                    )}
                                    <Button
                                        color='indigo'
                                        theme={{
                                            color: {
                                                indigo: 'bg-indigo-600 text-white ring-indigo-600',
                                            },
                                        }}
                                        onClick={() => {
                                            if (twinState.current) {
                                                const filteredSensors =
                                                    twinState.current.sensors.filter(
                                                        sensor =>
                                                            sensor.buildingId ===
                                                            selectedBuilding.id
                                                    );
                                                if (filteredSensors.length != 0) {
                                                    ToastNotification(
                                                        'warning',
                                                        'There can be only one sensor for each building.'
                                                    );
                                                } else {
                                                    setIsCreateSensorModalOpen(true);
                                                }
                                            } else {
                                                ToastNotification(
                                                    'error',
                                                    'Twin not selected. Try again.'
                                                );
                                            }
                                        }}
                                    >
                                        Create Sensor
                                    </Button>
                                </>
                            )}
                            {selectedItems.length == 1 && (
                                <>
                                    {selectedItems[0].type == 2 && (
                                        <div className={`text-lg font-semibold mb-4`}>
                                            Line Item {selectedItems[0].id}
                                        </div>
                                    )}
                                    <CustomJsonEditor
                                        data={JSON.parse(itemComponents)}
                                        onSave={updatedComponents => {
                                            saveBuildingComponents(
                                                JSON.stringify(updatedComponents)
                                            );
                                            setItemComponents(JSON.stringify(updatedComponents));
                                        }}
                                    />
                                </>
                            )}
                        </div>
                    </div>
                </div>
            </div>
            <CreateSensorModal
                isModalOpen={isCreateSensorModalOpen}
                selectedBuildingId={selectedBuilding?.id || null}
                handleCreateSensor={handleCreateSensor}
                closeModal={() => setIsCreateSensorModalOpen(false)}
            />
            <ShowSignalsModal
                isModalOpen={isShowSignalsModalOpen}
                sensor={selectedSensor}
                closeModal={() => {
                    setIsShowSignalsModalOpen(false);
                }}
            />
            <CreateIconsModal
                isModalOpen={isCreateIconsModalOpen}
                closeModal={() => {
                    setIsCreateIconModalOpen(false);
                    getAllPreset().then();
                }}
            ></CreateIconsModal>
        </>
    );
}

export default MapEditor;
