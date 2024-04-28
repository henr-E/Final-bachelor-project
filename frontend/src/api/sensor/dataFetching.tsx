import { Channel, WebsocketTransport, createChannel, createClient } from 'nice-grpc-web';
import { uiBackendServiceUrl } from '@/api/urls';
import { isAbortError } from 'abort-controller-x';
import { SensorDataFetchingServiceDefinition } from '@/proto/sensor/data-fetching';
import { isAssetError } from 'next/dist/client/route-loader';

export async function* LiveDataSingleSensor(
    sensorId: string
): AsyncGenerator<{ signalId: number; value: number }> {
    let serverUrl = uiBackendServiceUrl;
    if (uiBackendServiceUrl.slice(0, 4) !== 'http') {
        serverUrl = window.location.origin;
    }
    const abortController = new AbortController();
    let channel: Channel | undefined = createChannel(serverUrl, WebsocketTransport());
    const client = createClient(SensorDataFetchingServiceDefinition, channel);

    for await (const entry of client.fetchSensorDataSingleSensorStream(
        {
            sensorId: sensorId,
            // Default lookback of 20 seconds. This means that at launch values
            // from 20 seconds back will also be fetched. After that values
            // come in live.
            lookback: 20,
        },
        {}
    )) {
        if (window.location.pathname !== '/dashboard/realtime') break;
        console.debug(entry);
        for (const [signalId, { value: valueObj }] of Object.entries(entry.signals)) {
            const value = valueObj.at(-1)?.value;
            if (value === undefined) {
                continue;
            }

            const { exponent, sign, integer } = value;

            yield {
                signalId: Number(signalId),
                value:
                    (sign ? -1 : 1) *
                    integer.reduce((acc, i) => (acc << 32) + i, 0) *
                    Math.pow(10, exponent),
            };
        }
    }
    abortController.abort();
}
