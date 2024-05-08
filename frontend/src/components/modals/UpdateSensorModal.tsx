'use client';

import { useState, useRef, useEffect, useContext } from 'react';
import {
    Quantity,
    prefixes as allPrefixes,
    prefixExponents,
    Unit,
    Prefix,
    QuantityWithUnits,
    prefixMap,
} from '@/store/sensor';
import { v4 as uuidv4 } from 'uuid';
import { Button, ButtonGroup, Label, Modal } from 'flowbite-react';
import { HiXMark } from 'react-icons/hi2';
import { Sensor } from '@/proto/sensor/sensor-crud';
import ToastNotification from '@/components/notification/ToastNotification';
import { TwinContext } from '@/store/twins';
import { BackendGetQuantityWithUnits } from '@/api/sensor/crud';
import { BigDecimal } from '@/proto/sensor/bigdecimal';
import { bigIntToExponent } from '@/store/sensor';
import { getFirstQuantity } from '@/lib/util';

interface UpdateSensorModalProps {
    isModalOpen: boolean;
    selectedBuildingId: number | null;
    handleUpdateSensor: (sensor: Sensor) => Promise<void>;
    closeModal: () => void;
    sensor: Sensor;
}

enum ModalPage {
    BASIC,
    SIGNALS,
    INGEST,
}

