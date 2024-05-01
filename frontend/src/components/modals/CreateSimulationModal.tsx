'use client';
import { useContext, useState, useRef, useEffect } from 'react';
import { Button, Modal, Label, TextInput, Datepicker, Table, Checkbox } from 'flowbite-react';
import dynamic from 'next/dynamic';
import { CreateSimulationParams } from '@/proto/simulation/frontend';
import { SimulatorInfo } from '@/proto/simulation/simulation-manager';
import { TwinContext } from '@/store/twins';
import { LineItem, NodeItem } from '@/components/maps/MapItem';
import { Edge, Graph, Node, State } from '@/proto/simulation/simulation';
import ToastNotification from '@/components/notification/ToastNotification';
import {
    BackendCreateSimulation,
    BackendGetComponent,
    BackendGetSimulations,
    BackendGetSimulators,
} from '@/api/simulation/crud';
import CustomJsonEditor, { TypeConverter } from '@/components/CustomJsonEditor';
import { undefined } from 'zod';

interface CreateSimulationModalProps {
    isModalOpen: boolean;
    closeModal: () => void;
    title?: string;
    startDate?: Date;
    endDate?: Date;
    startTime?: string;
    endTime?: string;
    timeStepDelta?: number;
    globalComponents?: string;
    initialNodes?: Map<number, NodeItem>;
    initialEdges?: Array<LineItem>;
    simulators?: string[];
}

enum ModalPage {
    SIMULATION,
    SIMULATORS,
    GLOBALCOMPONENTS,
    GLOBAL,
}

interface JsonData {
    the: string;
    that: string;
    on: string;
    moon: string;
    maybe: number;
    i: string;
    probably: string[];
    am_i_right: boolean;
}

