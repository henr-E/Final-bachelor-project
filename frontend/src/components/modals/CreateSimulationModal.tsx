'use client';

import { useContext, useRef, useState } from 'react';
import { Button, Datepicker, Label, Modal, Select, TextInput } from 'flowbite-react';
import dynamic from 'next/dynamic';
import { CreateSimulationParams } from '@/proto/simulation/frontend';
import { TwinContext } from '@/store/twins';
import { LineItem, NodeItem } from '@/components/maps/MapItem';
import { Edge, Graph, Node, State } from '@/proto/simulation/simulation';
import ToastNotification from '@/components/notification/ToastNotification';
import { BackendCreateSimulation, BackendGetSimulations } from '@/api/simulation/crud';
import CustomJsonEditor from '@/components/CustomJsonEditor';

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
    const [name, setName] = useState<string>(propItems.title || '');
    const [startDate, setStartDate] = useState<Date>(propItems.startDate || new Date(Date.now()));
    const [endDate, setEndDate] = useState<Date>(propItems.endDate || new Date(Date.now()));
    const [startTime, setStartTime] = useState<string>(propItems.startTime || '');
    const [endTime, setEndTime] = useState<string>(propItems.endTime || '');
    const [timeStepDelta, setTimeStepDelta] = useState<number>(propItems.timeStepDelta || 0);
    const [step, setStep] = useState<number>(0);
    const nodeItemsRef = useRef<Map<number, NodeItem>>();
    const edgeItemsRef = useRef<Array<LineItem>>();

    const [globalComponents, setGlobalComponents] = useState(
        propItems.globalComponents || '{"global_temperature":{"current_temp":15}}'
    );

    const formRef = useRef<HTMLFormElement>(null);

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

        let globalComponentsObject: { [key: string]: any | undefined } =
            JSON.parse(globalComponents);
        globalComponentsObject['global_time'] = globalComponentsObject['global_time'] || {
            // Save start time in the global_time component, only if the component does not already
            // exist.
            unix_timestamp_millis: startDateTime.getTime(),
        };

        console.log(nodes);
        console.log(edges);
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
                globalComponents: globalComponentsObject,
            }),
            timeStepDelta: timeStepDelta,
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

    const closeModelAndReset = () => {
        setStep(0);
        setName(propItems.title || '');
        setStartDate(propItems.startDate || new Date());
        setEndDate(propItems.endDate || new Date());
        setStartTime(propItems.startTime || '00:00:00');
        setEndTime(propItems.endTime || '00:00:00');
        setTimeStepDelta(propItems.timeStepDelta || 1);
        setGlobalComponents(propItems.globalComponents || '{}');
        propItems.closeModal();
    };

    const handleCancelButtonClick = () => {
        if (step > 0) {
            setStep(step - 1);
            return;
        }
        closeModelAndReset();
    };

    const NextStepButtonClick = () => {
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
                    ToastNotification('warning', 'A simulation with this name already exists.');
                    return;
                }
            }
        }

        if (step == 1) {
            closeModelAndReset();
            GenerateSimulation().then();
            return;
        }
        if (!formRef.current?.checkValidity()) {
            formRef.current?.reportValidity();
            return;
        }

        try {
            JSON.parse(globalComponents);
        } catch (e) {
            ToastNotification('error', 'Not a valid json format for global vars');
            return;
        }

        setStep(step + 1);
    };

    const MapEditor = dynamic(() => import('@/components/maps/MapEditor'), {
        ssr: false,
    });

    const onJsonChange = (key: string, value: any, parent: any, data: JsonData) => {
        console.log(key, value, parent, data);
    };
    return (
        <>
            <Modal
                show={propItems.isModalOpen}
                onClose={closeModelAndReset}
                size={step === 0 ? 'xl' : ''}
                className='flex flex-row'
            >
                <Modal.Header>Create simulation</Modal.Header>
                {step == 0 ? (
                    <Modal.Body>
                        <form ref={formRef}>
                            <div className='flex flex-row w-full space-x-3 pt-3'>
                                <div className='basis-1/2'>
                                    <div className='mb-2 block'>
                                        <Label htmlFor='name' value='Name' />
                                    </div>
                                    <TextInput
                                        id='name'
                                        type='text'
                                        value={name}
                                        placeholder='name'
                                        required
                                        maxLength={50}
                                        onChange={e => setName(e.target.value)}
                                        style={{ marginBottom: '10px' }}
                                    />
                                </div>
                                <div className='basis-1/2'>
                                    <div className='mb-2 block'>
                                        <Label htmlFor='simualtion' value='Simualtion type' />
                                    </div>
                                    <Select id='simualtion' required>
                                        <option>Time simulator</option>
                                    </Select>
                                </div>
                            </div>
                            <div className='flex flex-row w-full space-x-3 pt-3'>
                                <div className='basis-1/2'>
                                    <div className='mb-2 block'>
                                        <Label htmlFor='starttime' value='Start time (hh:mm:ss)' />
                                    </div>
                                    <TextInput
                                        id='starttime'
                                        type='text'
                                        value={startTime}
                                        placeholder={'00:00:00'}
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
                                    <TextInput
                                        id='endtime'
                                        type='text'
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
                            <div className='flex flex-row w-full space-x-3 pt-3'>
                                <div className='basis-1/2'>
                                    <div className='mb-2 block'>
                                        <Label
                                            htmlFor='timestepdelta'
                                            value='Timestep delta (seconds)'
                                        />
                                    </div>
                                    <TextInput
                                        id='timesteps'
                                        value={timeStepDelta}
                                        placeholder={'0'}
                                        maxLength={200}
                                        required
                                        type='number'
                                        onChange={e => setTimeStepDelta(+e.target.value)}
                                    />
                                </div>
                            </div>
                            <div className='flex flex-row w-full space-x-3 pt-3'>
                                <div className='w-full'>
                                    <CustomJsonEditor
                                        data={JSON.parse(globalComponents)}
                                        onSave={updatedComponents => {
                                            ToastNotification(
                                                'success',
                                                'global components updated'
                                            );
                                            setGlobalComponents(JSON.stringify(updatedComponents));
                                        }}
                                    />
                                </div>

                                {/*<div className="w-full">*/}
                                {/*    <div className="mb-2 block">*/}
                                {/*        <Label htmlFor="gv" value="Global variables" />*/}
                                {/*    </div>*/}
                                {/*    <Textarea*/}
                                {/*        id="gv"*/}
                                {/*        placeholder="{}"*/}
                                {/*        required*/}
                                {/*        rows={4}*/}
                                {/*        value={globalComponents}*/}
                                {/*        onChange={e => setGlobalComponents(e.target.value)}*/}
                                {/*    />*/}
                                {/*</div>*/}
                            </div>
                        </form>
                    </Modal.Body>
                ) : (
                    <Modal.Body style={{ overflowY: 'hidden' }}>
                        <div className='h-screen w-full'>
                            <div style={{ height: '65%' }}>
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
                    <Button
                        outline
                        color='indigo'
                        theme={{
                            color: {
                                indigo: 'bg-indigo-600 text-white ring-indigo-600',
                            },
                        }}
                        onClick={handleCancelButtonClick}
                    >
                        {step == 0 ? 'Cancel' : 'Previous'}
                    </Button>
                    <div className='grow'></div>
                    <Button
                        color='indigo'
                        theme={{
                            color: {
                                indigo: 'bg-indigo-600 text-white ring-indigo-600',
                            },
                        }}
                        onClick={NextStepButtonClick}
                    >
                        {step == 1 ? 'Create' : 'Next'}
                    </Button>
                </Modal.Footer>
            </Modal>
        </>
    );
}

export default CreateSimulationModal;
