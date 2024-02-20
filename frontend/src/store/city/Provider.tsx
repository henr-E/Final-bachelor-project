import React, { createContext, useReducer } from "react";

// TODO: replace with auto-gen interface from backend protobuffers
interface City {
    name: string;
    longitude: number;
    latitde: number;
}

interface CityState {
    cities: City[];
};

interface LoadCitiesAction {
    type: 'load_cities';
    cities: City[];
}

type CityAction = LoadCitiesAction;

function reducer(state: CityState, action: CityAction): CityState {
    switch (action.type) {
        case 'load_cities': {
            return {
                ...state,
                cities: action.cities
            };
        }
        default: {
            return {
                ...state
            };
        }
    }
}

const initialState: CityState = {
    cities: [
        { name: 'Antwerp', longitude: 4.402771, latitde: 51.260197 },
        { name: 'Brussels', longitude: 4.34878, latitde: 50.85045 }
    ]
}

const CityContext = createContext<[CityState, React.Dispatch<CityAction>]>([initialState, () => { }]);

function CityProvider({ children }: { children: React.ReactNode }) {
    const [state, dispatch] = useReducer(reducer, initialState);

    return (
        <CityContext.Provider value={[state, dispatch]}>
            {children}
        </CityContext.Provider>
    );
}

export { type City, CityProvider, CityContext };
