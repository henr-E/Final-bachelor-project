'use client';
import React, { useContext, useEffect, useState } from 'react';
import { MapContainer, Marker, Rectangle, TileLayer, useMap } from 'react-leaflet';
import 'leaflet/dist/leaflet.css';
import { Button, Modal, TextInput } from 'flowbite-react';
import { TwinContext } from '@/store/twins';
import L, { LatLngBoundsExpression } from 'leaflet';
import Image from 'next/image';
import ToastNotification from '@/components/notification/ToastNotification';
import { BackendCreateTwin } from '@/api/twins/crud';
import loadingGif from '@/../public/loader/loading.gif';
import { TourControlContext } from '@/store/tour';

/**
 * The icon to put on the map
 */
let defaultIcon = L.icon({
    iconUrl: 'https://unpkg.com/leaflet@1.7.1/dist/images/marker-icon.png',
    shadowUrl: 'https://unpkg.com/leaflet@1.7.1/dist/images/marker-shadow.png',
    iconSize: [25, 41],
    iconAnchor: [12, 41],
    popupAnchor: [1, -34],
    shadowSize: [41, 41],
});
L.Marker.prototype.options.icon = defaultIcon;

/**
 * convert the city or place to coordinates
 * @param place
 */
async function geocodePlace(
    place: string
): Promise<{ lat: number; lng: number; displayname: string } | null> {
    const response = await fetch(
        `https://nominatim.openstreetmap.org/search?format=json&q=${encodeURIComponent(place)}`
    );
    const data = await response.json();
    if (data.length > 0) {
        return {
            lat: Number(data[0].lat),
            lng: Number(data[0].lon),
            displayname: data[0].display_name,
        };
    } else {
        return null;
    }
}

/**
 * convert the coordinates to a city or place
 * @param coords
 */
async function reverseGeocode(coords: [number, number]): Promise<{ displayname: string } | null> {
    const response = await fetch(
        `https://nominatim.openstreetmap.org/reverse?format=json&lat=${coords[0]}&lon=${coords[1]}`
    );
    const data = await response.json();
    if (data && data.address) {
        return {
            displayname: data.display_name,
        };
    } else {
        return null;
    }
}

/**
 * change the view of the map
 * @param center
 * @param zoom
 * @constructor
 */
function ChangeView({ center, zoom }: { center: [number, number]; zoom: number }) {
    const map = useMap();
    map.setView(center, zoom);
    return null;
}

export interface CreateTwinModalProps {
    isCreateTwinModalOpen: boolean;
    closeCreateTwinModal: () => void;
}

/**
 * createTwinModal gives the inputs that are needed to create a new twin
 * @param isCreateTwinModalOpen
 * @param closeCreateTwinModal
 * @constructor
 */