function UpdateSensorModal({
    isModalOpen,
    selectedBuildingId,
    closeModal,
    handleUpdateSensor,
    sensor,
}: UpdateSensorModalProps) {
    const [modalPage, setModalPage] = useState<ModalPage>(ModalPage.BASIC);

    const [quantitiesWithUnits, setQuantitiesWithUnits] = useState<
        Record<string, QuantityWithUnits>
    >({});

    // step 1: general settings
    const [name, setName] = useState(sensor.name);
    const [description, setDescription] = useState(sensor.description);

    // step 2: quantities
    const [quantity, setQuantity] = useState<Quantity | undefined>();
    const [quantities, setQuantities] = useState<Record<string, Quantity>>({});

    // step 3: signals
    const [units, setUnits] = useState<Record<string, Unit>>({});
    const [prefixes, setPrefixes] = useState<Record<string, Prefix>>({});
    const [aliases, setAliases] = useState<Record<string, string>>({});

    const [twinState, dispatch] = useContext(TwinContext);

    const basicFormRef = useRef<HTMLFormElement>(null);

    useEffect(() => {
        BackendGetQuantityWithUnits().then(quantitiesWithUnits => {
            setQuantitiesWithUnits(quantitiesWithUnits);

            const existingSensorQuantities = Object.fromEntries(
                sensor.signals.map(s => {
                    const quantity: Quantity = quantitiesWithUnits[s.quantity].quantity;

                    return [s.id, quantity];
                })
            );

            const existingSensorUnits = Object.fromEntries(
                sensor.signals.map(s => {
                    const unit: Unit | undefined = quantitiesWithUnits[s.quantity].units.find(
                        u => u.repr === s.unit.toUpperCase()
                    );

                    if (!unit) {
                        throw new Error(
                            'Update sensor modal: failed to load unit from the backend for an existing sensor'
                        );
                    }

                    return [s.id, unit];
                })
            );

            const existingSensorAliases = Object.fromEntries(
                sensor.signals.map(s => [s.id, s.alias])
            );

            const existingSensorPrefixes = Object.fromEntries(
                sensor.signals.map(s => {
                    if (!s.prefix) {
                        return [s.id, Prefix.ONE];
                    }

                    const exp = s.prefix.exponent + Math.log(s.prefix.integer[0]) / Math.log(10);

                    const prefix = allPrefixes.find(p => prefixExponents[p] === exp);

                    return [s.id, prefix as Prefix];
                })
            );

            setQuantities(existingSensorQuantities);
            setUnits(existingSensorUnits);
            setAliases(existingSensorAliases);
            setPrefixes(existingSensorPrefixes);
        });
    }, []);

    const handleModalClose = () => {
        setModalPage(ModalPage.BASIC);

        closeModal();
    };

    const handleNextButtonClick = async () => {
        switch (modalPage) {
            case ModalPage.BASIC: {
                if (!basicFormRef.current?.reportValidity()) return;
                setModalPage(modalPage + 1);
                break;
            }
            case ModalPage.SIGNALS: {
                setModalPage(modalPage + 1);
                break;
            }
            case ModalPage.INGEST: {
                if (!twinState.current?.id) {
                    ToastNotification('error', 'Something went wrong when creating the sensor.');
                    break;
                }

                //if no building is selected, it is a global sensor and it should be set to undefined.
                //Convert the selectedBuildingId from number | null to number | undefined.
                //I chose to not make selectedBuildingId number | undefined because if something goes wrong it will
                // become undefined and but won't give an error because it is a valid type.
                let sensorSelectedBuildingId: number | undefined;

                if (selectedBuildingId == null) {
                    sensorSelectedBuildingId = undefined;
                } else {
                    sensorSelectedBuildingId = selectedBuildingId;
                }

                const newSensor: Sensor = {
                    id: sensor.id,
                    twinId: twinState.current?.id,
                    name: name,
                    description: description,
                    latitude: 51,
                    longitude: 4.1,
                    signals: Object.entries(quantities).map(([id, quantity], i) => ({
                        id: 0,
                        quantity: quantity.id,
                        unit: quantitiesWithUnits[quantity.id].baseUnit,
                        ingestionUnit: units[id].id,
                        prefix: {
                            sign: false,
                            integer: [1],
                            exponent: prefixExponents[prefixes[id]],
                        },
                        alias: aliases[id],
                    })),
                    buildingId: sensorSelectedBuildingId,
                };

                await handleUpdateSensor(newSensor);

                handleModalClose();

                break;
            }
        }
    };

    const getBaseUnit = (q: QuantityWithUnits): Unit => {
        const baseUnitId = q.baseUnit;
        const unit = q.units.find(u => u.id === baseUnitId);
        if (unit === undefined) {
            throw Error('Base unit id not found in map of units (should be unreachable).');
        }
        return unit;
    };

    const handlePreviousButtonClick = () => {
        setModalPage(modalPage - 1);

        if (modalPage === ModalPage.BASIC) {
            handleModalClose();
        }
    };

    const UpdateSensorStepper = ({ page }: { page: ModalPage }) => {
        const activeStyles = 'text-indigo-600';

        return (
            <ol className='flex items-center w-full text-sm font-medium text-center text-gray-500 sm:text-base mb-8 '>
                <li
                    className={`flex md:w-full items-center ${activeStyles} sm:after:content-[''] after:w-full after:h-1 after:border-b after:border-gray-200 after:border-1 after:hidden sm:after:inline-block after:mx-6 xl:after:mx-10`}
                >
                    <span className="flex items-center after:content-['/'] sm:after:hidden after:mx-2 after:text-gray-200">
                        <svg
                            className='w-3.5 h-3.5 sm:w-4 sm:h-4 me-2.5'
                            aria-hidden='true'
                            xmlns='http://www.w3.org/2000/svg'
                            fill='currentColor'
                            viewBox='0 0 20 20'
                        >
                            <path d='M10 .5a9.5 9.5 0 1 0 9.5 9.5A9.51 9.51 0 0 0 10 .5Zm3.707 8.207-4 4a1 1 0 0 1-1.414 0l-2-2a1 1 0 0 1 1.414-1.414L9 10.586l3.293-3.293a1 1 0 0 1 1.414 1.414Z' />
                        </svg>
                        <span className={page === ModalPage.BASIC ? '' : ''}>General</span>
                    </span>
                </li>
                <li
                    className={`flex md:w-full items-center ${
                        page === ModalPage.INGEST || page === ModalPage.SIGNALS ? activeStyles : ''
                    } after:content-[''] after:w-full after:h-1 after:border-b after:border-gray-200 after:border-1 after:hidden sm:after:inline-block after:mx-6 xl:after:mx-10`}
                >
                    <span
                        className={`flex items-center after:content-['/'] sm:after:hidden after:mx-2 after:text-gray-200`}
                    >
                        <svg
                            className='w-3.5 h-3.5 sm:w-4 sm:h-4 me-2.5'
                            aria-hidden='true'
                            xmlns='http://www.w3.org/2000/svg'
                            fill='currentColor'
                            viewBox='0 0 20 20'
                        >
                            <path d='M10 .5a9.5 9.5 0 1 0 9.5 9.5A9.51 9.51 0 0 0 10 .5Zm3.707 8.207-4 4a1 1 0 0 1-1.414 0l-2-2a1 1 0 0 1 1.414-1.414L9 10.586l3.293-3.293a1 1 0 0 1 1.414 1.414Z' />
                        </svg>
                        Signals
                    </span>
                </li>
                <li
                    className={`flex items-center ${page === ModalPage.INGEST ? activeStyles : ''}`}
                >
                    <svg
                        className='w-3.5 h-3.5 sm:w-4 sm:h-4 me-2.5'
                        aria-hidden='true'
                        xmlns='http://www.w3.org/2000/svg'
                        fill='currentColor'
                        viewBox='0 0 20 20'
                    >
                        <path d='M10 .5a9.5 9.5 0 1 0 9.5 9.5A9.51 9.51 0 0 0 10 .5Zm3.707 8.207-4 4a1 1 0 0 1-1.414 0l-2-2a1 1 0 0 1 1.414-1.414L9 10.586l3.293-3.293a1 1 0 0 1 1.414 1.414Z' />
                    </svg>
                    Format
                </li>
            </ol>
        );
    };

    const handleAddSignalButtonClick = () => {
        if (quantity && !Object.values(quantities).includes(quantity)) {
            const id = uuidv4();

            setQuantities({
                ...quantities,
                [id]: quantity,
            });

            setUnits({
                ...units,
                [id]: getBaseUnit(quantitiesWithUnits[quantity.id]),
            });

            setPrefixes({
                ...prefixes,
                [id]: Prefix.ONE,
            });

            setAliases({
                ...aliases,
                [id]: `${quantity.repr}-N`,
            });
        } else {
            ToastNotification('warning', 'Quantity already exists.');
        }
    };

    return (
        <>
            <Modal
                show={isModalOpen}
                size={modalPage === ModalPage.INGEST ? '4xl' : 'xl'}
                onClose={handleModalClose}
            >
                <Modal.Header>
                    Update Sensor (
                    {selectedBuildingId === null
                        ? `Global Sensor for twin ${twinState.current?.name}`
                        : `Sensor for building number ${selectedBuildingId}`}
                    )
                </Modal.Header>
                <Modal.Body>
                    <UpdateSensorStepper page={modalPage} />
                    {modalPage === ModalPage.BASIC && (
                        <div className='my-4'>
                            <form ref={basicFormRef}>
                                <div>
                                    <div className='mb-2 block'>
                                        <Label htmlFor='name' value='Name' />
                                    </div>
                                    <input
                                        id='name'
                                        className='bg-gray-50 border border-gray-300 text-gray-900 rounded-lg text-sm focus:ring-indigo-500 w-full focus:border-indigo-500 p-2.5'
                                        type='text'
                                        value={name}
                                        placeholder='name'
                                        required
                                        maxLength={50}
                                        onChange={e => setName(e.target.value)}
                                        style={{ marginBottom: '10px' }}
                                    />
                                </div>
                                <div>
                                    <div className='mb-2 block'>
                                        <Label htmlFor='description' value='Description' />
                                    </div>
                                    <input
                                        id='description'
                                        className='bg-gray-50 border border-gray-300 text-gray-900 rounded-lg text-sm focus:ring-indigo-500 w-full focus:border-indigo-500 p-2.5'
                                        type='text'
                                        value={description}
                                        placeholder={'description'}
                                        maxLength={200}
                                        required
                                        onChange={e => setDescription(e.target.value)}
                                    />
                                </div>
                            </form>
                        </div>
                    )}
                    {modalPage === ModalPage.SIGNALS && (
                        <div className='flex flex-col space-y-4'>
                            <Label>What does this sensor measure?</Label>
                            <ButtonGroup className=''>
                                <select
                                    id='quantity'
                                    className='bg-gray-50 border border-gray-300 text-gray-900 rounded-l-lg text-sm focus:ring-indigo-500 focus:border-indigo-500 p-2.5'
                                    value={quantity?.id}
                                    onChange={e =>
                                        setQuantity(quantitiesWithUnits[e.target.value].quantity)
                                    }
                                    required
                                >
                                    {Object.values(quantitiesWithUnits).map((q, i) => (
                                        <option key={i} value={q.quantity.id}>
                                            {q.quantity.repr}
                                        </option>
                                    ))}
                                </select>
                                <button
                                    className='px-3 py-2 text-sm font-medium text-white rounded-r-lg bg-indigo-600'
                                    onClick={handleAddSignalButtonClick}
                                >
                                    Add Signal
                                </button>
                            </ButtonGroup>
                            <div className='w-100 min-h-32 p-2 bg-indigo-200 rounded-xl shadow-md flex flex-row flex-wrap items-center justify-center gap-2'>
                                {Object.entries(quantities).map(([id, quantity], i) => (
                                    <Button
                                        key={id}
                                        color='indigo'
                                        onClick={() => {
                                            if (quantity.id === 'timestamp') return;

                                            delete quantities[id];
                                            delete units[id];
                                            delete prefixes[id];
                                            delete aliases[id];

                                            setQuantities({ ...quantities });
                                            setUnits({ ...units });
                                            setPrefixes({ ...prefixes });
                                            setAliases({ ...aliases });
                                        }}
                                        theme={{
                                            color: {
                                                indigo: 'bg-indigo-600 text-white ring-indigo-600',
                                            },
                                        }}
                                        pill
                                    >
                                        {quantity.repr}
                                        {quantity.id !== 'timestamp' && (
                                            <HiXMark className='ml-2 text-gray-200' size={20} />
                                        )}
                                    </Button>
                                ))}
                                {Object.values(quantities).length === 0 && (
                                    <span className='text-sm text-gray-500'>No Signals</span>
                                )}
                            </div>
                        </div>
                    )}
                    {modalPage === ModalPage.INGEST && (
                        <div className='flex flex-col space-y-6'>
                            <Label>What is the format of the measured data?</Label>
                            <table className='text-sm text-left rtl:text-right text-gray-800 w-full table-auto'>
                                <thead className='border-gray-600 text-xs uppercase'>
                                    <tr>
                                        <th scope='col' className='w-32 px-2'>
                                            Signal
                                        </th>
                                        <th scope='col' className='w-32 px-2'>
                                            Unit
                                        </th>
                                        <th scope='col' className='w-24 px-2'>
                                            Prefix
                                        </th>
                                        <th scope='col' className='w-48 px-2'>
                                            Alias
                                        </th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {Object.entries(quantities).map(([id, quantity]) => (
                                        <tr key={id}>
                                            <td className='w-32 p-2'>
                                                <span>{quantity.repr}</span>
                                            </td>
                                            <td className='w-48 p-2'>
                                                <select
                                                    value={units[id].id}
                                                    onChange={e => {
                                                        setUnits({
                                                            ...units,
                                                            [id]: quantitiesWithUnits[
                                                                quantity.id
                                                            ].units.find(
                                                                u => u.id === e.target.value
                                                            ) as Unit,
                                                        });
                                                    }}
                                                    className='block w-full p-2 text-gray-900 border border-gray-300 rounded-lg bg-gray-50 text-xs focus:ring-indigo-500 focus:border-indigo-500'
                                                >
                                                    {quantitiesWithUnits[quantity.id].units.map(
                                                        u => (
                                                            <option key={u.id} value={u.id}>
                                                                {u.repr}
                                                            </option>
                                                        )
                                                    )}
                                                </select>
                                            </td>
                                            <td className='w-24 p-2'>
                                                <select
                                                    value={prefixes[id]}
                                                    onChange={e => {
                                                        setPrefixes({
                                                            ...prefixes,
                                                            [id]: parseInt(e.target.value),
                                                        });
                                                    }}
                                                    className='block w-full p-2 text-gray-900 border border-gray-300 rounded-lg bg-gray-50 text-xs focus:ring-indigo-500 focus:border-indigo-500'
                                                >
                                                    {allPrefixes.map((prefix, n) => (
                                                        <option key={n} value={prefix}>
                                                            {Prefix[prefix]}
                                                        </option>
                                                    ))}
                                                </select>
                                            </td>
                                            <td className='w-48 p-2'>
                                                <input
                                                    className='block w-full p-2 text-gray-900 border border-gray-300 rounded-lg bg-gray-50 text-xs focus:ring-indigo-500 focus:border-indigo-500'
                                                    value={aliases[id]}
                                                    onChange={e => {
                                                        setAliases({
                                                            ...aliases,
                                                            [id]: e.target.value,
                                                        });
                                                    }}
                                                />
                                            </td>
                                        </tr>
                                    ))}
                                </tbody>
                            </table>
                        </div>
                    )}
                </Modal.Body>
                <Modal.Footer className='flex flex-row w-100'>
                    <Button
                        outline
                        color='indigo'
                        theme={{
                            color: {
                                indigo: 'bg-indigo-600 text-white ring-indigo-600',
                            },
                        }}
                        onClick={handlePreviousButtonClick}
                    >
                        {modalPage === ModalPage.BASIC ? 'Cancel' : 'Previous'}
                    </Button>
                    <div className='grow'></div>
                    <Button
                        color='indigo'
                        disabled={
                            modalPage === ModalPage.SIGNALS &&
                            Object.values(quantities).length === 0
                        }
                        theme={{
                            color: {
                                indigo: 'bg-indigo-600 text-white ring-indigo-600',
                            },
                        }}
                        onClick={handleNextButtonClick}
                    >
                        {modalPage === ModalPage.INGEST ? 'Update' : 'Next'}
                    </Button>
                </Modal.Footer>
            </Modal>
        </>
    );
}

export default UpdateSensorModal;
