'use client';

import dynamic from "next/dynamic"
import { Twin, TwinContext } from '@/store/twins';
import { useContext, useState } from 'react';
import { Accordion, Label, TextInput, Button } from 'flowbite-react';

interface PredictionMapProps {
    twin: Twin;
};

function PredictionPage() {
    const [twinState, dispatch] = useContext(TwinContext);

    // TODO: put simulation config into a separate component to prevent map re-rendering
    const [hourOfDay, setHourOfDay] = useState(0);
    const [dayOfYear, setDayOfYear] = useState(0);
    const [kwhPrice, setKwhPrice] = useState(0);

    const [temperature, setTemperature] = useState(0);
    const [rainfall, setRainfall] = useState(0);

    if (!twinState.current) {
        return <h1>Please select a Twin</h1>
    }

    // TODO: look for a better way to import PredictionMap, will likely cause problems
    const PredictionMap = dynamic<PredictionMapProps>(() => import("../../../components/maps/PredictionMap"), { ssr: false });

    return (
        <div className="flex flex-col h-full">
            <div>
                <span></span>
            </div>
            <div className="grid grid-cols-12 grow">
                <div className="col-span-9">
                    <PredictionMap twin={twinState.current} />
                </div>
                <div className="col-span-3 mx-6">
                    <div className="flex flex-col h-full">
                        <div className="grow">
                            <span className="mb-2 text-2xl leading-none text-bold tracking-tight text-gray-900 md:text-4xl lg:text-4xl dark:text-white">Simulation Configuration</span>
                            <br />
                            <span className="mb-2 text-1xl leading-none tracking-tight text-gray-600 md:text-3xl lg:text-3xl dark:text-white">{twinState.current.name}</span>
                            <div className="m-3 bg-white">
                                <Accordion>
                                    <Accordion.Panel>
                                        <Accordion.Title>General</Accordion.Title>
                                        <Accordion.Content>
                                            <div>
                                                <Label value="Hour of Day" />
                                                <TextInput value={hourOfDay} type="number" min={0} max={23} required onChange={e => setHourOfDay(parseFloat(e.target.value))} />
                                            </div>
                                            <div>
                                                <Label value="Day of Year" />
                                                <TextInput value={dayOfYear} type="number" min={0} max={364} required onChange={e => setDayOfYear(parseFloat(e.target.value))} />
                                            </div>
                                            <div>
                                                <Label value="Price of Electricity (gas)" />
                                                <TextInput value={kwhPrice} type="number" required onChange={e => setKwhPrice(parseFloat(e.target.value))} />
                                            </div>
                                        </Accordion.Content>
                                    </Accordion.Panel>
                                    <Accordion.Panel>
                                        <Accordion.Title>Weather</Accordion.Title>
                                        <Accordion.Content>
                                            <div>
                                                <Label value="Temperature (celcius)" />
                                                <TextInput value={temperature} type="number" required onChange={e => setTemperature(parseFloat(e.target.value))} />
                                            </div>
                                            <div>
                                                <Label value="Rainfall (mm)" />
                                                <TextInput value={rainfall} type="number" required onChange={e => setRainfall(parseFloat(e.target.value))} />
                                            </div>
                                        </Accordion.Content>
                                    </Accordion.Panel>
                                    <Accordion.Panel>
                                        <Accordion.Title>Population</Accordion.Title>
                                        <Accordion.Content>
                                            <p className="mb-2 text-gray-500 dark:text-gray-400">
                                                Population Settings
                                            </p>
                                        </Accordion.Content>
                                    </Accordion.Panel>
                                </Accordion>
                            </div>
                        </div>
                        <div>
                            <Button fullSized color="blue">Run Simulation</Button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}

export default PredictionPage
