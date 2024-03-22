'use client';
import {LatLng, LatLngExpression, LeafletEventHandlerFnMap} from "leaflet";
import {Button} from "flowbite-react";
import {Icon} from "@mdi/react";
import {
    mdiCursorMove,
    mdiCursorPointer,
    mdiHomeLightningBoltOutline,
    mdiTransitConnectionHorizontal,
    mdiTransmissionTower,
    mdiWindTurbine
} from "@mdi/js";
import {useContext, useEffect, useRef, useState, Ref, MutableRefObject} from "react";
import {PredictionMapProps} from "@/components/maps/PredictionMap"
import dynamic from "next/dynamic";
import {Twin, TwinContext} from "@/store/twins";
import {BuildingItem, LineItem, MapItems, MapItemType, NodeItem} from "@/components/maps/MapItem";
import ToastNotification from "@/components/notification/ToastNotification";
import {buildingObject, TwinServiceDefinition} from '@/proto/twins/twin';
import {createChannel, createClient} from "nice-grpc-web";
import {uiBackendServiceUrl} from "@/api/urls";

enum CursorState {
    NONE,
    GRAB,
    PLACE_TOWER,
    PLACE_BOLT,
    PLACE_TURBINE,
    CONNECT_ITEMS,
    MOVE_ITEMS
}

const iconPaths = {
    [CursorState.PLACE_BOLT]: "/icons/home-lightning-bolt-outline.svg",
    [CursorState.PLACE_TOWER]: "/icons/transmission-tower.svg",
    [CursorState.PLACE_TURBINE]: "/icons/wind-turbine.svg",
    [CursorState.CONNECT_ITEMS]: "/icons/transit-connection-horizontal.svg",
    [CursorState.MOVE_ITEMS]: "/icons/cursor-move.svg",
    [CursorState.GRAB]: "hand",
    [CursorState.NONE]: "hand"
}

const cursorToType = {
    [CursorState.PLACE_BOLT]: MapItems.TransformerHouse,
    [CursorState.PLACE_TOWER]: MapItems.Tower,
    [CursorState.PLACE_TURBINE]: MapItems.Turbine,
    [CursorState.CONNECT_ITEMS]: MapItems.Line,
}

export interface MapEditorProps {
    mapItemRef?: MutableRefObject<MapItemType[] | undefined>
};

const PredictionMapImport = dynamic<PredictionMapProps>(() => import("@/components/maps/PredictionMap"), {ssr: false});

