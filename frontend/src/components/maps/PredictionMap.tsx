'use client';
import 'leaflet/dist/leaflet.css';
import {Twin} from '@/store/twins';
import React, {useEffect, useState} from 'react';
import {MapContainer, Polyline, Polygon, Popup, TileLayer, useMap, useMapEvents} from 'react-leaflet'
import {LatLngExpression, LeafletEventHandlerFnMap} from 'leaflet'
import {MapItem, MapItemType} from "@/components/maps/MapItem";
import BuildingsBackendRequest from "@/components/maps/BuildingsBackendRequest";
import Building from "@/components/maps/BuildingsBackendRequest";
//https://mhnpd.github.io/react-loader-spinner/docs/components/grid
import {Grid} from 'react-loader-spinner';
import Image from "next/image";
import loadingGif from "../../../public/loader/loading.gif";
import ToastNotification from "@/components/notification/ToastNotification";
import {PredictionMapMode} from "@/app/dashboard/GlobalVariables";

interface BuildingFeature {
    type: string;
    geometry: {
        type: string;
        coordinates: number[][][];
    };
    properties: { [key: string]: any };
}

export interface PredictionMapProps {
    twin: Twin;
    eventHandlers?: LeafletEventHandlerFnMap;
    mapItems?: MapItemType[];
    mode: PredictionMapMode;
}

export function PredictionMap({twin, eventHandlers = {}, mapItems = [], mode}: PredictionMapProps) {
    const [cityName, setCityName] = useState<string>(twin.name);
    const [buildings, setBuildings] = useState<BuildingFeature[]>([]);
    const [buildingsLoading, setBuildingsLoading] = useState<boolean>(false);

    useEffect(() => {
        const fetchBuildings = async () => {
            try {
                setBuildingsLoading(true);
                ToastNotification("info", "Your twin is being loaded.");
                const data = await BuildingsBackendRequest(twin.id);
                if (data.features) {
                    setBuildings(data.features);
                }
            } catch (error) {
                console.error("Failed to fetch buildings:", error);
            }
            setBuildingsLoading(false);
        };
        fetchBuildings();
    }, [twin.id]);

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
            {buildingsLoading && (
                <div style={{
                    display: 'flex',
                    flexDirection: 'column',
                    justifyContent: 'center',
                    alignItems: 'center',
                    position: 'absolute',
                    top: 0,
                    left: 0,
                    width: '100%',
                    height: '100%',
                    zIndex: 1000,
                }}>
                    <div style={{textAlign: 'center', marginBottom: '10px'}}>
                        <Grid
                            height="80"
                            width="80"
                            color="#5750E3"
                            ariaLabel="grid-loading"
                            radius="12.5"
                            visible={true}
                        />
                    </div>
                    <div style={{textAlign: 'center'}}>
                        <h1>Loading your twin ...</h1>
                    </div>
                </div>
            )}
            <MapContainer
                key={buildingsLoading ? 'loading' : 'loaded'}
                style={{
                    width: '100%',
                    height: '100%',
                    cursor: "inherit",
                    filter: buildingsLoading ? "blur(2px)" : "none",
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
                {mapItems && mapItems.map((item, i) => <MapItem itemData={item} key={i}/>)}
                {buildings.map((building, index) => (
                    <Polygon
                        key={index}
                        positions={building.geometry.coordinates[0].map(coordinate => [coordinate[1], coordinate[0]])}
                        color="red"
                        interactive={mode === PredictionMapMode.RealtimeMode || mode === PredictionMapMode.ForeCastingMode}
                    >
                        <Popup>
                            <div>Street: {building.properties?.['addr:street'] ?? "not available"}</div>
                            <div>Housenumber: {building.properties?.['addr:housenumber'] ?? "not available"}</div>
                        </Popup>
                    </Polygon>
                ))}
                <ChangeLocation/>
            </MapContainer>
        </>
    );
}

export default PredictionMap;
