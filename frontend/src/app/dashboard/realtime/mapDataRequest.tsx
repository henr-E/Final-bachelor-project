'use client';

import React, { useEffect, useState } from 'react';
import { Empty } from '@/proto/google/protobuf/empty';
import { MapDataServiceClient, MapDataServiceDefinition } from '@/proto/map_data';
import { createChannel, createClient } from "nice-grpc-web";
import {geoServiceUrl} from "@/api/urls";

const MapDataRequest: React.FC = () => {
    const [data, setData] = useState<{streets: string[], houses: string[]}>({streets: [], houses: []});

    useEffect(() => {
        const fetchMapData = async () => {
            const channel = createChannel(geoServiceUrl);
            const client = createClient(MapDataServiceDefinition, channel);
            try {
                const response = await client.getMapData({});
                setData({streets: response.streets, houses: response.houses});
            } catch (error) {
                console.error('Failed to fetch map data:', error);
            }
        };

        fetchMapData();
    }, []);

    return (
        <div>
            <h2>Streets loaded from backend:</h2>
            <ul>
                {data.streets.map((street, index) => (
                    <li key={index}>{street}</li>
                ))}
            </ul>
            <h2>Houses loaded from backend:</h2>
            <ul>
                {data.houses.map((house, index) => (
                    <li key={index}>{house}</li>
                ))}
            </ul>
        </div>
    );
};

export default MapDataRequest;