function CreateSimulationModal(propItems: CreateSimulationModalProps) {
    const [twinState, dispatchTwin] = useContext(TwinContext);
    const [modalPage, setModalPage] = useState<ModalPage>(ModalPage.SIMULATION);
    const [name, setName] = useState<string>(propItems.title || '');
    const [startDate, setStartDate] = useState<Date>(propItems.startDate || new Date(Date.now()));
    const [endDate, setEndDate] = useState<Date>(propItems.endDate || new Date(Date.now()));
    const [startTime, setStartTime] = useState<string>(propItems.startTime || '');
    const [endTime, setEndTime] = useState<string>(propItems.endTime || '');
    const [timeStepDelta, setTimeStepDelta] = useState<number>(propItems.timeStepDelta || 0);
    const [globalComponents, setGlobalComponents] = useState(propItems.globalComponents || '{}');
    const [simulatorsSelected, setSimulatorsSelected] = useState<string[]>([]);
    const [simulators, setSimulators] = useState<SimulatorInfo[]>();
    const nodeItemsRef = useRef<Map<number, NodeItem>>();
    const edgeItemsRef = useRef<Array<LineItem>>();
    const formRef = useRef<HTMLFormElement>(null);

    useEffect(() => {
        async function fetchSimulators() {
            setSimulators((await BackendGetSimulators()).simulator);
        }
        fetchSimulators().then();
    }, []);

    const GenerateSimulation = async () => {
        const startTimeSplit = startTime.split(':');
        let startDateTime = startDate;
        startDateTime.setHours(+startTimeSplit[0], +startTimeSplit[1], +startTimeSplit[2]);

        const endTimeSplit = endTime.split(':');
        let endDateTime = endDate;
        endDateTime.setHours(+endTimeSplit[0], +endTimeSplit[1], +endTimeSplit[2]);

        //Convert map edges and nodes to simulation edges and nodes
        let nodes = new Array<Node>();
        Array.from(nodeItemsRef.current?.values() ? nodeItemsRef.current?.values() : []).map(
            item => {
                let markerItem = item as NodeItem;
                nodes.push(
                    Node.create({
                        components: markerItem.components,
                        id: item.id,
                        // @ts-ignore
                        longitude: markerItem.location[1],
                        // @ts-ignore
                        latitude: markerItem.location[0],
                    })
                );
            }
        );

        let edges = new Array<Edge>();
        edgeItemsRef.current?.map(item => {
            //Convert multiple components to multiple edges for simulators, will be converted back in the interface in the MapFrame page.
            Object.entries(item.components || {}).forEach(([keyItem, value]) => {
                let lineItem = item as LineItem;
                let component: { [id: string]: any } = {};
                component[keyItem] = value;
                edges.push(
                    Edge.create({
                        id: lineItem.id,
                        from: lineItem.items[0].id,
                        // @ts-ignore
                        to: lineItem.items[1].id,
                        componentType: keyItem,
                        componentData: component,
                    })
                );
            });
            return;
        });

        if (!twinState.current) {
            ToastNotification('error', 'Select a twin');
            return;
        }

        const twin: CreateSimulationParams = {
            name: name,
            twinId: twinState.current?.id.toString(),
            // Division by 1000 to convert to seconds
            startDateTime: startDateTime.getTime() / 1000,
            endDateTime: endDateTime.getTime() / 1000,
            startState: State.create({
                graph: Graph.create({
                    nodes: nodes,
                    edge: edges,
                }),
                globalComponents: JSON.parse(globalComponents),
            }),
            timeStepDelta: timeStepDelta,
            simulators: {
                name: simulatorsSelected,
            },
        };

        const response = await BackendCreateSimulation(twin);
        if (!response.success) {
            ToastNotification('error', 'Could not create simulation, try again');
        } else {
            ToastNotification('success', `Simulation \"${name}\" is created`);
            let simulations = await BackendGetSimulations(String(twinState.current?.id));
            dispatchTwin({ type: 'load_simulations', simulations: simulations.item });
            propItems.closeModal();
        }
    };

    const getComponentGlobalComponents = async () => {
        try {
            let availableComponents = new Array<string>();

            simulators?.map(item => {
                if (simulatorsSelected.includes(item.name)) {
                    availableComponents = availableComponents.concat(item.outputComponents);
                }
            });
            let newGlobalComponents = JSON.parse(globalComponents);
            const response = await BackendGetComponent();
            const components = response.components;
            for (let componentName in components) {
                if (components.hasOwnProperty(componentName)) {
                    let componentSpec = components[componentName];
                    console.log(componentName, componentSpec);
                    if (
                        componentSpec.type === 2 &&
                        !newGlobalComponents[componentName] &&
                        availableComponents?.includes(componentName)
                    ) {
                        newGlobalComponents[componentName] = TypeConverter(componentSpec.structure);
                    }
                }
            }
            console.log(newGlobalComponents);
            setGlobalComponents(JSON.stringify(newGlobalComponents));
            ToastNotification('success', 'Global components loaded');
        } catch (error) {
            console.error('Error fetching components:', error);
            ToastNotification('error', 'Failed to load global components');
        }
    };

    const handleBackButtonClick = async () => {
        if (modalPage == ModalPage.GLOBALCOMPONENTS) {
            setGlobalComponents(propItems.globalComponents || '{}');
        }
        setModalPage(modalPage - 1);
    };

    const handleNextButtonClick = async () => {
        switch (modalPage) {
            case ModalPage.SIMULATION: {
                if (!formRef.current?.reportValidity()) return;
                if (startDate > endDate) {
                    ToastNotification('warning', 'Start date must be before/equal end date.');
                    return;
                }
                if (startDate.getTime() === endDate.getTime() && startTime >= endTime) {
                    ToastNotification(
                        'warning',
                        'Start time must be before end time when dates are the same.'
                    );
                    return;
                }
                if (twinState.current) {
                    for (let i = 0; i < twinState.current.simulations.length; i++) {
                        if (twinState.current.simulations[i].name == name) {
                            ToastNotification(
                                'warning',
                                'A simulation with this name already exists.'
                            );
                            return;
                        }
                    }
                }
                if (!formRef.current?.checkValidity()) {
                    formRef.current?.reportValidity();
                    return;
                }

                const startTimeSplit = startTime.split(':');
                let startDateTime = startDate;
                startDateTime.setHours(+startTimeSplit[0], +startTimeSplit[1], +startTimeSplit[2]);

                let globalComponentsObject: { [key: string]: any | undefined } =
                    JSON.parse(globalComponents);
                globalComponentsObject['global_time'] = globalComponentsObject['global_time'] || {
                    // Save start time in the global_time component, only if the component does not already
                    // exist.
                    unix_timestamp_millis: startDateTime.getTime(),
                };
                setGlobalComponents(JSON.stringify(globalComponentsObject));
                setModalPage(ModalPage.SIMULATORS);
                return;
            }
            case ModalPage.SIMULATORS: {
                if (simulatorsSelected.length == 0) {
                    ToastNotification('warning', 'Must select at least one simulator!');
                    return;
                }
                setModalPage(ModalPage.GLOBALCOMPONENTS);
                await getComponentGlobalComponents();
                return;
            }
            case ModalPage.GLOBALCOMPONENTS: {
                if (simulatorsSelected.length == 0) {
                    ToastNotification('warning', 'Must select at least one simulator!');
                    return;
                }
                setModalPage(ModalPage.GLOBAL);
                return;
            }
            case ModalPage.GLOBAL: {
                closeModelAndReset();
                GenerateSimulation().then();
                return;
            }
        }
    };
    const handlePreviousButtonClick = () => {
        switch (modalPage) {
            case ModalPage.SIMULATION: {
                closeModelAndReset();
                break;
            }
            case ModalPage.SIMULATORS: {
                setModalPage(ModalPage.SIMULATION);
                return;
            }
            case ModalPage.GLOBAL: {
                setModalPage(ModalPage.SIMULATION);
                return;
            }
        }
    };
    const closeModelAndReset = () => {
        setModalPage(ModalPage.SIMULATION);
        setName(propItems.title || '');
        setStartDate(propItems.startDate || new Date());
        setEndDate(propItems.endDate || new Date());
        setStartTime(propItems.startTime || '00:00:00');
        setEndTime(propItems.endTime || '00:00:00');
        setTimeStepDelta(propItems.timeStepDelta || 1);
        setGlobalComponents(propItems.globalComponents || '{}');
        setSimulatorsSelected([]);
        propItems.closeModal();
    };

    const MapEditor = dynamic(() => import('@/components/maps/MapEditor'), {
        ssr: false,
    });

    const onJsonChange = (key: string, value: any, parent: any, data: JsonData) => {
        console.log(key, value, parent, data);
    };
    const CreateSimulationStepper = () => {
        const activeStyles = 'text-indigo-600';
        return (
            <ol className='flex items-center w-full text-sm font-medium text-center text-gray-500 sm:text-base mb-8'>
                <li
                    className={`flex md:w-full items-center ${activeStyles}  after:w-full after:h-1 after:border-b after:border-gray-200 after:border-1 after:hidden sm:after:inline-block after:mx-6 xl:after:mx-10`}
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
                        <span className={modalPage === ModalPage.SIMULATION ? '' : ''}>
                            Simulation
                        </span>
                    </span>
                </li>
                <li
                    className={`flex md:w-full items-center ${
                        modalPage === ModalPage.SIMULATORS || modalPage === ModalPage.GLOBAL
                            ? activeStyles
                            : ''
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
                        Simulators
                    </span>
                </li>
                <li
                    className={`flex md:w-full items-center ${
                        modalPage === ModalPage.GLOBALCOMPONENTS || modalPage === ModalPage.GLOBAL
                            ? activeStyles
                            : ''
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
                        Components
                    </span>
                </li>
                <li
                    className={`flex items-center ${
                        modalPage === ModalPage.GLOBAL ? activeStyles : ''
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
                        Global
                    </span>
                </li>
            </ol>
        );
    };

    return (
        <>
            <Modal
                show={propItems.isModalOpen}
                onClose={closeModelAndReset}
                size={modalPage !== ModalPage.GLOBAL ? '4xl' : ''}
            >
                <Modal.Header>Create simulation</Modal.Header>
                {modalPage === ModalPage.SIMULATION && (
                    <Modal.Body>
                        <CreateSimulationStepper />
                        <form ref={formRef}>
                            <div className='flex flex-row w-full space-x-3 pt-3'>
                                <div className='basis-1/2'>
                                    <div className='mb-2 block'>
                                        <Label htmlFor='name' value='Name' />
                                    </div>
                                    <input
                                        id='name'
                                        type='text'
                                        value={name}
                                        className='bg-gray-50 border border-gray-300 text-gray-900 rounded-lg text-sm focus:ring-indigo-500 w-full focus:border-indigo-500 p-2.5'
                                        placeholder='name'
                                        required
                                        maxLength={50}
                                        onChange={e => setName(e.target.value)}
                                        style={{ marginBottom: '10px' }}
                                    />
                                </div>
                                <div className='basis-1/2'>
                                    <div className='mb-2 block'>
                                        <Label
                                            htmlFor='timestepdelta'
                                            value='Timestep delta (seconds)'
                                        />
                                    </div>
                                    <input
                                        id='timesteps'
                                        value={timeStepDelta}
                                        placeholder={'1'}
                                        className='bg-gray-50 border border-gray-300 text-gray-900 rounded-lg text-sm focus:ring-indigo-500 w-full focus:border-indigo-500 p-2.5'
                                        min={1}
                                        maxLength={200}
                                        required
                                        type='number'
                                        onChange={e => setTimeStepDelta(+e.target.value)}
                                    />
                                </div>
                            </div>
                            <div className='flex flex-row w-full space-x-3 pt-3'>
                                <div className='basis-1/2'>
                                    <div className='mb-2 block'>
                                        <Label htmlFor='starttime' value='Start time (hh:mm:ss)' />
                                    </div>
                                    <input
                                        id='starttime'
                                        type='text'
                                        value={startTime}
                                        placeholder={'00:00:00'}
                                        className='bg-gray-50 border border-gray-300 text-gray-900 rounded-lg text-sm focus:ring-indigo-500 w-full focus:border-indigo-500 p-2.5'
                                        pattern='\d{2}:\d{2}:\d{2}'
                                        maxLength={200}
                                        required
                                        onChange={e => setStartTime(e.target.value)}
                                    />
                                </div>
                                <div className='basis-1/2'>
                                    <div className='mb-2 block'>
                                        <Label htmlFor='startdate' value='Start date' />
                                    </div>
                                    <Datepicker
                                        id='startdate'
                                        style={{ zIndex: 50 }}
                                        defaultDate={startDate}
                                        required
                                        onSelectedDateChanged={date => setStartDate(date)}
                                    />
                                </div>
                            </div>
                            <div className='flex flex-row w-full space-x-3 pt-3'>
                                <div className='basis-1/2'>
                                    <div className='mb-2 block'>
                                        <Label htmlFor='endtime' value='End time (hh:mm:ss)' />
                                    </div>
                                    <input
                                        id='endtime'
                                        type='text'
                                        className='bg-gray-50 border border-gray-300 text-gray-900 rounded-lg text-sm focus:ring-indigo-500 w-full focus:border-indigo-500 p-2.5'
                                        value={endTime}
                                        placeholder={'00:00:00'}
                                        pattern='\d{2}:\d{2}:\d{2}'
                                        maxLength={200}
                                        required
                                        onChange={e => setEndTime(e.target.value)}
                                    />
                                </div>
                                <div className='basis-1/2'>
                                    <div className='mb-2 block'>
                                        <Label htmlFor='enddate' value='End date' />
                                    </div>
                                    <Datepicker
                                        id='enddate'
                                        style={{ zIndex: 50 }}
                                        defaultDate={endDate}
                                        required
                                        onSelectedDateChanged={date => setEndDate(date)}
                                    />
                                </div>
                            </div>
                        </form>
                    </Modal.Body>
                )}
                {modalPage === ModalPage.SIMULATORS && (
                    <Modal.Body>
                        <div className='overflow-x-auto'>
                            <Table hoverable>
                                <Table.Head>
                                    <Table.HeadCell className='p-4'>
                                        <Checkbox
                                            id='checkbox-all-search'
                                            checked={
                                                simulatorsSelected.length === simulators?.length
                                            }
                                            onChange={e => {
                                                if (simulators && simulators?.length > 0) {
                                                    if (
                                                        simulatorsSelected.length ===
                                                        simulators.length
                                                    ) {
                                                        setSimulatorsSelected([]);
                                                    } else {
                                                        setSimulatorsSelected(
                                                            simulators.map(sim => sim.name)
                                                        );
                                                    }
                                                }
                                            }}
                                            className='w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded focus:ring-blue-500 focus:ring-2 dark:bg-gray-700'
                                        />
                                    </Table.HeadCell>
                                    <Table.HeadCell>SIMULATOR NAME</Table.HeadCell>
                                    <Table.HeadCell>OUTPUT COMPONENTS</Table.HeadCell>
                                </Table.Head>
                                <Table.Body className='divide-y'>
                                    {simulators &&
                                        simulators.map(simulator => (
                                            <Table.Row key={simulator.name}>
                                                <Table.Cell>
                                                    <Checkbox
                                                        id='checkbox'
                                                        checked={simulatorsSelected.includes(
                                                            simulator.name
                                                        )}
                                                        onChange={e => {
                                                            if (
                                                                simulatorsSelected.includes(
                                                                    simulator.name
                                                                )
                                                            ) {
                                                                setSimulatorsSelected(
                                                                    simulatorsSelected.filter(
                                                                        s => s !== simulator.name
                                                                    )
                                                                );
                                                            } else {
                                                                setSimulatorsSelected(
                                                                    simulatorsSelected.concat([
                                                                        simulator.name,
                                                                    ])
                                                                );
                                                            }
                                                        }}
                                                        className='w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded focus:ring-blue-500 focus:ring-2 dark:bg-gray-700'
                                                    />
                                                </Table.Cell>
                                                <Table.Cell>{simulator.name}</Table.Cell>
                                                <Table.Cell>
                                                    {simulator.outputComponents.join(', ')}{' '}
                                                    {/*Assuming simulator.outputComponents is an array*/}
                                                </Table.Cell>
                                            </Table.Row>
                                        ))}
                                </Table.Body>
                            </Table>
                        </div>
                    </Modal.Body>
                )}
                {modalPage == ModalPage.GLOBALCOMPONENTS && (
                    <Modal.Body>
                        <div className='flex flex-row w-full space-x-3 pt-3'>
                            <div className='w-full'>
                                <CustomJsonEditor
                                    data={JSON.parse(globalComponents)}
                                    onSave={updatedComponents => {
                                        ToastNotification('success', 'global components updated');
                                        setGlobalComponents(JSON.stringify(updatedComponents));
                                    }}
                                />
                            </div>
                        </div>
                    </Modal.Body>
                )}
                {modalPage == ModalPage.GLOBAL && (
                    <Modal.Body style={{ overflowY: 'hidden' }}>
                        <div className='h-screen w-full'>
                            <div style={{ height: '60%' }}>
                                <MapEditor
                                    nodeItemRef={nodeItemsRef}
                                    edgeItemRef={edgeItemsRef}
                                    initialNodes={propItems.initialNodes}
                                    initialEdges={propItems.initialEdges}
                                ></MapEditor>
                            </div>
                        </div>
                    </Modal.Body>
                )}
                <Modal.Footer className='flex flex-row w-100'>
                    <div className='grow'></div>
                    {modalPage != ModalPage.SIMULATION && (
                        <Button
                            outline
                            color='indigo'
                            theme={{
                                color: {
                                    indigo: 'bg-indigo-600 text-white ring-indigo-600',
                                },
                            }}
                            onClick={handleBackButtonClick}
                        >
                            Previous
                        </Button>
                    )}
                    <Button
                        color='indigo'
                        theme={{
                            color: {
                                indigo: 'bg-indigo-600 text-white ring-indigo-600',
                            },
                        }}
                        onClick={handleNextButtonClick}
                    >
                        {modalPage == ModalPage.GLOBAL ? 'Create' : 'Next'}
                    </Button>
                </Modal.Footer>
            </Modal>
        </>
    );
}

export default CreateSimulationModal;
