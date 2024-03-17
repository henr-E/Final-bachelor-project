'use client';
import {LatLng, LeafletEventHandlerFnMap} from "leaflet";
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
import {LineItem, MapItems, MapItemType, MarkerItem} from "@/components/maps/MapItem";
import {PredictionMapMode} from "@/app/dashboard/GlobalVariables";

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

const PredictionMapImport = dynamic<PredictionMapProps>(() => import("@/components/maps/PredictionMap"), { ssr: false });

export function MapEditor({ mapItemRef }: MapEditorProps)  {
    const [twinState, dispatch] = useContext(TwinContext);
    const [cursor, setCursor] = useState<CursorState>(CursorState.GRAB);
    const [mapItems, setMapItems] = useState<Array<MapItemType>>([]);
    const [selectedItems, setSelectedItems] = useState<Array<number>>([]);
    const mapItemsRef = useRef(mapItems);//Use a reference because needed when called from eventHandlers
    const cursorRef = useRef(cursor);//Use a reference because needed when called from eventHandlers
    const selectedItemsRef = useRef(selectedItems);//Use a reference because needed when called from eventHandlers

    //Update the reference when state changes
    useEffect(() => {
        mapItemsRef.current = mapItems;
        cursorRef.current = cursor;
        selectedItemsRef.current = selectedItems;
        if(mapItemRef)
            mapItemRef.current = mapItems;

    }, [mapItems, cursor, selectedItems, mapItemRef]);


    if (!twinState.current) {
        return <h1>Please select a Twin</h1>
    }

    /**
     * Select an item on the map by id
     * @param id index of the selectedItems array
     */
    function selectItemMap (id: number) {
        if(cursorRef.current === CursorState.CONNECT_ITEMS){
            addLine(id);
            setSelectedItems([...selectedItemsRef.current, id]);
            return;
        }
        else if(cursorRef.current === CursorState.MOVE_ITEMS){
            moveMapItem(id);
        }
        setSelectedItems([id]);
    }

    /**
     * Add a line between items
     * @param id the id of the item you want the line to pass
     */
    const addLine =  (id: number) => {
        if (selectedItemsRef.current.length == 0){
            const newItem: LineItem = {
                name: "item: " + mapItemsRef.current.length.toString(),
                id:  mapItemsRef.current.length,
                items: [mapItemsRef.current[id] as MarkerItem],
                type: MapItems.Line,
                eventHandler: { click: (e) => console.log("item clicked")}}

            setMapItems([...mapItemsRef.current, newItem]);
            return;
        }
        (mapItemsRef.current[mapItemsRef.current.length - 1] as LineItem).items.push(mapItemsRef.current[id] as MarkerItem);
        changeCursor(CursorState.GRAB);
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
        }
        else if(cursor === CursorState.MOVE_ITEMS){
            moveMapItem(selectedItemsRef.current[0] ,latlng);
            return;
        }

        const newItem: MarkerItem = {
            name: "item: " + mapItemsRef.current.length.toString(),
            id: mapItemsRef.current.length,
            location: latlng,
            type: cursorToType[cursor],
            eventHandler: { click: (e) => selectItemMap(mapItems.length), contextmenu: (e) => removeMapItem(mapItems.length)}}

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
        if((selectedItemsRef.current.length === 0 || newPos === undefined) && id !== undefined){
            let tempMapItems = [...mapItemsRef.current];
            tempMapItems[id].inactive = true; //set item to inactive temporary
            setMapItems(tempMapItems);
            return;
        }
        else if(selectedItemsRef.current.length === 1){
            console.log("set item");
            let tempMapItems = [...mapItemsRef.current];
            tempMapItems[id].inactive = false; //set item to inactive temporary
            if((tempMapItems[id] as MarkerItem) == undefined || newPos == undefined){
                return;
            }
            (tempMapItems[id] as MarkerItem).location = newPos;
            setMapItems(tempMapItems);
        }
    }

    const eventHandlers = {
        click: (e) => {addMapItem(e.latlng);}
    } as LeafletEventHandlerFnMap;


        return (
            <div className="flex h-full grid grid-cols-12">
                <div className="h-full col-span-9" style={{ cursor: `url(${iconPaths[cursor]}) 15 15, crosshair` }} >
                    <div style={{height:"90%"}}>
                        <PredictionMapImport twin={twinState.current} eventHandlers={eventHandlers} mapItems={mapItems} mode={PredictionMapMode.EditorMode}/>
                    </div>
                    <div className="flex justify-start gap-2">
                        <div className="bg-white gap-4 p-2 my-1 rounded-md flex justify-start">
                            <Button outline={cursor !== CursorState.GRAB} onClick={e => changeCursor(CursorState.GRAB)}>
                                <span className="whitespace-nowrap text-xl font-semibold dark:text-white"><Icon path={mdiCursorPointer} size={1} /></span>
                            </Button>
                            <Button outline={cursor !== CursorState.PLACE_TOWER} onClick={e => changeCursor(CursorState.PLACE_TOWER)}>
                                <span className="whitespace-nowrap text-xl font-semibold dark:text-white"><Icon path={mdiTransmissionTower} size={1.2} /></span>
                            </Button>
                            <Button outline={cursor !== CursorState.PLACE_BOLT} onClick={e => changeCursor(CursorState.PLACE_BOLT)}>
                                <span className="whitespace-nowrap text-xl font-semibold dark:text-white"><Icon path={mdiHomeLightningBoltOutline} size={1.2} /></span>
                            </Button>
                            <Button outline={cursor !== CursorState.PLACE_TURBINE} onClick={e => changeCursor(CursorState.PLACE_TURBINE)}>
                                <span className="whitespace-nowrap text-xl font-semibold dark:text-white"><Icon path={mdiWindTurbine} size={1.2} /></span>
                            </Button>
                        </div>
                        <div className="bg-white grid-cols-12 gap-4 p-2 my-1 rounded-md flex">
                            <Button outline={cursor !== CursorState.CONNECT_ITEMS} onClick={e => changeCursor(CursorState.CONNECT_ITEMS)}>
                                <span className="whitespace-nowrap text-xl font-semibold dark:text-white"><Icon path={mdiTransitConnectionHorizontal} size={1.2} /></span>
                            </Button>
                            <Button outline={cursor !== CursorState.MOVE_ITEMS} onClick={e => changeCursor(CursorState.MOVE_ITEMS)}>
                                <span className="whitespace-nowrap text-xl font-semibold dark:text-white"><Icon path={mdiCursorMove } size={1.2} /></span>
                            </Button>
                        </div>
                    </div>
                </div>
                <div className="col-span-3 mx-6">
                    <div className="flex flex-col h-full">
                        <div className="bg-white grid-cols-12 gap-4 my-1 rounded-md flex justify-start w-full p-3">
                            {
                                (selectedItems.length == 0)?
                                    <h1>Select item</h1>:
                                    selectedItems.map((item: number, index: number) => <h1 key={index}>{mapItems[item].name}</h1>)
                            }
                        </div>
                    </div>

                </div>
            </div>
        );

}

export default MapEditor;

