'use client';
import React, {createContext, useEffect, useReducer} from "react";
import {createChannel, createClient} from "nice-grpc-web";
import {twinObject, TwinServiceDefinition} from '@/proto/twins/twin';
import {uiBackendServiceUrl} from "@/api/urls";
import ToastNotification from "@/components/notification/ToastNotification";

// TODO: replace with auto-gen interface from backend protobuffers
interface Twin {
    id: number;
    name: string;
    longitude: number;
    latitude: number;
    radius: number;
}

interface TwinState {
    current?: Twin;
    twins: Twin[];
}

interface SwitchTwinAction {
    type: 'switch_twin';
    twin: Twin;
}

interface LoadTwinsAction {
    type: 'load_twins';
    twins: Twin[];
}

interface CreateTwin {
    type: 'create_twin';
    twin: Twin;
}

type TwinAction = SwitchTwinAction | LoadTwinsAction | CreateTwin;

function reducer(state: TwinState, action: TwinAction): TwinState {
    switch (action.type) {
        case 'switch_twin': {
            return {
                ...state,
                current: action.twin,
            };
        }
        case 'load_twins': {
            return {
                ...state,
                twins: action.twins,
            };
        }
        case 'create_twin': {
            const updatedTwins = [...state.twins, action.twin];
            return {
                ...state,
                current: action.twin,
                twins: updatedTwins,
            };
        }
        default: {
            return {
                ...state,
            };
        }
    }
}

const initialState: TwinState = {
    twins: [],
};

async function getTwinsFromBackend(): Promise<Twin[]> {
    const channel = createChannel(uiBackendServiceUrl);
    const client = createClient(TwinServiceDefinition, channel);

    try {
        let response = await client.getAllTwins({});
        if (response.twins === undefined || response.twins.length === 0) {
            return [];
        } else {
            // Map the response to your Twin interface
            return response.twins.map((twinObj: any) => ({
                id: twinObj.id,
                name: twinObj.name,
                longitude: twinObj.longitude,
                latitude: twinObj.latitude,
                radius: Number(twinObj.radius)
            }));


        }
    } catch (error) {
        console.error("Failed to fetch all twins:", error);
        return [];
    }
}

const TwinContext = createContext<[TwinState, React.Dispatch<TwinAction>]>([
    initialState,
    () => {},
]);

function TwinProvider({ children }: { children: React.ReactNode }) {
    const [state, dispatch] = useReducer(reducer, initialState);

    useEffect(() => {
        async function fetchTwins() {
            const twinsFromBackend = await getTwinsFromBackend();

            if (twinsFromBackend.length > 0) {
                // Load all twins into the state
                dispatch({ type: 'load_twins', twins: twinsFromBackend });
                // Set the current twin to the first twin from the list
                dispatch({ type: 'switch_twin', twin: twinsFromBackend[0] });
                ToastNotification("info", "All twins are being loaded.")
            } else {
                // Optionally handle the case where no twins are returned
                ToastNotification("info", "No twins found.");
            }
        }

        fetchTwins();
    }, []);

    return (
        <TwinContext.Provider value={[state, dispatch]}>
            {children}
        </TwinContext.Provider>
    );
}

export { type Twin, TwinProvider, TwinContext };
