'use client';
import 'leaflet/dist/leaflet.css';
import { TwinFromProvider } from '@/store/twins';
import { TwinContext } from '@/store/twins';
import React, { useContext, useEffect, useRef, useState } from 'react';
import {
    MapContainer,
    Marker,
    Polygon,
    Polyline,
    Tooltip,
    TileLayer,
    useMap,
    useMapEvents,
} from 'react-leaflet';
import { mdiCheck, mdiClose, mdiAlert } from '@mdi/js';
import { Icon as leafLetIcon, LatLngExpression, LeafletEventHandlerFnMap } from 'leaflet';
import {
    BuildingItem,
    LineItem,
    MapItems,
    MapItemType,
    NodeItem,
    iconPaths,
} from '@/components/maps/MapItem';
import ToastNotification from '../notification/ToastNotification';
import { createChannel, createClient } from 'nice-grpc-web';
import { uiBackendServiceUrl } from '@/api/urls';
import { buildingObject, TwinServiceDefinition } from '@/proto/twins/twin';
import Icon from '@mdi/react';
import { s } from 'hastscript';
import '@/css/leaflet.css';

export interface PredictionMapProps {
    twin: TwinFromProvider;
    eventHandlers?: LeafletEventHandlerFnMap;
    nodes?: Map<number, NodeItem>;
    edges?: LineItem[];
    onSelectBuilding?: (building: BuildingItem) => void;
}
enum status {
    ok = 1,
    warning,
    error,
}

interface componentStatus {
    name: string;
    status: status;
    value: number;
    compareValue: number;
    compareSymbol: string;
}

interface ComponentStateTooltipProps {
    warningComp: any[];
}

/**
 * Tooltip used when hovered over item
 * @param warningComp all components to show
 * @constructor
 */
function ComponentStateTooltip({ warningComp }: ComponentStateTooltipProps) {
    return (
        warningComp.length > 0 && (
            <Tooltip>
                {warningComp.map((item, index) => (
                    <div key={item.status} className={'flex p-0 h-5 m-0'}>
                        <p className={'pr-2'}>
                            {item.status == status.ok ? (
                                <Icon path={mdiCheck} color='green' size={0.8} />
                            ) : item.status == status.warning ? (
                                <Icon path={mdiAlert} color='orange' size={0.8} />
                            ) : (
                                <Icon path={mdiClose} color='red' size={0.8} />
                            )}
                        </p>
                        <p className={'pr-2'}>{item.name}</p>
                        <p>{item.value}</p>
                        <p> {' ' + item.compareSymbol + ' '} </p>
                        <p>{item.compareValue}</p>
                    </div>
                ))}
            </Tooltip>
        )
    );
}

