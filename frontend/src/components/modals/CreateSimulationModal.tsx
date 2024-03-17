'use client';

import { useContext, useState, useRef } from 'react';
import {
    Button,
    Modal,
    Label,
    TextInput,
    Select,
    Datepicker, Toast
} from 'flowbite-react';
import dynamic from "next/dynamic";
import {createChannel, createClient} from "nice-grpc-web";
import {
    CreateSimulationResponse,
    SimulationInterfaceServiceDefinition,
    Simulations,
    CreateSimulationParams
} from "@/proto/simulation/frontend";
import {TwinContext} from "@/store/twins";
import {LineItem, MapItemType, MarkerItem} from "@/components/maps/MapItem";
import {Edge, Node} from "@/proto/simulation/simulation";
import ToastNotification from "@/components/notification/ToastNotification";
import {uiBackendServiceUrl} from "@/api/urls";

interface CreateSimulationModalProps {
    isModalOpen: boolean;
    closeModal: () => void;
}

function CreateSimulationModal({ isModalOpen, closeModal }: CreateSimulationModalProps) {
    const [twinState, dispatch] = useContext(TwinContext);
    const [name, setName] = useState<string>("");
    const [startDate, setStartDate] = useState<Date>(new Date(Date.now()));
    const [endDate, setEndDate] = useState<Date>(new Date(Date.now()));
    const [startTime, setStartTime] = useState<string>("");
    const [endTime, setEndTime] = useState<string>("");
    const [timeStepDelta, setTimeStepDelta] = useState<number>(0);
    const [step, setStep] = useState<number>(0);
    const mapItemsRef = useRef<Array<MapItemType>>();

    const formRef = useRef<HTMLFormElement>(null);

    const CreateSimulation = async () => {
        console.log(startTime, startDate);
        const startTimeSplit = startTime.split(":");
        let startDateTime = startDate;
        startDateTime.setHours(+startTimeSplit[0], +startTimeSplit[1], +startTimeSplit[2]);

        const endTimeSplit = endTime.split(":");
        let endDateTime = endDate;
        endDateTime.setHours(+endTimeSplit[0], +endTimeSplit[1], +endTimeSplit[2]);

        //Convert map edges and nodes to simulation edges and nodes
        let nodes = new Array<Node>;
        let edges = new Array<Edge>;
        mapItemsRef.current?.map(item => {
            console.log(item)
            if (item.type == 3) {
                let lineItem = item as LineItem;
                edges.push(Edge.create({
                    id: lineItem.id,
                    from: lineItem.items[0].id,
                    // @ts-ignore
                    to: lineItem.items[1].id,
                    componentType: "Edge",
                    componentData: {}
                }));
                return;
            }
            let markerItem = item as MarkerItem;
            nodes.push(Node.create({
                components: {},
                id: item.id,
                // @ts-ignore
                longitude: markerItem.location.lng,
                // @ts-ignore
                latitude: markerItem.location.lat
            }))

        });
        if (!twinState.current) {
            ToastNotification("error", "Select a twin");
            return;
        }

        const twin: CreateSimulationParams = {
            name: name,
            twinId: twinState.current?.id.toString(),
            //division by 1000 to convert to ms
            startDateTime: startDateTime.getTime()/1000,
            endDateTime: endDateTime.getTime()/1000,
            graph: {
                nodes: nodes,
                edge: edges
            },
            timeStepDelta: timeStepDelta
        };

        const channel = createChannel(uiBackendServiceUrl);
        const client = createClient(SimulationInterfaceServiceDefinition, channel);

        const response = await client.createSimulation(twin);
        if (!response.success) {
            ToastNotification("error", "Could not create simulation, try again");
        }
        ToastNotification("success", `Simulation \"${name}\" is created`);
        closeModal();
    }

    const closeModel = () => {
        setStep(0);
        setName("");
        closeModal();
    }

    const handleCancelButtonClick = () => {
        if(step > 0){
            setStep(step - 1)
            return;
        }
    }

    const NextStepButtonClick = () => {
        if(step == 1){
            CreateSimulation().then();
            return;
        }
        if (!formRef.current?.checkValidity()) {
            formRef.current?.reportValidity();
            return;
        }
        setStep(step+1);
    }

    function dateTimeChange(e: any){
        console.log(e);
        setEndDate(e.target.valueAsDate? e.target.valueAsDate: new Date());
    }

    const MapEditor = dynamic(() => import("@/components/maps/MapEditor"), { ssr: false });

    return (
        <>
            <Modal show={isModalOpen} onClose={closeModal} size={step === 0? "xl": ""} className="flex flex-row">
                <Modal.Header>Create simulation</Modal.Header>
                {
                    step == 0?
                        <Modal.Body>
                            <form ref={formRef}>
                                <div className="flex flex-row w-full space-x-3 pt-3">
                                    <div className="basis-1/2">
                                        <div className="mb-2 block">
                                            <Label htmlFor="name" value="Name"/>
                                        </div>
                                        <TextInput id="name" type="text" value={name} placeholder="name" required
                                                   maxLength={50} onChange={(e) => setName(e.target.value)}
                                                   style={{marginBottom: '10px'}}/>
                                    </div>
                                    <div className="basis-1/2">
                                        <div className="mb-2 block">
                                            <Label htmlFor="simualtion" value="Simualtion type"/>
                                        </div>
                                        <Select id="simualtion" required>
                                            <option>Time simulator</option>
                                        </Select>
                                    </div>
                                </div>
                                <div className="flex flex-row w-full space-x-3 pt-3">
                                    <div className="basis-1/2">
                                        <div className="mb-2 block">
                                            <Label htmlFor="starttime" value="Start time (hh:mm:ss)"/>
                                        </div>
                                        <TextInput id="starttime" type="text" value={startTime} placeholder={"00:00:00"}
                                                   pattern="\d{2}:\d{2}:\d{2}"
                                                   maxLength={200} required
                                                   onChange={(e) => setStartTime(e.target.value)}/>
                                    </div>
                                    <div className="basis-1/2">
                                        <div className="mb-2 block">
                                            <Label htmlFor="startdate" value="Start date"/>
                                        </div>
                                        <Datepicker id="startdate" style={{zIndex: 50}} required
                                                    onSelectedDateChanged={(date) => setStartDate(date)}/>
                                    </div>
                                </div>
                                <div className="flex flex-row w-full space-x-3 pt-3">
                                    <div className="basis-1/2">
                                        <div className="mb-2 block">
                                            <Label htmlFor="endtime" value="End time (hh:mm:ss)"/>
                                        </div>
                                        <TextInput id="endtime" type="text" value={endTime} placeholder={"00:00:00"} pattern="\d{2}:\d{2}:\d{2}"
                                                   maxLength={200} required
                                                   onChange={(e) => setEndTime(e.target.value)}/>
                                    </div>
                                    <div className="basis-1/2">
                                        <div className="mb-2 block">
                                            <Label htmlFor="enddate" value="End date"/>
                                        </div>
                                        <Datepicker id="enddate" style={{zIndex: 50}} required
                                                    onSelectedDateChanged={(date) => setEndDate(date)}/>
                                    </div>
                                </div>
                                <div className="flex flex-row w-full space-x-3 pt-3">
                                    <div className="basis-1/2">
                                        <div className="mb-2 block">
                                            <Label htmlFor="timestepdelta" value="Timestep delta (seconds)"/>
                                        </div>
                                        <TextInput id="timesteps" value={timeStepDelta} placeholder={"0"}
                                                   maxLength={200} required
                                                   type="number"
                                                   onChange={(e) => setTimeStepDelta(+e.target.value)}/>
                                    </div>
                                </div>
                            </form>
                        </Modal.Body> :
                        <Modal.Body>
                            <div className="h-screen w-full">
                                <div style={{height: "65%"}}>
                                    <MapEditor mapItemRef={mapItemsRef}></MapEditor>
                                </div>
                            </div>
                        </Modal.Body>
                }
                <Modal.Footer className="flex flex-row w-100">
                    <Button
                        outline
                        color="indigo"
                        theme={{color: {indigo: 'bg-indigo-600 text-white ring-indigo-600'}}}
                        onClick={handleCancelButtonClick}
                    >
                        {step == 0 ? "Cancel" : "Previous"}
                    </Button>
                    <div className="grow"></div>
                    <Button
                        color="indigo"
                        theme={{color: { indigo: 'bg-indigo-600 text-white ring-indigo-600' } }}
                        onClick={NextStepButtonClick}
                    >
                        {step == 1 ? "Create" : "Next"}
                    </Button>
                </Modal.Footer>
            </Modal>
        </>
    );
}

export default CreateSimulationModal;

