'use client';
import 'leaflet/dist/leaflet.css';
import {Twin} from '@/store/twins';
import React, {useEffect, useState} from 'react';
import {MapContainer, TileLayer, useMap, useMapEvents} from 'react-leaflet'
import {LeafletEventHandlerFnMap} from 'leaflet'
import {MapItem, MapItemType} from "@/components/maps/MapItem";


export interface PredictionMapProps {
    twin: Twin;
    eventHandlers?: LeafletEventHandlerFnMap;
    mapItems?: MapItemType[];
}

export function PredictionMap({twin, eventHandlers = {}, mapItems = []}: PredictionMapProps) {
    const [cityName, setCityName] = useState<string>(twin.name);

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
                {mapItems && mapItems.map((item, i) => <MapItem itemData={item} key={i + Math.random()}/>)}
                <ChangeLocation/>
            </MapContainer>
        </>
    )
        ;
}

export default PredictionMap;
