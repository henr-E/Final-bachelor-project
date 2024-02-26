'use client';
import 'leaflet/dist/leaflet.css';
import { Twin } from '@/store/twins';
import { useEffect,useState} from 'react';
import { MapContainer, TileLayer, useMap, useMapEvents} from 'react-leaflet'
import {LeafletEventHandlerFnMap} from 'leaflet'
import {MapItem, MapItemType} from "@/components/maps/MapItem";

export interface PredictionMapProps {
    twin: Twin;
    eventHandlers: LeafletEventHandlerFnMap;
    mapItems?: MapItemType[];
};

export function PredictionMap({ twin, eventHandlers = {}, mapItems = []}: PredictionMapProps) {
    const [cityName, setCityName] = useState<string>(twin.city.name);

    const MapClickHandler = () => {
        useMapEvents(eventHandlers);
        return <></>
    }

    const ChangeLocation = () => {
        const map = useMap();
        useEffect(() => {
            if(cityName != twin.city.name) {
                map.setView([twin.city.latitde, twin.city.longitude]);
                setCityName(twin.city.name);
            }
        }, [twin]);
        return null;
    }

    return (
        <MapContainer style={{ width: '100%', height: '100%', cursor: "inherit" }} className="rounded-md" center={[twin.city.latitde, twin.city.longitude]} zoom={13}>
            <MapClickHandler/>
            <TileLayer
                attribution='&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
                url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
            />
            {
                mapItems?.map((item: MapItemType, i) => <MapItem itemData={item} key={i}/>)
            }
            <ChangeLocation />
        </MapContainer>
    );
}

export default PredictionMap;

