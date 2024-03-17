'use client'
import React, {createContext, useEffect, useReducer} from "react";
import {createChannel, createClient} from "nice-grpc-web";
import {TwinServiceDefinition} from '@/proto/twins/twin';
import {uiBackendServiceUrl} from "@/api/urls";
import ToastNotification from "@/components/notification/ToastNotification";


// TODO: replace with auto-gen interface from backend protobuffers
interface Twin {
    id: string; // UUID generated by server
    name: string;
    longitude: number;
    latitude: number;
    radius: number;
}

interface TwinState {
    current?: Twin;
    twins: Twin[];
};

interface SwitchTwinAction {
    type: 'switch_twin';
    twin: Twin;
};

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
                current: action.twin
            };
        }
        case 'load_twins': {
            return {
                ...state,
                twins: action.twins
            };
        }
        case 'create_twin': {
            const updatedTwins = [...state.twins, action.twin];
            return {
                ...state,
                current: action.twin,
                twins: updatedTwins
            };
        }
        default: {
            return {
                ...state
            };
        }
    }
}

const initialState: TwinState = {
    twins: []
}

async function getTwinsFromBackend(){
    const channel = createChannel(uiBackendServiceUrl);
    const client = createClient(TwinServiceDefinition, channel);

    try{
        // Notification("info", "All twins are being loaded.")
        return await client.getAllTwins({});

    }catch (error){
        console.error("Failed to fetch all twins:", error);
    }
}

const TwinContext = createContext<[TwinState, React.Dispatch<TwinAction>]>([initialState, () => { }]);

function TwinProvider({ children }: { children: React.ReactNode }) {
    const [state, dispatch] = useReducer(reducer, initialState);

    useEffect(() => {
        async function fetchTwins() {
            const twinsFromBackend = await getTwinsFromBackend();

            //check if empty => no twins
            if(twinsFromBackend?.twins == ""){
                //todo twins should be loaded when the dashboard is loaded and not when the page is loaded
                // ToastNotification("info", "You haven't created a twin yet!")
                dispatch({ type: 'load_twins', twins: [] });
            }
            else{
                ToastNotification("info", "All twins have been loaded.")
                //check if undefined
                if (twinsFromBackend?.twins != null) {
                    const twinsFromBackendJSON: Twin[] = JSON.parse(twinsFromBackend?.twins);
                    dispatch({ type: 'load_twins', twins: twinsFromBackendJSON });
                }
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
