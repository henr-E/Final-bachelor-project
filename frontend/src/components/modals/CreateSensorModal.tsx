'use client';

import { useContext, useState, useRef } from 'react';
import {
    Sensor,
    SensorContext,
    Quantity,
    Signal,
    quantityBaseUnits,
    quantities as allQuantities,
    prefixes as allPrefixes,
    Unit,
    Prefix
} from '@/store/sensor';
import { v4 as uuidv4 } from 'uuid';
import {
    Button,
    Modal,
    Label,
    ButtonGroup,
    Table
} from 'flowbite-react';
import { HiXMark } from 'react-icons/hi2';

interface CreateSensorModalProps {
    isModalOpen: boolean;
    closeModal: () => void;
}

enum ModalPage {
    BASIC,
    SIGNALS,
    INGEST
}

function CreateSensorModal({ isModalOpen, closeModal }: CreateSensorModalProps) {
    const [sensorState, dispatchSensor] = useContext(SensorContext);

    const [modalPage, setModalPage] = useState<ModalPage>(ModalPage.BASIC);

    // step 1: general settings
    const [name, setName] = useState("");
    const [description, setDescription] = useState("");

    // step 2: quantities
    const [quantity, setQuantity] = useState<Quantity>(Quantity.TEMPERATURE);
    const [quantities, setQuantities] = useState<Quantity[]>([]);

    // step 3: signals
    const [units, setUnits] = useState<Unit[]>([]);
    const [prefixes, setPrefixes] = useState<Prefix[]>([]);
    const [aliases, setAliases] = useState<string[]>([]);

    const basicFormRef = useRef<HTMLFormElement>(null);

    const handleModalClose = () => {
        setModalPage(ModalPage.BASIC);
        setName("");
        setDescription("");
        setQuantity(Quantity.TEMPERATURE);
        setQuantities([]);
        setUnits([]);
        setPrefixes([]);
        setAliases([]);

        closeModal();
    }

    const handleNextButtonClick = () => {
        switch (modalPage) {
            case ModalPage.BASIC: {
                if (!basicFormRef.current?.reportValidity()) return;
                setModalPage(modalPage + 1);
                break;
            }
            case ModalPage.SIGNALS: {
                setModalPage(modalPage + 1);

                setUnits(quantities.map(q => quantityBaseUnits[q]));
                setPrefixes(quantities.map(q => Prefix.ONE));
                setAliases(quantities.map(q => `${Quantity[q]}-N`));

                break;
            }
            case ModalPage.INGEST: {
                setModalPage(0);

                const sensor: Sensor = {
                    id: uuidv4(),
                    name: name,
                    description: description,
                    location: { lat: 51, lng: 4.1 },
                    signals: quantities.map((q, i) => ({
                        quantity: q,
                        unit: quantityBaseUnits[q],
                        ingestionUnit: units[i],
                        ingestionPrefix: prefixes[i],
                        ingestionColumnAlias: aliases[i]
                    }))
                };

                dispatchSensor({ type: 'create_sensor', sensor });

                handleModalClose();
                break;
            }
        }
    }

    const handlePreviousButtonClick = () => {
        setModalPage(modalPage - 1);

        if (modalPage === ModalPage.BASIC) {
            setName("");
            setDescription("");
            setQuantity(Quantity.TEMPERATURE);

            handleModalClose();
        }
    }

    const CreateSensorStepper = ({ page }: { page: ModalPage }) => {
        const activeStyles = "text-indigo-600";

        return <ol className="flex items-center w-full text-sm font-medium text-center text-gray-500 sm:text-base mb-8">
            <li className={`flex md:w-full items-center ${activeStyles} sm:after:content-[''] after:w-full after:h-1 after:border-b after:border-gray-200 after:border-1 after:hidden sm:after:inline-block after:mx-6 xl:after:mx-10`}>
                <span className="flex items-center after:content-['/'] sm:after:hidden after:mx-2 after:text-gray-200">
                    <svg className="w-3.5 h-3.5 sm:w-4 sm:h-4 me-2.5" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="currentColor" viewBox="0 0 20 20">
                        <path d="M10 .5a9.5 9.5 0 1 0 9.5 9.5A9.51 9.51 0 0 0 10 .5Zm3.707 8.207-4 4a1 1 0 0 1-1.414 0l-2-2a1 1 0 0 1 1.414-1.414L9 10.586l3.293-3.293a1 1 0 0 1 1.414 1.414Z" />
                    </svg>
                    <span className={page === ModalPage.BASIC ? '' : ''}>General</span>
                </span>
            </li>
            <li className={`flex md:w-full items-center ${page === ModalPage.INGEST || page === ModalPage.SIGNALS ? activeStyles : ''} after:content-[''] after:w-full after:h-1 after:border-b after:border-gray-200 after:border-1 after:hidden sm:after:inline-block after:mx-6 xl:after:mx-10`}>
                <span className={`flex items-center after:content-['/'] sm:after:hidden after:mx-2 after:text-gray-200`}>
                    <svg className="w-3.5 h-3.5 sm:w-4 sm:h-4 me-2.5" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="currentColor" viewBox="0 0 20 20">
                        <path d="M10 .5a9.5 9.5 0 1 0 9.5 9.5A9.51 9.51 0 0 0 10 .5Zm3.707 8.207-4 4a1 1 0 0 1-1.414 0l-2-2a1 1 0 0 1 1.414-1.414L9 10.586l3.293-3.293a1 1 0 0 1 1.414 1.414Z" />
                    </svg>
                    Signals
                </span>
            </li>
            <li className={`flex items-center ${page === ModalPage.INGEST ? activeStyles : ''}`}>
                <svg className="w-3.5 h-3.5 sm:w-4 sm:h-4 me-2.5" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="currentColor" viewBox="0 0 20 20">
                    <path d="M10 .5a9.5 9.5 0 1 0 9.5 9.5A9.51 9.51 0 0 0 10 .5Zm3.707 8.207-4 4a1 1 0 0 1-1.414 0l-2-2a1 1 0 0 1 1.414-1.414L9 10.586l3.293-3.293a1 1 0 0 1 1.414 1.414Z" />
                </svg>
                Format
            </li>
        </ol>
    }

    const handleAddSignalButtonClick = () => {
        setQuantities([...quantities, quantity]);
    }

    return (
        <>
            <Modal show={isModalOpen} size={modalPage === ModalPage.INGEST ? '4xl' : 'xl'} onClose={handleModalClose}>
                <Modal.Header>Create Sensor</Modal.Header>
                <Modal.Body>
                    <CreateSensorStepper page={modalPage} />
                    {
                        modalPage === ModalPage.BASIC &&
                        <div className="my-4">
                            <form ref={basicFormRef}>
                                <div>
                                    <div className="mb-2 block">
                                        <Label htmlFor="name" value="Name" />
                                    </div>
                                    <input id="name" className="bg-gray-50 border border-gray-300 text-gray-900 rounded-lg text-sm focus:ring-indigo-500 w-full focus:border-indigo-500 p-2.5" type="text" value={name} placeholder="name" required maxLength={50} onChange={(e) => setName(e.target.value)} style={{ marginBottom: '10px' }} />
                                </div>
                                <div>
                                    <div className="mb-2 block">
                                        <Label htmlFor="description" value="Description" />
                                    </div>
                                    <input id="description" className="bg-gray-50 border border-gray-300 text-gray-900 rounded-lg text-sm focus:ring-indigo-500 w-full focus:border-indigo-500 p-2.5" type="text" value={description} placeholder={"description"} maxLength={200} required onChange={(e) => setDescription(e.target.value)} />
                                </div>
                            </form>
                        </div>
                    }
                    {
                        modalPage === ModalPage.SIGNALS && <div className="flex flex-col space-y-4">
                            <Label>What does this sensor measure?</Label>
                            <ButtonGroup className="">
                                <select id="quantity" className="bg-gray-50 border border-gray-300 text-gray-900 rounded-l-lg text-sm focus:ring-indigo-500 focus:border-indigo-500 p-2.5" value={quantity} onChange={e => setQuantity(parseInt(e.target.value))} required>
                                    {
                                        allQuantities.map((q, i) => <option key={i} value={i}>{Quantity[q]}</option>)
                                    }
                                </select>
                                <button className="px-3 py-2 text-sm font-medium text-white rounded-r-lg bg-indigo-600" onClick={handleAddSignalButtonClick}>Add Signal</button>
                            </ButtonGroup>
                            <div className="w-100 min-h-32 p-2 bg-indigo-200 rounded-xl shadow-md flex flex-row flex-wrap items-center justify-center gap-2">
                                {
                                    quantities.map((quantity, i) => <Button
                                        key={i}
                                        color="indigo"
                                        onClick={() => setQuantities([...quantities.slice(0, i), ...quantities.slice(i + 1)])}
                                        theme={{ color: { indigo: 'bg-indigo-600 text-white ring-indigo-600' } }} pill
                                    >
                                        {Quantity[quantity]}
                                        <HiXMark className="ml-2 text-gray-200" size={20} />
                                    </Button>
                                    )
                                }
                                {
                                    quantities.length === 0 && <span className="text-sm text-gray-500">No Signals</span>
                                }
                            </div>
                        </div>
                    }
                    {
                        modalPage === ModalPage.INGEST && <div className="flex flex-col space-y-6">
                            <Label>What is the format of the measured data?</Label>
                            <table className="text-sm text-left rtl:text-right text-gray-800 w-full table-auto">
                                <thead className="border-gray-600 text-xs uppercase">
                                    <tr>
                                        <th scope="col" className="w-32 px-2">Signal</th>
                                        <th scope="col" className="w-32 px-2">Unit</th>
                                        <th scope="col" className="w-24 px-2">Prefix</th>
                                        <th scope="col" className="w-48 px-2">Alias</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {
                                        quantities.map((quantity, i) => <tr key={i}>
                                            <td className="w-32 p-2"><span>{Quantity[quantity]}</span></td>
                                            <td className="w-48 p-2">
                                                <select
                                                    value={units[i]}
                                                    className="block w-full p-2 text-gray-900 border border-gray-300 rounded-lg bg-gray-50 text-xs focus:ring-indigo-500 focus:border-indigo-500">
                                                    <option value={quantityBaseUnits[quantity]}>{Unit[quantityBaseUnits[quantity]]}</option>
                                                </select>
                                            </td>
                                            <td className="w-24 p-2">
                                                <select
                                                    value={prefixes[i]}
                                                    onChange={e => setPrefixes([...prefixes.slice(0, i), parseInt(e.target.value), ...prefixes.slice(i + 1)])}
                                                    className="block w-full p-2 text-gray-900 border border-gray-300 rounded-lg bg-gray-50 text-xs focus:ring-indigo-500 focus:border-indigo-500">
                                                    {allPrefixes.map((prefix, i) => <option key={i} value={prefix}>{Prefix[prefix]}</option>)}
                                                </select>
                                            </td>
                                            <td className="w-48 p-2">
                                                <input
                                                    className="block w-full p-2 text-gray-900 border border-gray-300 rounded-lg bg-gray-50 text-xs focus:ring-indigo-500 focus:border-indigo-500"
                                                    value={aliases[i]}
                                                    onChange={e => setAliases([...aliases.slice(0, i), e.target.value, ...aliases.slice(i + 1)])}
                                                />
                                            </td>
                                        </tr>
                                        )
                                    }
                                </tbody>
                            </table>
                        </div>

                    }
                </Modal.Body>
                <Modal.Footer className="flex flex-row w-100">
                    <Button
                        outline
                        color="indigo"
                        theme={{ color: { indigo: 'bg-indigo-600 text-white ring-indigo-600' } }}
                        onClick={handlePreviousButtonClick}
                    >
                        {modalPage === ModalPage.BASIC ? "Cancel" : "Previous"}
                    </Button>
                    <div className="grow"></div>
                    <Button
                        color="indigo"
                        disabled={modalPage === ModalPage.SIGNALS && quantities.length === 0}
                        theme={{ color: { indigo: 'bg-indigo-600 text-white ring-indigo-600' } }}
                        onClick={handleNextButtonClick}
                    >
                        {modalPage === ModalPage.INGEST ? "Create" : "Next"}
                    </Button>
                </Modal.Footer>
            </Modal>
        </>
    );
}

export default CreateSensorModal;

