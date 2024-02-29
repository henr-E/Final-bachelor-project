import React, { createContext, useReducer } from "react";

// TODO: replace with auto-gen interface from backend protobuffers
interface City {
    name: string;
    longitude: number;
    latitude: number;
}

interface CityState {
    current?: City;
    cities: City[];
};

interface LoadCitiesAction {
    type: 'load_cities';
    cities: City[];
}

interface SwitchCityAction {
    type: 'switch_city';
    city: City;
};

type CityAction = LoadCitiesAction | SwitchCityAction;

function reducer(state: CityState, action: CityAction): CityState {
    switch (action.type) {
        case 'load_cities': {
            return {
                ...state,
                cities: action.cities
            };
        }
        case 'switch_city': {
            return {
                ...state,
                current: action.city
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
        { name: 'Antwerp', longitude: 4.402771, latitude: 51.260197 },
        { name: 'Brussels', longitude: 4.34878, latitude: 50.85045 },
        { name: 'New York', longitude: 3.222, latitude: 23.67878}
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
