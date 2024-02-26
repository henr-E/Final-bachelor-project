'use client';

import React, { useEffect, useState } from 'react';
import {StringServiceClient, StringServiceDefinition} from '@/proto/string_list';
import {createChannel, createClient} from "nice-grpc-web"; // Update this path

const StringListRequest: React.FC = () => {
    const [strings, setStrings] = useState<string[]>([]);

    useEffect(() => {

        const fetchStrings = async () => {

            const channel = createChannel("http://127.0.0.1:8081");
            const client = createClient(StringServiceDefinition, channel)
            try {
                const response = await client.listStrings({});
                setStrings(response.strings);
            } catch (error) {
                console.error('Failed to fetch strings:', error);
            }
        };

        fetchStrings();
    }, []);

    return (
        <div>
            <h2>Strings loaded from backend:</h2>

            <ul>
                {strings.map((string, index) => (
                    <li key={index}>{string}</li>
                ))}
            </ul>
        </div>
    );
};

export default StringListRequest;
