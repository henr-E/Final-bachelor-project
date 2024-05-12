'use client';
import { PredictionMapProps } from '@/components/maps/PredictionMap';
import { useContext, useEffect, useState } from 'react';
import { TwinContext } from '@/store/twins';
import dynamic from 'next/dynamic';
import { QuantityWithUnits, SensorContext } from '@/store/sensor';
import { Dropdown, DropdownItem } from 'flowbite-react';
import { BackendGetQuantityWithUnits } from '@/api/sensor/crud';
import { LiveDataAllSensor } from '@/api/sensor/dataFetching';
import { getFirstQuantity } from '@/lib/util';
import { ToggleSwitch } from 'flowbite-react';
import { isAbortError } from 'abort-controller-x';
import { TourControlContext } from '@/store/tour';

const PredictionMapImport = dynamic<PredictionMapProps>(
    () => import('@/components/maps/PredictionMap'),
    { ssr: false }
);

async function* mockLoadSensorStream(): AsyncGenerator<{ signalId: number; value: number }> {
    await new Promise((resolve, reject) => {
        setTimeout(() => resolve(0), 1000 * Math.random() + 500);
    });

    yield { signalId: 0, value: Math.random() * 10000 };
}

function RealTimePage() {
    const [twinState, dispatch] = useContext(TwinContext);
    const [sensorContext, dispatchSensor] = useContext(SensorContext);
    const tourController = useContext(TourControlContext);
    const [showLabels, setShowLabels] = useState(true);

    const [quantitiesWithUnits, setQuantitiesWithUnits] = useState<
        Record<string, QuantityWithUnits>
    >({});
    const [quantityFilter, setQuantityFilter] = useState<string | undefined>(
        getFirstQuantity(quantitiesWithUnits)?.quantity.id
    );

    useEffect(() => {
        (async () => {
            const quantitiesWithUnits = await BackendGetQuantityWithUnits();
            setQuantitiesWithUnits(quantitiesWithUnits);
            setQuantityFilter(getFirstQuantity(quantitiesWithUnits)?.quantity.id);
        })();
    }, []);

    useEffect(() => {
        const abortController = new AbortController();
        (async () => {
            if (twinState.current) {
                const stream = LiveDataAllSensor(abortController.signal);
                try {
                    const sensorIds = new Set(twinState.current.sensors.map(s => s.id));
                    for await (const val of stream) {
                        console.log(
                            `DISPATCHED set_most_recent_value USING VALUES: ${val.sensorId} ${val.signalId} ${val.value}`
                        );
                        if (!sensorIds.has(val.sensorId)) {
                            continue;
                        }
                        dispatchSensor({
                            type: 'set_most_recent_value',
                            ...val,
                        });
                    }
                } catch (err) {
                    if (!isAbortError(err)) {
                        throw err;
                    }
                }
            }
        })();

        return () => {
            console.debug('Closing sensor data streams.');
            abortController.abort();
            console.debug('Streams closed.');
        };
    }, [twinState, twinState.current?.sensors, dispatchSensor]);

    if (!twinState.current) {
        return <h1>Please select a Twin</h1>;
    }

    return (
        <div className='space-y-2 h-full'>
            <div style={{ height: '94%' }}>
                <PredictionMapImport
                    twin={twinState.current}
                    realtime
                    showLabels={showLabels}
                    quantityFilter={quantityFilter}
                />
            </div>
            <div
                className='tour-step-0-realtime flex items-center'
                onClick={() => {
                    tourController?.customGoToNextTourStep(1);
                }}
            >
                <Dropdown
                    pill
                    color='indigo'
                    theme={{
                        floating: {
                            target: 'enabled:hover:bg-indigo-700 bg-indigo-600 text-white',
                        },
                    }}
                    label={quantityFilter ?? 'All Quantities'}
                    dismissOnClick
                >
                    <div className='tour-step-1-realtime'>
                        {Object.keys(quantitiesWithUnits).map(quantity => (
                            <DropdownItem
                                key={quantity}
                                onClick={() => setQuantityFilter(quantity)}
                            >
                                {quantity}
                            </DropdownItem>
                        ))}
                    </div>
                </Dropdown>
                <div className='pl-5'>
                    <ToggleSwitch
                        label={'Show values'}
                        checked={showLabels}
                        onChange={setShowLabels}
                        color={'indigo'}
                    />
                </div>
            </div>
        </div>
    );
}

export default RealTimePage;