function CreateTwinModal({ isCreateTwinModalOpen, closeCreateTwinModal }: CreateTwinModalProps) {
    const tourController = useContext(TourControlContext);
    const [twinState, dispatchTwin] = useContext(TwinContext);
    const [place, setPlace] = useState<string>('');
    const [coords, setCoords] = useState<string>('');
    const [position, setPosition] = useState<[number, number]>([51.505, -0.09]);
    const [mapRadius, setMapRadius] = useState<number>(400);
    const [radiusInput, setRadiusInput] = useState<string>('400');
    const [isConfirmModalOpen, setIsConfirmModalOpen] = useState(false);
    const [customName, setCustomName] = useState<string>('');
    const [createTwinLoading, setCreateTwinLoading] = useState<boolean>(false);
    const [bounds, setBounds] = useState<LatLngBoundsExpression>([
        [0, 0],
        [0, 0],
    ]);
    const handleSearch = async (e: React.FormEvent) => {
        e.preventDefault();
        // Search by place if the place input is not empty and coords is empty
        if (place !== '' && coords === '') {
            const result = await geocodePlace(place);
            if (result) {
                setPosition([result.lat, result.lng]);
                setCoords(`${result.lat}, ${result.lng}`);
                setPlace(result.displayname);
                const beforeComma = result.displayname.split(',')[0];
                setCustomName(beforeComma);
                ToastNotification('success', 'Succesfully found location');
            } else {
                ToastNotification('warning', 'Place not found. Please try another search.');
            }
        }
        // Search by coordinates if the coords input is not empty and place is empty
        else if (coords !== '' && place === '') {
            const [lat, lng] = coords.split(',').map(Number);
            if (!isNaN(lat) && !isNaN(lng)) {
                const result = await reverseGeocode([lat, lng]);
                if (result) {
                    setPosition([lat, lng]);
                    setPlace(result.displayname);
                    const beforeComma = result.displayname.split(',')[0];
                    setCustomName(beforeComma);
                    ToastNotification('success', 'Succesfully found location');
                } else {
                    ToastNotification(
                        'warning',
                        'Coordinates not found. Please try another input.'
                    );
                }
            } else {
                ToastNotification(
                    'warning',
                    'Invalid coordinates. Please enter in the format "lat, lng".'
                );
            }
        }
    };

    /**
     * when the focus of the mouse moves away from the radius input,
     * check if the value is right
     */
    const handleRadiusBlur = () => {
        const radiusValue = Number(radiusInput);
        if (radiusValue < 10 || isNaN(radiusValue)) {
            ToastNotification('warning', 'The provided radius is invalid. Using default 400.');
            setMapRadius(10);
            setRadiusInput('10');
        } else if (radiusValue > 1000) {
            ToastNotification('warning', 'The provided radius is invalid. Using default 1000.');
            setMapRadius(1000);
            setRadiusInput('1000');
        } else {
            setMapRadius(radiusValue);
        }
    };

    /**
     * open the second modal that asks for confirmation
     */
    const openConfirmModal = () => {
        let twinExists = false;
        if (place !== '' && coords !== '') {
            //check if the city already exists
            for (let i = 0; i < twinState.twins.length; i++) {
                if (
                    twinState.twins[i].latitude == position[0] &&
                    twinState.twins[i].longitude == position[1]
                ) {
                    twinExists = true;
                    ToastNotification('warning', 'Hmm, looks like this twin already exists.');
                }
                if (twinState.twins[i].name == customName) {
                    twinExists = true;
                    ToastNotification(
                        'warning',
                        'Hmm, looks like a twin with this name already exists'
                    );
                }
            }
            if (!twinExists) {
                setIsConfirmModalOpen(true);
            }
        } else {
            ToastNotification('warning', 'Oops, you forgot to fill in the place or coordinates.');
        }
    };

    /**
     * backend request when modals are closed
     */
    const closeModalsAndCreateTwin = async () => {
        setCreateTwinLoading(true);

        const latitude = position[0];
        const longitude = position[1];

        const response = await BackendCreateTwin(customName, latitude, longitude, mapRadius);
        if (response) {
            let twin = {
                id: response.id,
                name: customName,
                longitude: longitude,
                latitude: latitude,
                radius: mapRadius,
                sensors: [],
                simulations: [],
                creation_date_time: response.creationDateTime,
                simulation_amount: 0,
            };
            dispatchTwin({ type: 'create_twin', twin: twin });
            ToastNotification('success', 'The twin was created successfully.');

            setIsConfirmModalOpen(false);
            closeCreateTwinModal();

            resetAll();
        }
        tourController?.customGoToNextTourStep(1);
        setCreateTwinLoading(false);
    };

    const resetAll = () => {
        //reset everything
        setPlace('');
        setCoords('');
        setPosition([51.505, -0.09]);
        setMapRadius(10);
        setRadiusInput('10');
        setCustomName('');
    };

    /**
     * when the confirmation modal is closed, all variables should be reset and all modals closed
     */
    const handleBack = () => {
        setIsConfirmModalOpen(false); // Return to the first modal
    };

    useEffect(() => {
        let radius_km = mapRadius / 1000.0;
        // Calculate degrees of latitude per kilometer
        let delta_lat = radius_km / 111.0;
        // Calculate degrees of longitude per kilometer at the given latitude
        // Make sure to specify the type for numerical literals and perform operations on the correct type
        let pi = Math.PI;
        let latitude_in_radians = position[0] * (pi / 180);
        let delta_lon = radius_km / (111.0 * Math.cos(latitude_in_radians));

        // Calculate bounds for the rectangle
        const bounds: LatLngBoundsExpression = [
            [position[0] - delta_lat, position[1] - delta_lon], // Southwest corner
            [position[0] + delta_lat, position[1] + delta_lon], // Northeast corner
        ];

        setBounds(bounds);
    }, [mapRadius, position]);

    return (
        <>
            {createTwinLoading && (
                <>
                    <div
                        style={{
                            display: 'flex',
                            justifyContent: 'center',
                            alignItems: 'center',
                            position: 'absolute',
                            top: 0,
                            left: 0,
                            width: '100%',
                            height: '100%',
                            zIndex: 3000,
                        }}
                    >
                        {/* Additional container div for image and text */}
                        <div style={{ textAlign: 'center' }}>
                            <Image
                                src={loadingGif} // Corrected path
                                alt='Loading...'
                                style={{ marginBottom: '10px' }} // Adjust as needed
                            />
                            <h1>Creating your twin ...</h1>
                        </div>
                    </div>
                </>
            )}
            <>
                <Modal
                    show={isCreateTwinModalOpen && !isConfirmModalOpen}
                    onClose={() => {
                        resetAll();
                        closeCreateTwinModal();
                    }}
                    style={{
                        maxWidth: '100%',
                        maxHeight: '100%',
                        zIndex: 2000,
                    }}
                >
                    <Modal.Header>Create Twin</Modal.Header>
                    <Modal.Body>
                        <div style={{ display: 'flex', gap: '20px' }}>
                            {/* Left Side - MapContainer */}
                            <div style={{ flex: 4, height: '400px' }}>
                                <MapContainer
                                    center={position}
                                    zoom={14}
                                    style={{ height: '100%', width: '100%' }}
                                >
                                    <TileLayer url='https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png' />
                                    <Marker position={position}></Marker>
                                    <Rectangle bounds={bounds} />
                                    <ChangeView center={position} zoom={14} />
                                </MapContainer>
                            </div>

                            {/* Right Side - Form Inputs */}
                            <div
                                style={{
                                    flex: 2,
                                    display: 'flex',
                                    flexDirection: 'column',
                                    gap: '10px',
                                }}
                            >
                                <div className={'tour-step-2-overview'}>
                                    <div>
                                        Enter a city, place or coordinates. Press Enter or Search.
                                        <TextInput
                                            type='text'
                                            value={place}
                                            placeholder='City or place'
                                            onChange={e => {
                                                setPlace(e.target.value);
                                                if (coords) setCoords('');
                                            }}
                                            onKeyDown={e => {
                                                if (e.key === 'Enter') handleSearch(e);
                                            }}
                                        />
                                    </div>
                                    <div>
                                        <TextInput
                                            type='text'
                                            value={coords}
                                            placeholder='Coordinates (lat, lng)'
                                            onChange={e => {
                                                setCoords(e.target.value);
                                                if (place) setPlace('');
                                            }}
                                            onKeyDown={e => {
                                                if (e.key === 'Enter') handleSearch(e);
                                            }}
                                        />
                                    </div>
                                </div>
                                <Button
                                    className={'tour-step-3-overview'}
                                    color='indigo'
                                    theme={{
                                        color: {
                                            indigo: 'bg-indigo-600 text-white ring-indigo-600',
                                        },
                                    }}
                                    onClick={e => {
                                        handleSearch(e);
                                        tourController?.customGoToNextTourStep(1);
                                    }}
                                >
                                    Search
                                </Button>

                                <div>
                                    Enter the radius (in meters) and press Enter. Min:400 Max:1000
                                    <TextInput
                                        className={'tour-step-4-overview'}
                                        type='number'
                                        value={radiusInput}
                                        onChange={e => setRadiusInput(e.target.value)}
                                        onBlur={handleRadiusBlur}
                                        onKeyDown={e => {
                                            if (e.key === 'Enter') {
                                                e.preventDefault();
                                                handleRadiusBlur();
                                            }
                                        }}
                                        max={1000}
                                        min={10}
                                    />
                                </div>
                                <div>
                                    Enter a custom name or keep the suggested name.
                                    <TextInput
                                        className={'tour-step-5-overview'}
                                        type='text'
                                        value={customName}
                                        placeholder={'Custom name'}
                                        onChange={e => {
                                            setCustomName(e.target.value);
                                        }}
                                    />
                                </div>
                                <div style={{ display: 'flex', gap: '20px' }}>
                                    <div
                                        style={{
                                            flex: 1,
                                            display: 'flex',
                                            flexDirection: 'column',
                                            gap: '10px',
                                        }}
                                    >
                                        <Button
                                            outline
                                            color='indigo'
                                            theme={{
                                                color: {
                                                    indigo: 'bg-indigo-600 text-white ring-indigo-600',
                                                },
                                            }}
                                            onClick={() => {
                                                closeCreateTwinModal();
                                                resetAll();
                                            }}
                                        >
                                            Cancel
                                        </Button>
                                    </div>
                                    <div
                                        style={{
                                            flex: 1,
                                            display: 'flex',
                                            flexDirection: 'column',
                                            gap: '10px',
                                        }}
                                    >
                                        <Button
                                            className={'tour-step-6-overview'}
                                            color='indigo'
                                            theme={{
                                                color: {
                                                    indigo: 'bg-indigo-600 text-white ring-indigo-600',
                                                },
                                            }}
                                            onClick={() => {
                                                openConfirmModal();
                                                if (place !== '' && coords !== '') {
                                                    tourController?.customGoToNextTourStep(1);
                                                }
                                            }}
                                        >
                                            Create
                                        </Button>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </Modal.Body>
                    <Modal.Footer></Modal.Footer>
                </Modal>

                <Modal
                    show={isConfirmModalOpen}
                    style={{
                        filter: createTwinLoading ? 'blur(2px)' : 'none',
                        transition: 'filter 0.3s ease',
                        zIndex: 2000,
                    }}
                    onClose={() => {
                        resetAll();
                        closeCreateTwinModal();
                    }}
                >
                    <Modal.Header>Confirm Creation</Modal.Header>
                    <Modal.Body>
                        <b>Are you sure you want to create a twin with the following settings?</b>
                        <h1>Custom Name: {customName}</h1>
                        <div>Place: {place}</div>
                        <div>
                            Position: {position[0]} , {position[1]}{' '}
                        </div>
                        <div>Radius: {mapRadius} meters</div>
                        <div style={{ height: '400px', width: '100%', marginTop: '20px' }}>
                            <MapContainer
                                center={position}
                                zoom={14}
                                style={{ height: '100%', width: '100%' }}
                            >
                                <TileLayer url='https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png' />
                                <Marker position={position}></Marker>
                                <Rectangle bounds={bounds} />
                                <ChangeView center={position} zoom={14} />
                            </MapContainer>
                        </div>
                    </Modal.Body>
                    <Modal.Footer className='flex flex-row w-100'>
                        <Button
                            outline
                            color='indigo'
                            theme={{
                                color: { indigo: 'bg-indigo-600 text-white ring-indigo-600' },
                            }}
                            onClick={handleBack}
                        >
                            Back
                        </Button>
                        <div className='grow'></div>
                        <Button
                            className={'tour-step-7-overview'}
                            color='indigo'
                            theme={{
                                color: { indigo: 'bg-indigo-600 text-white ring-indigo-600' },
                            }}
                            onClick={closeModalsAndCreateTwin}
                        >
                            Create
                        </Button>
                    </Modal.Footer>
                </Modal>
            </>
        </>
    );
}

export default CreateTwinModal;
