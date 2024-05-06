'use client';
import { PredictionMapProps } from '@/components/maps/PredictionMap';
import { useContext, useEffect, useState } from 'react';
import { TwinContext } from '@/store/twins';
import dynamic from 'next/dynamic';
import { sensorServiceUrl } from '@/api/urls';
import { createChannel, createClient, WebsocketTransport } from 'nice-grpc-web';
import { QuantityWithUnits, SensorContext } from '@/store/sensor';
import { Button, Dropdown, DropdownItem } from 'flowbite-react';
import { BackendGetQuantityWithUnits } from '@/api/sensor/crud';
import { SensorDataFetchingServiceDefinition } from '@/proto/sensor/data-fetching';
import { LiveDataSingleSensor } from '@/api/sensor/dataFetching';
import { getFirstQuantity } from '@/lib/util';
import { useRouter } from 'next/router';
import { isAbortError } from 'abort-controller-x';

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
        if (twinState.current) {
            twinState.current.sensors.forEach(async sensor => {
                const stream = LiveDataSingleSensor(sensor.id, abortController.signal);

                try {
                    for await (const val of stream) {
                        console.log(
                            `DISPATCHED set_most_recent_value USING VALUES: ${sensor.id} ${val.signalId} ${val.value}`
                        );
                        dispatchSensor({
                            type: 'set_most_recent_value',
                            sensorId: sensor.id,
                            ...val,
                        });
                    }
                } catch (err) {
                    if (!isAbortError(err)) {
                        throw err;
                    }
                }
            });
        }

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
                    quantityFilter={quantityFilter}
                />
            </div>
            <div>
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
                    {Object.keys(quantitiesWithUnits).map(quantity => (
                        <DropdownItem key={quantity} onClick={() => setQuantityFilter(quantity)}>
                            {quantity}
                        </DropdownItem>
                    ))}
                </Dropdown>
            </div>
        </div>
    );
}

export default RealTimePage;
