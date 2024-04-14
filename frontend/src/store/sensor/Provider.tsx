'use client'
import React, {createContext, useContext, useEffect, useReducer, useState} from 'react';
import {BigInt, Sensor} from '@/proto/sensor/sensor-crud';
import {TwinContext} from "@/store/twins";

enum Quantity {
    // SI base quantities
    TIME,
    LENGTH,
    MASS,
    ELECTRIC_CURRENT,
    THERMODYNAMIC_TEMPERATURE,
    SUBSTANCE_AMOUNT,
    LUMINOUS_INTENSITY,
    // SI derived quantities (also refer to ISO 80000)
    FREQUENCY,
    FORCE,
    PRESSURE,
    ENERGY,
    POWER,
    ELECTRIC_CHARGE,
    ELECTRIC_POTENTIAL,
    CAPACITANCE,
    RESISTANCE,
    ELECTRICAL_CONDUCTANCE,
    MAGNETIC_FLUX,
    MAGNETIC_FLUX_DENSITY,
    INDUCTANCE,
    TEMPERATURE,
    LUMINOUS_FLUX,
    ILLUMINANCE,
    RADIONUCLIDE,
    ABSORBED_DOSE,
    EQUIVALENT_DOSE,
    CATALYTIC_ACTIVITY,
    // non-SI quantities (utility)
    RAINFALL,
    ELECTRICITY_CONSUMPTION,
}

enum Unit {
    // SI base units
    SECOND,
    METRE,
    KILOGRAM,
    AMPERE,
    KELVIN,
    MOLE,
    CANDELA,
    // SI derived units
    HERTZ,
    NEWTON,
    PASCAL,
    JOULE,
    WATT,
    COULOMB,
    VOLT,
    FARAD,
    OHM,
    SIEMENS,
    WEBER,
    TESLA,
    HENRY,
    DEGREES_CELSIUS,
    LUMEN,
    LUX,
    BECQUEREL,
    GRAY,
    SIEVERT,
    KATAL,
    // non-SI units (utility)
    WATT_HOUR, // using Wh instead of kWh, see https://en.wikipedia.org/wiki/Coherence_(units_of_measurement)
}

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

const quantityBaseUnits: Record<Quantity, Unit> = {
    [Quantity.TIME]: Unit.SECOND,
    [Quantity.LENGTH]: Unit.METRE,
    [Quantity.MASS]: Unit.KILOGRAM,
    [Quantity.ELECTRIC_CURRENT]: Unit.AMPERE,
    [Quantity.THERMODYNAMIC_TEMPERATURE]: Unit.KELVIN,
    [Quantity.SUBSTANCE_AMOUNT]: Unit.MOLE,
    [Quantity.LUMINOUS_INTENSITY]: Unit.CANDELA,
    [Quantity.FREQUENCY]: Unit.HERTZ,
    [Quantity.FORCE]: Unit.NEWTON,
    [Quantity.PRESSURE]: Unit.PASCAL,
    [Quantity.ENERGY]: Unit.JOULE,
    [Quantity.POWER]: Unit.WATT,
    [Quantity.ELECTRIC_CHARGE]: Unit.COULOMB,
    [Quantity.ELECTRIC_POTENTIAL]: Unit.VOLT,
    [Quantity.CAPACITANCE]: Unit.FARAD,
    [Quantity.RESISTANCE]: Unit.OHM,
    [Quantity.ELECTRICAL_CONDUCTANCE]: Unit.SIEMENS,
    [Quantity.MAGNETIC_FLUX]: Unit.WEBER,
    [Quantity.MAGNETIC_FLUX_DENSITY]: Unit.TESLA,
    [Quantity.INDUCTANCE]: Unit.HENRY,
    [Quantity.TEMPERATURE]: Unit.DEGREES_CELSIUS,
    [Quantity.LUMINOUS_FLUX]: Unit.LUMEN,
    [Quantity.ILLUMINANCE]: Unit.LUX,
    [Quantity.RADIONUCLIDE]: Unit.BECQUEREL,
    [Quantity.ABSORBED_DOSE]: Unit.GRAY,
    [Quantity.EQUIVALENT_DOSE]: Unit.SIEVERT,
    [Quantity.CATALYTIC_ACTIVITY]: Unit.KATAL,
    [Quantity.RAINFALL]: Unit.METRE,
    [Quantity.ELECTRICITY_CONSUMPTION]: Unit.WATT_HOUR,
};

const quantities: Array<Quantity> = [
    Quantity.TIME,
    Quantity.LENGTH,
    Quantity.MASS,
    Quantity.ELECTRIC_CURRENT,
    Quantity.THERMODYNAMIC_TEMPERATURE,
    Quantity.SUBSTANCE_AMOUNT,
    Quantity.LUMINOUS_INTENSITY,
    Quantity.FREQUENCY,
    Quantity.FORCE,
    Quantity.PRESSURE,
    Quantity.ENERGY,
    Quantity.POWER,
    Quantity.ELECTRIC_CHARGE,
    Quantity.ELECTRIC_POTENTIAL,
    Quantity.CAPACITANCE,
    Quantity.RESISTANCE,
    Quantity.ELECTRICAL_CONDUCTANCE,
    Quantity.MAGNETIC_FLUX,
    Quantity.MAGNETIC_FLUX_DENSITY,
    Quantity.INDUCTANCE,
    Quantity.TEMPERATURE,
    Quantity.LUMINOUS_FLUX,
    Quantity.ILLUMINANCE,
    Quantity.RADIONUCLIDE,
    Quantity.ABSORBED_DOSE,
    Quantity.EQUIVALENT_DOSE,
    Quantity.CATALYTIC_ACTIVITY,
    Quantity.RAINFALL,
    Quantity.ELECTRICITY_CONSUMPTION,
];

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

interface SensorState {
}

interface DeleteSensorAction {
    type: 'delete_sensor';
    sensorId: string;
}

type SensorAction =
    | DeleteSensorAction;

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

const SensorContext = createContext<[{state: SensorState }, React.Dispatch<SensorAction>]>([{state: () => {}}, async () => {}]);

function SensorProvider({children}: { children: React.ReactNode }) {
    const [state, dispatch] = useReducer(reducer, {});

    return (
        <SensorContext.Provider value={[{state: state}, dispatch]}>
            {children}
        </SensorContext.Provider>
    );
}

export {
    Quantity,
    Unit,
    Prefix,
    prefixExponents,
    quantityBaseUnits,
    quantities,
    prefixes,
    SensorProvider,
    SensorContext,
    type SensorAction,
};
