'use client';
import React, { createContext, useReducer } from 'react';
import {
    GetQuantitiesAndUnitsResponse_Unit as Unit,
    GetQuantitiesAndUnitsResponse_Quantity as Quantity,
} from '@/proto/sensor/sensor-crud';
import { BigDecimal } from '@/proto/sensor/bigdecimal';

enum Prefix {
    QUETTA,
    RONNA,
    YOTTA,
    ZETTA,
    EXA,
    PETA,
    TERA,
    GIGA,
    MEGA,
    KILO,
    HECTO,
    DECA,
    ONE,
    DECI,
    CENTI,
    MILI,
    MICRO,
    NANO,
    PICO,
    FEMTO,
    ATTO,
    ZEPTO,
    YOCTO,
    RONTO,
    QUECTO,
}

const prefixExponents: Record<Prefix, number> = {
    [Prefix.QUETTA]: 30,
    [Prefix.RONNA]: 27,
    [Prefix.YOTTA]: 24,
    [Prefix.ZETTA]: 21,
    [Prefix.EXA]: 18,
    [Prefix.PETA]: 15,
    [Prefix.TERA]: 12,
    [Prefix.GIGA]: 9,
    [Prefix.MEGA]: 6,
    [Prefix.KILO]: 3,
    [Prefix.HECTO]: 2,
    [Prefix.DECA]: 1,
    [Prefix.ONE]: 0,
    [Prefix.DECI]: -1,
    [Prefix.CENTI]: -2,
    [Prefix.MILI]: -3,
    [Prefix.MICRO]: -6,
    [Prefix.NANO]: -9,
    [Prefix.PICO]: -12,
    [Prefix.FEMTO]: -15,
    [Prefix.ATTO]: -18,
    [Prefix.ZEPTO]: -21,
    [Prefix.YOCTO]: -24,
    [Prefix.RONTO]: -27,
    [Prefix.QUECTO]: -30,
};

const prefixes: Array<Prefix> = [
    Prefix.QUETTA,
    Prefix.RONNA,
    Prefix.YOTTA,
    Prefix.ZETTA,
    Prefix.EXA,
    Prefix.PETA,
    Prefix.TERA,
    Prefix.GIGA,
    Prefix.MEGA,
    Prefix.KILO,
    Prefix.HECTO,
    Prefix.DECA,
    Prefix.ONE,
    Prefix.DECI,
    Prefix.CENTI,
    Prefix.MILI,
    Prefix.MICRO,
    Prefix.NANO,
    Prefix.PICO,
    Prefix.FEMTO,
    Prefix.ATTO,
    Prefix.ZEPTO,
    Prefix.YOCTO,
    Prefix.RONTO,
    Prefix.QUECTO,
];

export const prefixMap = new Map<number, string>([
    [30, 'QUETTA'],
    [27, 'RONNA'],
    [24, 'YOTTA'],
    [21, 'ZETTA'],
    [18, 'EXA'],
    [15, 'PETA'],
    [12, 'TERA'],
    [9, 'GIGA'],
    [6, 'MEGA'],
    [3, 'KILO'],
    [2, 'HECTO'],
    [1, 'DECA'],
    [0, 'ONE'],
    [-1, 'DECI'],
    [-2, 'CENTI'],
    [-3, 'MILLI'],
    [-6, 'MICRO'],
    [-9, 'NANO'],
    [-12, 'PICO'],
    [-15, 'FEMTO'],
    [-18, 'ATTO'],
    [-21, 'ZEPTO'],
    [-24, 'YOCTO'],
    [-27, 'RONTO'],
    [-30, 'QUECTO'],
]);

export function bigIntToExponent(bigDecimal: BigDecimal): number {
    //convert integer to 1 number
    //example 1000
    const number = parseInt(bigDecimal.integer.reverse().join(''));

    //count the amount of zeros
    //example 1000 => 3 zeros
    const str = number.toString(); // Convert the number to a string
    let zeroCount = 0; // Initialize zero counter

    for (const char of str) {
        if (char === '0') {
            zeroCount++; // Increment the counter for each zero found
        }
    }

    return bigDecimal.exponent + zeroCount;
}

interface QuantityWithUnits {
    quantity: Quantity;
    units: Unit[];
    baseUnit: string;
}

interface SensorState {
    // sensorID -> < signalID -> number >
    mostRecentValues: Record<string, Record<number, number>>;
}

interface SetMostRecentValueAction {
    type: 'set_most_recent_value';
    sensorId: string;
    signalId: number;
    value: number;
}

type SensorAction = SetMostRecentValueAction;

function reducer(state: SensorState, action: SensorAction): SensorState {
    switch (action.type) {
        case 'set_most_recent_value': {
            return {
                ...state,
                mostRecentValues: {
                    ...state.mostRecentValues,
                    [action.sensorId]: {
                        ...state.mostRecentValues[action.sensorId],
                        [action.signalId]: action.value,
                    },
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

const initialState: SensorState = {
    mostRecentValues: {},
};

const SensorContext = createContext<[SensorState, React.Dispatch<SensorAction>]>([
    initialState,
    async () => {},
]);

function SensorProvider({ children }: { children: React.ReactNode }) {
    const [state, dispatch] = useReducer(reducer, initialState);

    return <SensorContext.Provider value={[state, dispatch]}>{children}</SensorContext.Provider>;
}

export {
    Prefix,
    Quantity,
    SensorContext,
    SensorProvider,
    Unit,
    prefixExponents,
    prefixes,
    type QuantityWithUnits,
    type SensorAction,
};
