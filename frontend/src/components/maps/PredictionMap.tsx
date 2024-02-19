'use client';

import 'leaflet/dist/leaflet.css';
import { Twin, TwinContext } from '@/store/twins';
import {useContext, useId, useState} from 'react';
import { MapContainer, TileLayer, Marker, Popup, useMap, Polyline, useMapEvents, } from 'react-leaflet'
import { Icon as leafLetIcon, LatLngExpression } from 'leaflet'
import Icon from '@mdi/react';
import { Button } from 'flowbite-react';
import { mdiTransmissionTower, mdiCursorPointer, mdiHomeLightningBoltOutline, mdiWindTurbine} from '@mdi/js';

interface PredictionMapProps {
    twin: Twin;
};

interface MapItem {
    location: LatLngExpression;
    icon: leafLetIcon;
}

function PredictionMap({ twin }: PredictionMapProps) {
    const [currentMode, setCurrentMode] = useState(0);
    const [cursor, setCursor] = useState("grab");
    const [mapItems, setMapItems] = useState<Array<MapItem>>([]);

    function changeMode(newMode: number) {
        setCurrentMode(newMode);
        if(newMode == 0){
            setCursor("");
        }
        else if(newMode == 1){
            setCursor("/icons/transmission-tower.svg");
        }
        else if(newMode == 2){
            setCursor("/icons/home-lightning-bolt-outline.svg");
        }
        else if(newMode == 3){
            setCursor("/icons/wind-turbine.svg");
        }
    }

    //https://react-leaflet.js.org/docs/api-map/#usemapevents
    function MapEventTracker() {
        const map = useMapEvents({
            click: (e) => {
                console.log("test", e);
                addItemOnMap(e.latlng, currentMode);
            }
        })
        return null
    }

    /**
     * Sets an item on the map
     * @param location location of item
     * @param itemType witch item
     */
    function addItemOnMap(location: LatLngExpression, itemType: number){
        if(itemType == 0)
            return
        let newItem: MapItem = {
            location: location,
            icon: new leafLetIcon({
                iconUrl: cursor,
                iconSize: [38, 45], // size of the icon
            })
        }
        setMapItems( // Replace the state
            [ // with a new array
                ...mapItems, // that contains all the old items
                newItem
            ]
        );
    }

    return (
        <div style={{ width: '100%', height: '100%',  cursor: cursor == "" ? "grab": "url('" + cursor + "'), crosshair"}}>
            <MapContainer style={{ width: '100%', height: '90%', cursor: "inherit"}} className="rounded-md" center={[twin.city.latitde, twin.city.longitude]} zoom={13} scrollWheelZoom={false}>
                <MapEventTracker>
                </MapEventTracker>
                <TileLayer
                    attribution='&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
                    url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
                />
                {mapItems.map(item => (
                    <Marker position={item.location} key={1} icon={item.icon}>
                        <Button>
                            <span className="whitespace-nowrap text-xl font-semibold dark:text-whit"><Icon path={mdiWindTurbine} size={1.2} /></span>
                        </Button>
                    </Marker>
                ))}
                <Polyline key={1} positions={[[51.2289271, 4.4022454],[51.2242539, 4.4139502], [51.2165844, 4.4171321]]} color={"red"} eventHandlers={{
                    click: () => {console.log("mas", "kathedraal", "station")}
                }} />
            </MapContainer>

            <div className="bg-white grid-cols-12 gap-4 p-2 my-1 rounded-md flex justify-start">
                <Button onClick={e=>changeMode(0)}>
                    <span className="whitespace-nowrap text-xl font-semibold dark:text-whit"><Icon path={mdiCursorPointer} size={1} /></span>
                </Button>
                <Button onClick={e=>changeMode(1)}>
                    <span className="whitespace-nowrap text-xl font-semibold dark:text-whit"><Icon path={mdiTransmissionTower} size={1.2} /></span>
                </Button>
                <Button onClick={e=>changeMode(2)}>
                    <span className="whitespace-nowrap text-xl font-semibold dark:text-whit"><Icon path={mdiHomeLightningBoltOutline } size={1.2} /></span>
                </Button>
                <Button>
                    <span className="whitespace-nowrap text-xl font-semibold dark:text-whit"><Icon path={mdiWindTurbine} size={1.2} /></span>
                </Button>
            </div>
        </div>
    );
}

export default PredictionMap;

