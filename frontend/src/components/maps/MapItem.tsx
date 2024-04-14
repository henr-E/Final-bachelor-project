'use client';
import 'leaflet/dist/leaflet.css';
import {Marker, Polygon, Polyline,} from 'react-leaflet'
import {Icon as leafLetIcon, LatLngExpression, LeafletEventHandlerFnMap} from 'leaflet'

export enum MapItems {
    TransformerHouse,
    Tower,
    Turbine,
    Line,
    Building
}

const iconPaths = {
    [MapItems.TransformerHouse]: "/icons/home-lightning-bolt-outline.svg",
    [MapItems.Tower]: "/icons/transmission-tower.svg",
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

export interface LineItem extends MapItemType {
    items: Array<NodeItem>;
}

export interface BuildingItem extends NodeItem {
    id: number;
    street: string;
    houseNumber: string;
    postcode: string;
    city: string;
    coordinates: number[][];
    color: string;
    visible: boolean;
}

export function MapItem(mapItem: any) {
    if (mapItem.itemData.inactive) {
        return null;
    }

    if (mapItem.itemData.type === MapItems.Line) {
        let lineItem = mapItem.itemData as LineItem;
        let positions: Array<LatLngExpression> = [];
        lineItem.items.forEach(marker => {
            if (!marker)
                return
            if (!marker.inactive) {
                positions.push(marker.location);
            }
        });

        return (
            <Polyline
                eventHandlers={mapItem.itemData.eventHandler}
                positions={positions}
            />
        );
    } else if (mapItem.itemData.type === MapItems.Building) {
        let buildingItem = mapItem.itemData as BuildingItem;
        return (
            <Polygon
                key={buildingItem.id}
                positions={buildingItem.coordinates.map(coordinate => [coordinate[0], coordinate[1]])}
                color={buildingItem.color}
                eventHandlers={buildingItem.eventHandler}
            />
        )

    }

    let item = mapItem.itemData as NodeItem;
    return (
        <Marker
            eventHandlers={item.eventHandler}
            position={item.location}
            icon={
                new leafLetIcon({
                    iconUrl: iconPaths[item.type],
                    iconSize: [30, 30],
                })
            }
        />
    );
}

export default MapItem;

