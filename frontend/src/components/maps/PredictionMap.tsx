'use client';

import 'leaflet/dist/leaflet.css';
import { Twin, TwinContext } from '@/store/twins';
import { useContext, useId, useState } from 'react';
import { MapContainer, TileLayer, Marker, Popup, useMap, Polyline, useMapEvents, SVGOverlay, } from 'react-leaflet'
import { Icon as leafLetIcon, LatLngExpression, LatLng } from 'leaflet'
import { Button } from 'flowbite-react';
import { mdiTransmissionTower, mdiCursorPointer, mdiHomeLightningBoltOutline, mdiWindTurbine } from '@mdi/js';
import { Icon } from '@mdi/react';

interface PredictionMapProps {
    twin: Twin;
};

interface MapItem {
    location: LatLngExpression;
    iconPath: string;
}

enum CursorState {
    NONE,
    GRAB,
    PLACE_TOWER,
    PLACE_BOLT,
    PLACE_TURBINE
}

const iconPaths = {
    [CursorState.PLACE_BOLT]: "/icons/home-lightning-bolt-outline.svg",
    [CursorState.PLACE_TOWER]: "/icons/transmission-tower.svg",
    [CursorState.PLACE_TURBINE]: "/icons/wind-turbine.svg",
    [CursorState.GRAB]: "hand",
    [CursorState.NONE]: "hand"
}

function PredictionMap({ twin }: PredictionMapProps) {
    const [cursor, setCursor] = useState<CursorState>(CursorState.GRAB);
    const [mapItems, setMapItems] = useState<Array<MapItem>>([]);

    // https://react-leaflet.js.org/docs/api-map/#usemapevents
    const addMapItem = (latlng: LatLng) => {
        if (cursor === CursorState.NONE || cursor === CursorState.GRAB) {
            return;
        }

        const newItem: MapItem = {
            location: latlng,
            iconPath: iconPaths[cursor]
        }

        setMapItems(mapItems.concat([newItem]));
    }

    const MapClickHandler = ({ addMapItem }: { addMapItem: (latlng: LatLng) => void }) => {
        useMapEvents({
            click: (e) => addMapItem(e.latlng)
        });

        return <></>
    }

    return (
        <div style={{ width: '100%', height: '100%', cursor: `url(${iconPaths[cursor]}), crosshair` }}>
            <MapContainer style={{ width: '100%', height: '90%', cursor: "inherit" }} className="rounded-md" center={[twin.city.latitde, twin.city.longitude]} zoom={13} scrollWheelZoom={false}>
                <MapClickHandler addMapItem={addMapItem} />
                <TileLayer
                    attribution='&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
                    url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
                />
                {
                    mapItems.map((item: MapItem, i) => <Marker position={item.location} key={i} icon={new leafLetIcon({ iconUrl: item.iconPath, iconSize: [38, 45] })} />)
                }
            </MapContainer>
            <div className="bg-white grid-cols-12 gap-4 p-2 my-1 rounded-md flex justify-start">
                <Button outline={cursor !== CursorState.GRAB} onClick={e => setCursor(CursorState.GRAB)}>
                    <span className="whitespace-nowrap text-xl font-semibold dark:text-whit"><Icon path={mdiCursorPointer} size={1} /></span>
                </Button>
                <Button outline={cursor !== CursorState.PLACE_TOWER} onClick={e => setCursor(CursorState.PLACE_TOWER)}>
                    <span className="whitespace-nowrap text-xl font-semibold dark:text-whit"><Icon path={mdiTransmissionTower} size={1.2} /></span>
                </Button>
                <Button outline={cursor !== CursorState.PLACE_BOLT} onClick={e => setCursor(CursorState.PLACE_BOLT)}>
                    <span className="whitespace-nowrap text-xl font-semibold dark:text-whit"><Icon path={mdiHomeLightningBoltOutline} size={1.2} /></span>
                </Button>
                <Button outline={cursor !== CursorState.PLACE_TURBINE} onClick={e => setCursor(CursorState.PLACE_TURBINE)}>
                    <span className="whitespace-nowrap text-xl font-semibold dark:text-whit"><Icon path={mdiWindTurbine} size={1.2} /></span>
                </Button>
            </div>
        </div>
    );
}

export default PredictionMap;

