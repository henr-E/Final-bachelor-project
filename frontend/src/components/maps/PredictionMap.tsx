'use client';
import 'leaflet/dist/leaflet.css';
import {TwinFromProvider} from '@/store/twins';
import {TwinContext} from '@/store/twins';
import React, {useContext, useEffect, useRef, useState} from 'react';
import {MapContainer, Marker, Polygon, Polyline, TileLayer, useMap, useMapEvents} from 'react-leaflet'
import {Icon as leafLetIcon, LatLngExpression, LeafletEventHandlerFnMap} from 'leaflet'
import {BuildingItem, LineItem, MapItems, MapItemType, NodeItem, iconPaths} from "@/components/maps/MapItem";
import ToastNotification from '../notification/ToastNotification';
import {createChannel, createClient} from "nice-grpc-web";
import {uiBackendServiceUrl} from "@/api/urls";
import {buildingObject, TwinServiceDefinition} from "@/proto/twins/twin";


export interface PredictionMapProps {
    twin: TwinFromProvider;
    eventHandlers?: LeafletEventHandlerFnMap;
    nodes?: Map<number, NodeItem>;
    edges?: LineItem[];
    onSelectBuilding?: (building: BuildingItem) => void;
}

export function PredictionMap({twin, eventHandlers = {}, nodes = new Map(), edges = [], onSelectBuilding = undefined}: PredictionMapProps) {
    const [twinState, dispatch] = useContext(TwinContext);
    const [cityName, setCityName] = useState<string>(twin.name);
    const [buildings, setBuildings] = useState<Array<BuildingItem>>([]);
    const buildingsRef = useRef(buildings);//Use a reference because needed when called from eventHandlers

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
        buildingsRef.current = buildings;

    }, [buildings]);

    useEffect(() => {
        console.log("edges", edges)

    }, [edges]);


    useEffect(() => {
        const fetchBuildings = async () => {
            try {
                if (twinState.current) {
                    ToastNotification("success", "Your twin is being loaded.");
                    const channel = createChannel(uiBackendServiceUrl);
                    const client = createClient(TwinServiceDefinition, channel);
                    const request = {id: twinState.current.id};

                    const response = await client.getBuildings(request);
                    //convert buildings to mapItems
                    if (!response) {
                        return;
                    }
                    let buildings: Array<BuildingItem> = response?.buildings.map((building: buildingObject, index) => {
                        const center = calculateCenterPoint(building);
                        const item: BuildingItem = {
                            id: building.id,
                            city: building.city,
                            coordinates: building.coordinates,
                            houseNumber: building.houseNumber,
                            postcode: building.postcode,
                            street: building.street,
                            visible: building.visible,
                            selected: false,
                            eventHandler: {
                                click: (e) => selectBuilding(index)
                            },
                            location: center
                        }
                        return item;
                    });
                    setBuildings(buildings);
                }
            } catch (error) {
                console.error("Failed to fetch buildings:", error);
            }
        }
        let _ = fetchBuildings();

        // eslint-disable-next-line
    }, [twinState]);

    function selectBuilding(index: number) {
        const updatedMapItems = buildingsRef.current.map((item, idx) => {
            const buildingItem = {...item} as BuildingItem;
            //if item is selected => always red
            if (idx === index) {
                buildingItem.selected = true
            }
            //if building is not selected and is visible => blue
            else {
                buildingItem.selected = false
            }
            return buildingItem;
        });
        setBuildings(updatedMapItems);
        if(onSelectBuilding)
            onSelectBuilding(updatedMapItems[index] as BuildingItem);
    }

    const MapClickHandler = () => {
        useMapEvents(eventHandlers);
        return <></>
    }

    const ChangeLocation = () => {
        const map = useMap();
        useEffect(() => {
            if (cityName != twin.name) {
                map.setView([twin.latitude, twin.longitude]);
                setCityName(twin.name);
            }
        }, [map]);
        return null;
    }

    return (
        <>
            <MapContainer
                style={{
                    width: '100%',
                    height: '100%',
                    cursor: "inherit",
                    transition: "filter 0.3s ease",
                    zIndex: 1
                }}
                className="rounded-md"

                center={[twin.latitude, twin.longitude]} zoom={16}>
                <MapClickHandler/>
                <TileLayer
                    attribution='&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
                    url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
                />
                {nodes && false && Array.from(nodes.values()).map((item, i) =>
                    <Marker
                        key={item.id}
                        eventHandlers={item.eventHandler}
                        position={item.location}
                        icon={
                            new leafLetIcon({
                                iconUrl: "/icons/home-lightning-bolt-outline.svg",
                                iconSize: [30, 30],
                            })
                        }
                    />)
                }
                {edges && edges.map((edge) =>
                    {
                        let positions: Array<LatLngExpression> = [];
                        edge.items.forEach(marker => {
                            if(!marker)
                                return
                            if (!marker.inactive) {
                                positions.push(marker.location);
                            }
                        });
                        return (
                            <Polyline
                                key={edge.id + Math.random()}
                                eventHandlers={edge.eventHandler}
                                positions={positions}
                            />
                        );
                    }

                )}
                {buildings && buildings.map((item, i) =>
                    {
                        let color = '#808080';
                        let nodeItem = nodes?.get(item.id);
                        if (nodeItem) {
                            if(nodeItem.components?.hasOwnProperty("energy_producer_node")){
                                color = "yellow";
                            }
                            else if(nodeItem.components?.hasOwnProperty("energy_consumer_node")){
                                color = "blue";
                            }
                            else {
                                color = "#ffa200";
                            }
                        }
                        else if(!item.visible){
                            color = 'red';
                        }

                        return (<Polygon
                            positions={(item as BuildingItem).coordinates.map(coordinate => [coordinate[0], coordinate[1]])}
                            key={item.id + "S" + String(item.selected) + "N" + nodeItem}
                            fillOpacity={item.selected? 0.8: 0.2}
                            opacity={item.selected? 0.8: 0.5}
                            color={color}
                            eventHandlers={item.eventHandler}
                        />)
                    })
                }
                <ChangeLocation/>
            </MapContainer>
        </>
    )
        ;
}

export default PredictionMap;
