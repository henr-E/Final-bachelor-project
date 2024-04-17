'use client';
import React from 'react';
import { MapContainer, Marker, Rectangle, TileLayer, useMap } from 'react-leaflet';
import { LatLngBoundsExpression } from 'leaflet';

export interface CustomMapContainerProps {
    position: [number, number];
    bounds: LatLngBoundsExpression;
}

function ChangeView({ center, zoom }: { center: [number, number]; zoom: number }) {
    const map = useMap();
    map.setView(center, zoom);
    return null;
}

const CustomMapContainer: React.FC<CustomMapContainerProps> = ({ position, bounds }) => {
    return (
        <MapContainer center={position} zoom={14} style={{ height: '100%', width: '100%' }}>
            <TileLayer url='https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png' />
            <Marker position={position} />
            <Rectangle bounds={bounds} />
            <ChangeView center={position} zoom={14} />
        </MapContainer>
    );
};

export default CustomMapContainer;
