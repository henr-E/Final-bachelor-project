'use client';
import React, { createContext, useEffect, useReducer } from 'react';
import { createChannel, createClient } from 'nice-grpc-web';
import { twinObject, TwinServiceDefinition } from '@/proto/twins/twin';
import { uiBackendServiceUrl } from '@/api/urls';
import ToastNotification from '@/components/notification/ToastNotification';
import { BackendGetTwins } from '@/api/twins/crud';
import { BackendGetSimulations } from '@/api/simulation/crud';
import { BackendGetSensors } from '@/api/sensor/crud';
import { Sensor } from '@/proto/sensor/sensor-crud';
import { Simulation } from '@/proto/simulation/frontend';
import { useRouter } from 'next/navigation';

interface TwinFromProvider {
    id: number;
    name: string;
    longitude: number;
    latitude: number;
    radius: number;
    simulations: Simulation[];
    sensors: Sensor[];
    creation_date_time: number;
    simulation_amount: number;
}

interface TwinState {
    current?: TwinFromProvider;
    twins: TwinFromProvider[];
}

interface SwitchTwinAction {
    type: 'switch_twin';
    twin: TwinFromProvider;
}

interface LoadTwinsAction {
    type: 'load_twins';
    twins: TwinFromProvider[];
}

interface CreateTwin {
    type: 'create_twin';
    twin: TwinFromProvider;
}

interface DeleteTwin {
    type: 'delete_twin';
    twinId: number;
}

interface DeleteSensorAction {
    type: 'delete_sensor';
    sensorId: string;
}

interface DeleteSimulationAction {
    type: 'delete_simulation';
    simulationName: string;
}

interface LoadSimulationsAction {
    type: 'load_simulations';
    simulations: Simulation[];
}

interface LoadSensorsAction {
    type: 'load_sensors';
    sensors: Sensor[];
}

type TwinAction =
    | SwitchTwinAction
    | LoadTwinsAction
    | CreateTwin
    | DeleteTwin
    | LoadSimulationsAction
    | LoadSensorsAction
    | DeleteSensorAction
    | DeleteSimulationAction;

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
        case 'delete_twin': {
            const remainingTwins = state.twins.filter(twin => twin.id !== action.twinId);
            const isCurrentTwinDeleted = state.current?.id === action.twinId;
            return {
                ...state,
                twins: remainingTwins,
                current: isCurrentTwinDeleted ? remainingTwins[0] || null : state.current,
            };
        }
        case 'delete_sensor': {
            if (state.current) {
                const updatedSensors = state.current.sensors.filter(
                    sensor => sensor.id !== action.sensorId
                );
                return {
                    ...state,
                    current: {
                        ...state.current,
                        sensors: updatedSensors,
                    },
                };
            }
            return state;
        }
        case 'delete_simulation': {
            if (state.current) {
                const updatedSimulations = state.current.simulations.filter(
                    simulation => simulation.name !== action.simulationName
                );
                return {
                    ...state,
                    current: {
                        ...state.current,
                        simulations: updatedSimulations,
                    },
                };
            }
            return state;
        }

        case 'load_simulations': {
            if (!state.current) {
                console.error('Cannot load simulations: current twin is undefined.');
                return state;
            }

            return {
                ...state,
                current: {
                    ...state.current,
                    simulations: action.simulations,
                },
            };
        }
        case 'load_sensors': {
            if (!state.current) {
                console.error('Cannot load sensors: current twin is undefined.');
                return state;
            }

            return {
                ...state,
                current: {
                    ...state.current,
                    sensors: action.sensors,
                },
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

const TwinContext = createContext<[TwinState, React.Dispatch<TwinAction>]>([
    initialState,
    () => {},
]);

function TwinProvider({ children }: { children: React.ReactNode }) {
    const [state, dispatchTwin] = useReducer(reducer, initialState);
    const router = useRouter();

    useEffect(() => {
        async function getTwins() {
            let response = await BackendGetTwins();
            if (response) {
                let twinsFromBackend = response.twins.map((twinObj: twinObject) => ({
                    id: twinObj.id,
                    name: twinObj.name,
                    longitude: twinObj.longitude,
                    latitude: twinObj.latitude,
                    radius: Number(twinObj.radius),
                    sensors: [],
                    simulations: [],
                    creation_date_time: twinObj.creationDateTime,
                    simulation_amount: twinObj.simulationAmount,
                }));

                if (twinsFromBackend.length > 0) {
                    // Load all twins into the state
                    dispatchTwin({ type: 'load_twins', twins: twinsFromBackend });
                    // Set the current twin to the first twin from the list
                    dispatchTwin({ type: 'switch_twin', twin: twinsFromBackend[0] });
                    ToastNotification('info', 'All twins are being loaded.');
                } else {
                    // Optionally handle the case where no twins are returned
                    ToastNotification('info', 'No twins found.');
                }
            }
        }

        getTwins();
    }, [router]);

    useEffect(() => {
        const currentId = state.current?.id;
        if (currentId) {
            const fetchSimulations = async () => {
                ToastNotification('info', 'loading all simulations');
                const simulationsResult = await BackendGetSimulations(String(currentId));
                dispatchTwin({ type: 'load_simulations', simulations: simulationsResult.item });
            };

            const fetchSensors = async () => {
                ToastNotification('info', 'loading all sensors');
                const sensorsResult = await BackendGetSensors(currentId);
                dispatchTwin({ type: 'load_sensors', sensors: sensorsResult });
            };

            fetchSimulations();
            fetchSensors();
        }
        // eslint-disable-next-line
    }, [state.current?.id]);

    return <TwinContext.Provider value={[state, dispatchTwin]}>{children}</TwinContext.Provider>;
}

export { type TwinFromProvider, TwinProvider, TwinContext };
