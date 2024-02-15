import React, { createContext, useReducer } from "react";

// TODO: replace with auto-gen interface from backend protobuffers
interface City {
    name: string;
    longitude: number;
    latitde: number;
}

interface CityState {
    city?: City;
};

interface SwitchCityAction {
    type: 'switch_city';
    city: City;
};

type CityAction = SwitchCityAction;

function reducer(state: CityState, action: CityAction): CityState {
    switch (action.type) {
        case 'switch_city': {
            return {
                city: action.city,
                ...state
            };
        }
        default: {
            return state;
        }
    }
}

const CityContext = createContext<[CityState, React.Dispatch<CityAction>]>([{}, () => { }]);

function CityProvider({ children }: { children: React.ReactNode }) {
    const [state, dispatch] = useReducer(reducer, {});

    return (
        <CityContext.Provider value={[state, dispatch]}>
            {children}
        </CityContext.Provider>
    );
}

export { CityProvider, CityContext };
