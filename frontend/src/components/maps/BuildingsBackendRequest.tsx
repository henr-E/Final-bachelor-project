// BuildingsBackendRequest.tsx:
import { createChannel, createClient } from 'nice-grpc-web';
import { TwinServiceDefinition, getBuildingsRequest } from '@/proto/twins/twin';
import { uiBackendServiceUrl } from '@/api/urls';

interface BuildingFeature {
    type: string;
    geometry: {
        type: string;
        coordinates: number[][][];
    };
    properties: { [key: string]: any };
}

interface GeoJSONBuildingResponse {
    type: string;
    features: BuildingFeature[];
}

async function BuildingsBackendRequest(id: string | undefined): Promise<GeoJSONBuildingResponse> {
    const channel = createChannel(uiBackendServiceUrl);
    const client = createClient(TwinServiceDefinition, channel);
    const request = { id: id };

    try {
        const response = await client.getBuildings(request);
        return JSON.parse(response.buildings);
    } catch (error) {
        console.error('Failed to fetch buildings:', error);
        // Return an empty GeoJSON structure in case of error
        return { type: 'FeatureCollection', features: [] };
    }
}

export default BuildingsBackendRequest;
