'use client';
import 'leaflet/dist/leaflet.css';
import {LatLngExpression, LeafletEventHandlerFnMap} from 'leaflet'

export enum MapItems {
    TransformerHouse,
    Turbine,
    Line,
    Building,
    None
}

export const iconPaths = {
    [MapItems.TransformerHouse]: "/icons/home-lightning-bolt-outline.svg",
    [MapItems.Turbine]: "/icons/wind-turbine.svg",
    [MapItems.Line]: "/icons/transit-connection-horizontal.svg",
    [MapItems.Building]: "",
}

export interface MapItemType {
    name: string;
    id: number;
    eventHandler?: LeafletEventHandlerFnMap;
    type: MapItems;
    inactive?: boolean;
    components?: { [id: string]: any };
}

export interface NodeItem extends MapItemType {
    location: LatLngExpression;
}

export interface LineItem extends MapItemType{
    items: Array<NodeItem>;
}

export interface BuildingItem {
    id: number;
    street: string;
    houseNumber: string;
    postcode: string;
    city: string;
    coordinates: number[][];
    visible: boolean;
    eventHandler?: LeafletEventHandlerFnMap;
    location: LatLngExpression;
    selected: boolean;
    nodeItem?: NodeItem;
}