export function PredictionMap({
    twin,
    eventHandlers = {},
    nodes = new Map(),
    edges = [],
    onSelectBuilding = undefined,
}: PredictionMapProps) {
    const [twinState, dispatch] = useContext(TwinContext);
    const [cityName, setCityName] = useState<string>(twin.name);
    const [buildings, setBuildings] = useState<Array<BuildingItem>>([]);
    const buildingsRef = useRef(buildings); //Use a reference because needed when called from eventHandlers

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
            throw new Error('No coordinates provided');
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
        const fetchBuildings = async () => {
            try {
                if (twinState.current) {
                    ToastNotification('success', 'Your twin is being loaded.');
                    const channel = createChannel(uiBackendServiceUrl);
                    const client = createClient(TwinServiceDefinition, channel);
                    const request = { id: twinState.current.id };

                    const response = await client.getBuildings(request);
                    //convert buildings to mapItems
                    if (!response) {
                        return;
                    }
                    let buildings: Array<BuildingItem> = response?.buildings.map(
                        (building: buildingObject, index) => {
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
                                    click: e => selectBuilding(index),
                                },
                                location: center,
                            };
                            return item;
                        }
                    );
                    setBuildings(buildings);
                }
            } catch (error) {
                console.error('Failed to fetch buildings:', error);
            }
        };
        let _ = fetchBuildings();

        // eslint-disable-next-line
    }, [twinState]);

    function selectBuilding(index: number) {
        const updatedMapItems = buildingsRef.current.map((item, idx) => {
            const buildingItem = { ...item } as BuildingItem;
            //if item is selected => always red
            if (idx === index) {
                buildingItem.selected = true;
            }
            //if building is not selected and is visible => blue
            else {
                buildingItem.selected = false;
            }
            return buildingItem;
        });
        setBuildings(updatedMapItems);
        if (onSelectBuilding) onSelectBuilding(updatedMapItems[index] as BuildingItem);
    }

    const MapClickHandler = () => {
        useMapEvents(eventHandlers);
        return <></>;
    };

    const ChangeLocation = () => {
        const map = useMap();
        useEffect(() => {
            if (cityName != twin.name) {
                map.setView([twin.latitude, twin.longitude]);
                setCityName(twin.name);
            }
        }, [map]);
        return null;
    };

    /**
     * Calculate color for border and components using the _max values
     * @param component
     */
    function calculateWarningColorAndComponents(component: any): {
        components: Array<componentStatus>;
        color: string;
    } {
        let innerColor = 'green';
        let components: Array<componentStatus> = [];
        Object.entries(component || {}).map((item: any, key: any) => {
            Object.entries(item[1] || {}).map((innerItem: any, innerKey: any) => {
                //Check for max values
                if (item[1]['max_' + innerItem[0]] !== undefined) {
                    let currentValue = item[1][innerItem[0]];
                    let maxValue = item[1]['max_' + innerItem[0]];
                    let usagePercentage = currentValue / maxValue;
                    let compStatus = status.ok;
                    if (usagePercentage > 1) {
                        innerColor = 'red';
                        compStatus = status.error;
                    } else if (usagePercentage > 0.95) {
                        compStatus = status.warning;
                        if (innerColor == 'green') innerColor = 'orange';
                    }
                    components.push({
                        name: innerItem[0],
                        status: compStatus,
                        value: currentValue,
                        compareValue: maxValue,
                        compareSymbol: '≤',
                    });
                }

                //Check for min values
                if (item[1]['min_' + innerItem[0]] !== undefined) {
                    let currentValue = item[1][innerItem[0]];
                    let minValue = item[1]['min_' + innerItem[0]];
                    let usagePercentage = minValue / currentValue;

                    let compStatus = status.ok;
                    if (usagePercentage > 1) {
                        innerColor = 'red';
                        compStatus = status.error;
                    } else if (usagePercentage > 0.95) {
                        compStatus = status.warning;
                        if (innerColor == 'green') innerColor = 'orange';
                    }
                    components.push({
                        name: innerItem[0],
                        status: compStatus,
                        value: currentValue,
                        compareValue: minValue,
                        compareSymbol: '≥',
                    });
                }

                //Check for equal values
                if (item[1]['eq_' + innerItem[0]] !== undefined) {
                    let currentValue = item[1][innerItem[0]];
                    let eqValue = item[1]['eq_' + innerItem[0]];

                    let compStatus = status.ok;
                    if (currentValue != eqValue) {
                        innerColor = 'red';
                        compStatus = status.error;
                    }
                    components.push({
                        name: innerItem[0],
                        status: compStatus,
                        value: currentValue,
                        compareValue: eqValue,
                        compareSymbol: '=',
                    });
                }
            });
        });
        return { components, color: innerColor };
    }

    return (
        <>
            <MapContainer
                style={{
                    width: '100%',
                    height: '100%',
                    cursor: 'inherit',
                    transition: 'filter 0.3s ease',
                    zIndex: 1,
                }}
                className='rounded-md'
                center={[twin.latitude, twin.longitude]}
                zoom={16}
            >
                <MapClickHandler />
                <TileLayer
                    attribution='&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
                    url='https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png'
                />
                {nodes &&
                    false &&
                    Array.from(nodes.values()).map((item, i) => (
                        <Marker
                            key={item.id}
                            eventHandlers={item.eventHandler}
                            position={item.location}
                            icon={
                                new leafLetIcon({
                                    iconUrl: '/icons/home-lightning-bolt-outline.svg',
                                    iconSize: [30, 30],
                                })
                            }
                        />
                    ))}
                {edges &&
                    edges.map(edge => {
                        let positions: Array<LatLngExpression> = [];
                        edge.items.forEach(marker => {
                            if (!marker) return;
                            if (!marker.inactive) {
                                positions.push(marker.location);
                            }
                        });
                        const colorsAndWarning = calculateWarningColorAndComponents(
                            edge.components
                        );
                        let warningComp = colorsAndWarning.components;
                        let lineColor = colorsAndWarning.color;
                        return (
                            <Polyline
                                key={edge.id + Math.random()}
                                eventHandlers={edge.eventHandler}
                                color={lineColor}
                                positions={positions}
                            >
                                <ComponentStateTooltip
                                    warningComp={warningComp}
                                ></ComponentStateTooltip>
                            </Polyline>
                        );
                    })}
                {buildings &&
                    buildings.map((item, i) => {
                        let lineColor = '#808080';
                        let innerColor = '#808080';
                        let warningComp: Array<componentStatus> = [];
                        let nodeItem = nodes?.get(item.id);
                        if (nodeItem) {
                            const colorsAndWarning = calculateWarningColorAndComponents(
                                nodeItem.components
                            );
                            lineColor = colorsAndWarning.color;
                            warningComp = colorsAndWarning.components;
                            if (nodeItem.components?.hasOwnProperty('energy_producer_node')) {
                                innerColor = 'yellow';
                            } else if (
                                nodeItem.components?.hasOwnProperty('energy_consumer_node')
                            ) {
                                innerColor = 'blue';
                            } else {
                                innerColor = '#ffa200';
                            }
                        } else if (!item.visible) {
                            innerColor = 'black';
                        }

                        return (
                            <Polygon
                                positions={(item as BuildingItem).coordinates.map(coordinate => [
                                    coordinate[0],
                                    coordinate[1],
                                ])}
                                key={item.id + 'S' + String(item.selected) + 'N' + nodeItem}
                                fillOpacity={item.selected ? 0.8 : 0.4}
                                opacity={item.selected ? 0.8 : 0.8}
                                pathOptions={{ color: lineColor, fillColor: innerColor }}
                                eventHandlers={item.eventHandler}
                            >
                                <ComponentStateTooltip
                                    warningComp={warningComp}
                                ></ComponentStateTooltip>
                            </Polygon>
                        );
                    })}
                <ChangeLocation />
            </MapContainer>
        </>
    );
}

export default PredictionMap;
