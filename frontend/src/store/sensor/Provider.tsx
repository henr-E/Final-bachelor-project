'use client';
import React, { createContext, useReducer } from 'react';
import {
    GetQuantitiesAndUnitsResponse_Unit as Unit,
    GetQuantitiesAndUnitsResponse_Quantity as Quantity,
} from '@/proto/sensor/sensor-crud';

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

interface QuantityWithUnits {
    quantity: Quantity;
    units: Unit[];
    baseUnit: string;
}

interface SensorState {}

interface DeleteSensorAction {
    type: 'delete_sensor';
    sensorId: string;
}

type SensorAction = DeleteSensorAction;

function reducer(state: SensorState, action: SensorAction): SensorState {
    switch (action.type) {
        case 'delete_sensor': {
            return true;
        }
        default: {
            return {
                ...state,
            };
        }
    }
}

const SensorContext = createContext<[{ state: SensorState }, React.Dispatch<SensorAction>]>([
    { state: () => {} },
    async () => {},
]);

function SensorProvider({ children }: { children: React.ReactNode }) {
    const [state, dispatch] = useReducer(reducer, {});

    return (
        <SensorContext.Provider value={[{ state: state }, dispatch]}>
            {children}
        </SensorContext.Provider>
    );
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
