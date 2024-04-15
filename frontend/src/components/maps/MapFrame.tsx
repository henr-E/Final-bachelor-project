'use client';
import {LeafletEventHandlerFnMap} from "leaflet";
import {Button, Card, Spinner} from "flowbite-react";
import {useContext, useEffect, useRef, useState, Ref, MutableRefObject} from "react";
import {PredictionMapProps} from "@/components/maps/PredictionMap"
import dynamic from "next/dynamic";
import {TwinContext, TwinFromProvider} from "@/store/twins";
import {BuildingItem, LineItem, MapItems, MapItemType, NodeItem} from "@/components/maps/MapItem";
import ToastNotification from "@/components/notification/ToastNotification";
import {JsonToTable} from "react-json-to-table";
import {SimulationFrame} from "@/proto/simulation/simulation-manager";

export interface MapFrameProps {
    twin: TwinFromProvider;
    eventHandlers?: LeafletEventHandlerFnMap;
    frame?: SimulationFrame;
    frameNr: number;
    onSelectBuilding?: (building: BuildingItem) => void;
}

const PredictionMapImport = dynamic<PredictionMapProps>(() => import("@/components/maps/PredictionMap"), {ssr: false});

/**
 * Converts a Simulation frame to a node map and an edge array
 * @param frame
 * @param showNode
 * @param showEdge
 */
export function FrameToMapInformation(frame: SimulationFrame, showNode: (id: number) => void = (id) => {}, showEdge: (id: number) => void = (id) => {}) : [Map<number, NodeItem> | undefined, LineItem[] | undefined] {
    let nodeItemArray = frame?.state?.graph?.nodes.map((item) => {
        const newItem: NodeItem = {
            components: item.components,
            inactive: false,
            location: [item.latitude, item.longitude],
            name: item.id.toString(),
            id: item.id,
            type: MapItems.TransformerHouse,
            eventHandler: {click: (e) => showNode(item.id)}
        }
        return newItem;
    });

    if (!nodeItemArray)
        return [undefined, undefined]

    let nodeItems = new Map(nodeItemArray.map(i => [i.id, i]));

    let lineItemArray = frame?.state?.graph?.edge.map(item => {
        let connectingItems = [nodeItems.get(item.from) as NodeItem, nodeItems.get(item.to) as NodeItem];
        const newItem: LineItem = {
            name: item.componentType,
            id: item.id,
            components: item.componentData,
            items: connectingItems,
            type: MapItems.Line,
            eventHandler: {click: (e) => showEdge(item.id)}
        }
        return newItem;
    });
    console.log(lineItemArray);

    if (!lineItemArray)
        return [nodeItems, undefined]

    return [nodeItems, lineItemArray]

}

function MapEditor({frame, frameNr}: MapFrameProps) {
    const [twinState, dispatch] = useContext(TwinContext);
    const [nodes, setNodes] = useState<Map<number, NodeItem>>(new Map());
    const [edges, setEdges] = useState<LineItem[]>([]);
    const [clickedItem, setClickedItem] = useState<MapItemType | undefined>(undefined);
    const nodesRef = useRef(nodes);//Use a reference because needed when called from eventHandlers
    const edgesRef = useRef(edges);//Use a reference because needed when called from eventHandlers

    useEffect(() => {
        nodesRef.current = nodes;
        edgesRef.current = edges;
    }, [nodes, edges]);

    /**
     * When frame changes,
     * Convert the frame into nodes and edges to be displayed onto the map
     */
    useEffect(() => {
        if(!frame)
            return
        let mapItems = FrameToMapInformation(frame, showNode, showEdge);

        if (!mapItems[0])
            return
        setNodes(mapItems[0]);

        if (!mapItems[1])
            return
        setEdges(mapItems[1]);
    }, [frame]);

    if (!twinState.current) {
        return <h1>Please select a Twin</h1>
    }

    const showNode = (id: number) => {
        setClickedItem(nodesRef.current.get(id));
    }

    const showEdge = (id: number) => {
        setClickedItem(edgesRef.current[id]);
    }

    const onSelectBuilding = (building: BuildingItem) => {
        setClickedItem(nodesRef.current.get(building.id));
    }

    return (
        <div className="h-full grid grid-cols-12">
            <div className={`h-full col-span-9 ${!frame ? 'blur-sm' : ''}`}>
                <PredictionMapImport twin={twinState.current} nodes={nodes}
                                     onSelectBuilding={onSelectBuilding}
                                     edges={edges}/>
            </div>
            <div className="col-span-3 mx-6">
                {frame ?
                    <div className="h-full space-y-1">
                        <Card className="overflow-x-scroll">
                            <h1>Global variable</h1>
                            {frame.state?.globalComponents &&
                                <JsonToTable json={frame.state?.globalComponents}/>}
                        </Card>
                        <Card className="overflow-x-scroll">
                            <h1>Vars of selected component</h1>
                            {frame && (clickedItem)?.components &&
                                <JsonToTable
                                    json={clickedItem?.components}/>}
                        </Card>
                    </div> :
                    <div className="col-span-3 mx-6">
                        <Card className="text-center items-center">
                            <Spinner aria-label="Medium sized spinner example" size="md"/>
                            {frame ?
                                <div><h3 className="text-lg">Not loaded into buffer</h3><h4>leave timeline to load
                                    frame: {frameNr}</h4></div> : <h1>Loading frame</h1>}
                        </Card>
                    </div>
                }
            </div>
        </div>
)
    ;
}

export default MapEditor;
