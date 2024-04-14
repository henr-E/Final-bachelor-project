'use client'
import React, {createContext, useContext, useEffect, useReducer, useState} from 'react';
import {TwinContext} from "@/store/twins";
import {Simulation} from "@/proto/simulation/frontend";

interface SimulationState {
}

interface DeleteSimulationAction {
    type: 'delete_simulation';
    simulationId: string;
}

type SimulationAction =
    | DeleteSimulationAction;


function reducer(state: SimulationState, action: SimulationAction): SimulationState {
    switch (action.type) {
        case 'delete_simulation': {
            return true;
        }
        default: {
            return {
                ...state,
            };
        }
    }
}


const SimulationContext = createContext<
    [{state: SimulationState }, React.Dispatch<SimulationAction>]>([{state: () => {}}, async () => {}]);


function SimulationProvider({children}: { children: React.ReactNode }) {
    const [state, dispatch] = useReducer(reducer, {});

    return (
        <SimulationContext.Provider value={[{state: state}, dispatch]}>
            {children}
        </SimulationContext.Provider>
    );
}

export {SimulationProvider, SimulationContext};