function MapEditor({mapItemRef}: MapEditorProps) {
    const [twinState, dispatch] = useContext(TwinContext);
    const [cursor, setCursor] = useState<CursorState>(CursorState.GRAB);
    const [mapItems, setMapItems] = useState<Array<MapItemType>>([]);
    const [selectedItems, setSelectedItems] = useState<Array<number>>([]);
    const mapItemsRef = useRef(mapItems);//Use a reference because needed when called from eventHandlers
    const cursorRef = useRef(cursor);//Use a reference because needed when called from eventHandlers
    const selectedItemsRef = useRef(selectedItems);//Use a reference because needed when called from eventHandlers

    const [selectedBuildingIndex, setSelectedBuildingIndex] = useState<number>(-1);
    const [selectedBuilding, setSelectedBuilding] = useState<BuildingItem>();
    const [selectedBuildingVisible, setSelectedBuildingVisible] = useState<boolean | null>(null);

    function calculateCenterPoint(building: buildingObject): LatLngExpression {
        let totalX = 0;
        let totalY = 0;
        let totalCount = 0;

        for (var coord of building.coordinates) {
            totalX += coord[0]; // Add latitude
            totalY += coord[1]; // Add longitude
            totalCount++;
        }

        if (totalCount === 0) {
            throw new Error("No coordinates provided");
        }

        // Calculate the average for each
        const centerX = totalX / totalCount;
        const centerY = totalY / totalCount;

        return [centerX, centerY] as LatLngExpression;
    }

    //Update the reference when state changes
    useEffect(() => {
        mapItemsRef.current = mapItems;
        cursorRef.current = cursor;
        selectedItemsRef.current = selectedItems;
        if (mapItemRef)
            mapItemRef.current = mapItems;

    }, [mapItems, cursor, selectedItems, mapItemRef]);


    useEffect(() => {
        const fetchBuildings = async () => {
            try {
                if (twinState.current) {
                    ToastNotification("info", "Your twin is being loaded.");
                    const channel = createChannel(uiBackendServiceUrl);
                    const client = createClient(TwinServiceDefinition, channel);
                    const request = {id: twinState.current.id};

                    const response = await client.getBuildings(request);
                    //convert buildings to mapItems
                    if (!response) {
                        return;
                    }
                    let buildings: Array<MapItemType> = response?.buildings.map((building: buildingObject, index) => {
                        const center = calculateCenterPoint(building);
                        const item: BuildingItem = {
                            id: building.id,
                            city: building.city,
                            coordinates: building.coordinates,
                            houseNumber: building.houseNumber,
                            name: "",
                            postcode: building.postcode,
                            street: building.street,
                            type: MapItems.Building,
                            color: building.visible ? 'blue' : '#808080',
                            visible: building.visible,
                            eventHandler: {
                                click: (e) => selectItemMap(index)
                            },
                            location: center
                        }
                        return item;
                    });
                    const items = mapItems.concat(buildings);
                    setMapItems(items);
                }
            } catch (error) {
                console.error("Failed to fetch buildings:", error);
            }
        }
        let _ = fetchBuildings();
        // eslint-disable-next-line
    }, [twinState]);

    useEffect(() => {
        setSelectedBuilding(mapItemsRef.current[selectedBuildingIndex] as BuildingItem)
    }, [selectedBuildingIndex]);

    useEffect(() => {
        const building = mapItemsRef.current[selectedBuildingIndex] as BuildingItem;
        setSelectedBuildingVisible(building?.visible); // Update visibility state
    }, [selectedBuildingIndex]);

    if (!twinState.current) {
        return <h1>Please select a Twin</h1>
    }

    function updateBuildingColor(index: number) {
        const updatedMapItems = mapItemsRef.current.map((item, idx) => {
            if (item.type === MapItems.Building) {
                const buildingItem = {...item} as BuildingItem;
                //if item is selected => always red
                if (idx === index) {
                    buildingItem.color = 'red'
                }
                //if item is not visible => grey
                else if (!buildingItem.visible) {
                    buildingItem.color = '#808080'
                }
                //if building is not selected and is visible => blue
                else {
                    buildingItem.color = 'blue'
                }
                return buildingItem;
            }
            return item;
        });
        setMapItems(updatedMapItems);
        setSelectedBuildingIndex(index);
    }

    /**
     * Select an item on the map by id
     * @param index
     */
    function selectItemMap(index: number) {
        if (mapItemsRef.current[index] && mapItemsRef.current[index].type === MapItems.Building) {
            updateBuildingColor(index);
        }
        if (cursorRef.current === CursorState.CONNECT_ITEMS) {
            addLine(index);
            setSelectedItems([...selectedItemsRef.current, index]);
            return;
        } else if (cursorRef.current === CursorState.MOVE_ITEMS) {
            moveMapItem(index);
        }
        setSelectedItems([index]);
    }

    /**
     * Add a line between items
     * @param id the id of the item you want the line to pass
     */
    const addLine = (id: number) => {
        if (selectedItemsRef.current.length == 0) {
            const newItem: LineItem = {
                name: "item: " + mapItemsRef.current.length.toString(),
                id: mapItemsRef.current.length,
                items: [mapItemsRef.current[id] as NodeItem],
                type: MapItems.Line,
                eventHandler: {click: (e) => console.log("item clicked")}
            }

            setMapItems([...mapItemsRef.current, newItem]);
            return;
        }
        changeCursor(CursorState.GRAB);
        (mapItemsRef.current[mapItemsRef.current.length - 1] as LineItem).items.push(mapItemsRef.current[id] as NodeItem);
    }

    const changeCursor = (cursor: CursorState) => {
        setSelectedItems([]);
        setCursor(cursor);
    }

    /**
     * Add a new item to the map
     * @param latlng location of item
     */
    const addMapItem = (latlng: LatLng) => {
        if (cursor === CursorState.NONE || cursor === CursorState.GRAB || cursor === CursorState.CONNECT_ITEMS) {
            return;
        } else if (cursor === CursorState.MOVE_ITEMS) {
            moveMapItem(selectedItemsRef.current[0], latlng);
            return;
        }

        const newItem: NodeItem = {
            id: mapItemsRef.current.length,
            name: "item: " + mapItems.length.toString(),
            location: latlng,
            type: cursorToType[cursor],
            eventHandler: {
                click: (e) => selectItemMap(mapItems.length),
                contextmenu: (e) => removeMapItem(mapItems.length)
            }
        }

        setMapItems([...mapItems, newItem]);
    }

    /**
     * Remove an item from the map
     * (currently set item to inactive)
     * @param id index id
     */
    const removeMapItem = (id: number) => {
        let tempMapItems = [...mapItemsRef.current];
        tempMapItems[id].inactive = true; //currently set item to inactive TODO
        setMapItems(tempMapItems);
    }

    /**
     * Move item
     * @param id item to move
     * @param newPos new position (optional) needed by second click
     */
    const moveMapItem = (id: number, newPos?: LatLng) => {
        if ((selectedItemsRef.current.length === 0 || newPos === undefined) && id !== undefined) {
            let tempMapItems = [...mapItemsRef.current];
            tempMapItems[id].inactive = true; //set item to inactive temporary
            setMapItems(tempMapItems);
            return;
        } else if (selectedItemsRef.current.length === 1) {
            let tempMapItems = [...mapItemsRef.current];
            tempMapItems[id].inactive = false; //set item to inactive temporary
            if ((tempMapItems[id] as NodeItem) == undefined || newPos == undefined) {
                return;
            }
            (tempMapItems[id] as NodeItem).location = newPos;
            setMapItems(tempMapItems);
        }
    }

    const eventHandlers = {
        click: (e) => {
            addMapItem(e.latlng);
        }
    } as LeafletEventHandlerFnMap;

    const handleDeleteBuilding = async () => {
        const channel = createChannel(uiBackendServiceUrl);
        const client = createClient(TwinServiceDefinition, channel);
        const request = {id: selectedBuilding?.id};

        const response = await client.deleteBuilding(request);
        if (response.deleted) {
            setSelectedBuildingVisible(false);
            (mapItemsRef.current[selectedBuildingIndex] as BuildingItem).visible = false;
            ToastNotification("info", "Building succesfully deleted.")
            updateBuildingColor(selectedBuildingIndex);
        }
    }

    const handleUndoDeleteBuilding = async () => {
        const channel = createChannel(uiBackendServiceUrl);
        const client = createClient(TwinServiceDefinition, channel);
        const request = {id: selectedBuilding?.id};

        const response = await client.undoDeleteBuilding(request);
        if (response.undone) {
            setSelectedBuildingVisible(true);
            (mapItemsRef.current[selectedBuildingIndex] as BuildingItem).visible = true;
            ToastNotification("info", "Building succesfully restored.")
            updateBuildingColor(selectedBuildingIndex);
        }
    }


    return (
        <div className="flex h-full grid grid-cols-12">
            <div className="h-full col-span-9" style={{ cursor: `url(${iconPaths[cursor]}) 15 15, crosshair` }} >
                <div style={{height:"90%"}}>
                    <PredictionMapImport twin={twinState.current} eventHandlers={eventHandlers} mapItems={mapItems}/>
                </div>
                <div className="flex justify-start gap-2">
                    <div className="bg-white gap-4 p-2 my-1 rounded-md flex justify-start">
                        <Button outline={cursor !== CursorState.GRAB} onClick={(_: any) => changeCursor(CursorState.GRAB)}>
                            <span className="whitespace-nowrap text-xl font-semibold dark:text-white"><Icon path={mdiCursorPointer} size={1} /></span>
                        </Button>
                        <Button outline={cursor !== CursorState.PLACE_TOWER} onClick={(_: any) => changeCursor(CursorState.PLACE_TOWER)}>
                            <span className="whitespace-nowrap text-xl font-semibold dark:text-white"><Icon path={mdiTransmissionTower} size={1.2} /></span>
                        </Button>
                        <Button outline={cursor !== CursorState.PLACE_BOLT} onClick={(_: any) => changeCursor(CursorState.PLACE_BOLT)}>
                            <span className="whitespace-nowrap text-xl font-semibold dark:text-white"><Icon path={mdiHomeLightningBoltOutline} size={1.2} /></span>
                        </Button>
                        <Button outline={cursor !== CursorState.PLACE_TURBINE} onClick={(_: any) => changeCursor(CursorState.PLACE_TURBINE)}>
                            <span className="whitespace-nowrap text-xl font-semibold dark:text-white"><Icon path={mdiWindTurbine} size={1.2} /></span>
                        </Button>
                    </div>
                    <div className="bg-white grid-cols-12 gap-4 p-2 my-1 rounded-md flex">
                        <Button outline={cursor !== CursorState.CONNECT_ITEMS} onClick={(_: any) => changeCursor(CursorState.CONNECT_ITEMS)}>
                            <span className="whitespace-nowrap text-xl font-semibold dark:text-white"><Icon path={mdiTransitConnectionHorizontal} size={1.2} /></span>
                        </Button>
                        <Button outline={cursor !== CursorState.MOVE_ITEMS} onClick={(_: any) => changeCursor(CursorState.MOVE_ITEMS)}>
                            <span className="whitespace-nowrap text-xl font-semibold dark:text-white"><Icon path={mdiCursorMove } size={1.2} /></span>
                        </Button>
                    </div>
                </div>
            </div>
            <div className="col-span-3 mx-6">
                <div className="flex flex-col h-full">
                    <div
                        className="bg-white grid-cols-12 gap-4 my-1 rounded-md flex flex-col justify-start w-full p-3">
                        {selectedBuildingIndex === -1 ? (
                            <div className="text-gray-700 text-md mb-2">Please select a building.</div>
                        ) : (
                            <>
                                <div
                                    className={`text-lg font-semibold mb-4 ${!selectedBuildingVisible ? 'blur-sm' : ''}`}>
                                    Selected Building Details:
                                </div>
                                <div className={`text-gray-700 text-md ${!selectedBuildingVisible ? 'blur-sm' : ''}`}>
                                    <div><span className="font-semibold">Building Number:</span> {selectedBuilding?.id}
                                    </div>
                                    <div><span className="font-semibold">City:</span> {selectedBuilding?.city}</div>
                                    <div><span
                                        className="font-semibold">House Number:</span> {selectedBuilding?.houseNumber}
                                    </div>
                                    <div><span className="font-semibold">Postcode:</span> {selectedBuilding?.postcode}
                                    </div>
                                    <div><span className="font-semibold">Street:</span> {selectedBuilding?.street}</div>
                                </div>
                                {selectedBuildingVisible ? (
                                    <Button
                                        color={"red"}
                                        onClick={() => handleDeleteBuilding()}
                                    >Delete building</Button>
                                ) : (
                                    <Button
                                        color={"red"}
                                        onClick={() => handleUndoDeleteBuilding()}
                                    >Restore building</Button>
                                )}
                            </>
                        )}
                    </div>
                </div>
            </div>
        </div>
    );
}

export default MapEditor;



