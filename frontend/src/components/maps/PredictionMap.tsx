'use client';

import 'leaflet/dist/leaflet.css';
import { Twin, TwinContext } from '@/store/twins';
import { useContext } from 'react';
import { MapContainer, TileLayer, Marker, Popup, useMap } from 'react-leaflet'

interface PredictionMapProps {
    twin: Twin;
};

function PredictionMap({ twin }: PredictionMapProps) {
    return (
        <MapContainer style={{ width: '100%', height: '100%' }} center={[twin.city.latitde, twin.city.longitude]} zoom={13} scrollWheelZoom={false}>
            <TileLayer
                attribution='&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
                url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
            />
            <Marker position={[51.505, -0.09]}>
                <Popup>
                </Popup>
            </Marker>
        </MapContainer>
    );
}

export default PredictionMap;

