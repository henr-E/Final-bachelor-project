'use client';
import React from 'react';
import 'leaflet/dist/leaflet.css';
import { Button, Modal } from 'flowbite-react';
import { Sensor } from '@/proto/sensor/sensor-crud';
import { BigDecimal } from '@/proto/sensor/bigdecimal';

interface ShowSignalsModalProps {
    isModalOpen: boolean;
    sensor?: Sensor;
    closeModal: () => void;
}

const prefixMap = new Map<number, string>([
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

function formatBigInt(bigDecimal: BigDecimal | null | undefined): string {
    if (bigDecimal === null || bigDecimal === undefined) return 'None';

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

    let amountOfZeros = bigDecimal.exponent + zeroCount;
    let prefix = prefixMap.get(amountOfZeros);

    if (prefix === undefined) {
        return 'No prefix found';
    }
    return prefix;
}

/**
 *
 * @param isModalOpen
 * @param sensor
 * @param closeModal
 * @constructor
 */
function ShowSignalsModal({ isModalOpen, sensor, closeModal }: ShowSignalsModalProps) {
    return (
        <>
            <Modal
                show={isModalOpen}
                onClose={() => {
                    closeModal();
                }}
                style={{
                    maxWidth: '100%',
                    maxHeight: '100%',
                    zIndex: 2000,
                }}
            >
                <Modal.Header>Signals for Sensor {sensor?.name} </Modal.Header>
                <Modal.Body>
                    <div className='shadow-md sm:rounded-lg bg-white p-2 w-full min-h-96 relative'>
                        <table className='text-sm text-left rtl:text-right text-gray-500 w-full table-auto'>
                            <thead className='border-gray-600 text-xs text-gray-700 uppercase bg-gray-50 dark:bg-gray-700 dark:text-gray-400'>
                                <tr>
                                    <th scope='col' className='p-3 px-3' style={{ width: '20%' }}>
                                        Alias
                                    </th>
                                    <th scope='col' className='p-3 px-3' style={{ width: '20%' }}>
                                        Unit
                                    </th>
                                    <th scope='col' className='p-3 px-3' style={{ width: '20%' }}>
                                        Quantity
                                    </th>
                                    <th scope='col' className='p-3 px-3' style={{ width: '20%' }}>
                                        Unit
                                    </th>
                                    <th scope='col' className='p-3 px-3' style={{ width: '20%' }}>
                                        Prefix
                                    </th>
                                </tr>
                            </thead>
                            <tbody>
                                {sensor?.signals?.map((signal, index) => (
                                    <tr key={index} className='my-6' style={{ cursor: 'pointer' }}>
                                        <td className='p-3 px-3'>{signal.alias}</td>
                                        <td className='p-3 px-3'>{signal.unit}</td>
                                        <td className='p-3 px-3'>{signal.quantity}</td>
                                        <td className='p-3 px-3'>{signal.ingestionUnit}</td>
                                        <td className='p-3 px-3'>{formatBigInt(signal.prefix)}</td>
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    </div>
                </Modal.Body>
                <Modal.Footer>
                    <Button
                        outline
                        color='indigo'
                        theme={{
                            color: {
                                indigo: 'bg-indigo-600 text-white ring-indigo-600',
                            },
                        }}
                        onClick={() => {
                            closeModal();
                        }}
                    >
                        Back
                    </Button>
                </Modal.Footer>
            </Modal>
        </>
    );
}

export default ShowSignalsModal;
