'use client';
import 'leaflet/dist/leaflet.css';
import { Twin, TwinContext } from '@/store/twins';
import {useContext, useEffect, useId, useState} from 'react';
import { MapContainer, TileLayer, Marker, Popup, useMap, Polyline, useMapEvents, SVGOverlay, } from 'react-leaflet'
import {Icon as leafLetIcon, LatLngExpression, LatLng, LeafletEventHandlerFnMap} from 'leaflet'
import { Button } from 'flowbite-react';
import { mdiTransmissionTower, mdiCursorPointer, mdiHomeLightningBoltOutline, mdiWindTurbine } from '@mdi/js';
import { Icon } from '@mdi/react';
import mapItem from "@/components/maps/MapItem";
export enum MapItems {
    TransformerHouse,
    Tower,
    Turbine,
    Line
}

const iconPaths = {
    [MapItems.TransformerHouse]: "/icons/home-lightning-bolt-outline.svg",
    [MapItems.Tower]: "/icons/transmission-tower.svg",
    [MapItems.Turbine]: "/icons/wind-turbine.svg",
    [MapItems.Line]: "/icons/transit-connection-horizontal.svg",
}

export interface MapItemType {
    name: string;
    eventHandler?: LeafletEventHandlerFnMap;
    type: MapItems
    inactive?: boolean
}

export interface MarkerItem extends MapItemType{
    location: LatLngExpression;
}

export interface LineItem extends MapItemType{
    items: [MarkerItem];
}

export function MapItem( mapItem: any) {
    if(mapItem.itemData.inactive){
        return null;
    }

    if(mapItem.itemData.type === MapItems.Line){
        let lineItem = mapItem.itemData as LineItem;
        let positions: Array<LatLngExpression> = [];
        lineItem.items.forEach((marker) => {
            if(!marker.inactive) {
                positions.push(marker.location);
            }
        });

        return (
            <Polyline
                eventHandlers={mapItem.itemData.eventHandler}
                positions={positions}
            />
        );
    }

    let item = mapItem.itemData as MarkerItem;
    return (
        <Marker
            eventHandlers={item.eventHandler}
            position={item.location}
            icon={new leafLetIcon({
                iconUrl: iconPaths[item.type],
                iconSize: [30, 30]
            })}
        />
    );
}

export default MapItem;
