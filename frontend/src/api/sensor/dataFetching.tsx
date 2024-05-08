import { Channel, WebsocketTransport, createChannel, createClient } from 'nice-grpc-web';
import { uiBackendServiceUrl } from '@/api/urls';
import {
    SensorDataFetchingServiceDefinition,
    SingleSensorDataMessage,
} from '@/proto/sensor/data-fetching';
import { clientAuthLayer } from '@/api/protecteRequestFactory';

export async function* LiveDataSingleSensor(
    sensorId: string,
    abortSignal: AbortSignal | undefined
): AsyncGenerator<{ signalId: number; value: number }> {
    let serverUrl = uiBackendServiceUrl;
    if (serverUrl.slice(0, 4) !== 'http') {
        serverUrl = window.location.origin;
    }
    const channel: Channel | undefined = createChannel(serverUrl, WebsocketTransport());
    const client = clientAuthLayer.create(SensorDataFetchingServiceDefinition, channel);

    const stream = async function* (
        signal: AbortSignal | undefined
    ): AsyncIterable<SingleSensorDataMessage> {
        // Send the initial request message to the server.
        yield {
            request: {
                sensorId: sensorId,
                // Default lookback of 20 seconds. This means that at launch values
                // from 20 seconds back will also be fetched. After that values
                // come in live.
                lookback: 20,
            },
        };

        // Create a promise that can be waited for and is only resolved if the
        // `sendShutdownSignal` function is called. This (weird) code avoids
        // having to loop and check for a value here, making the waiting as
        // performant as possible.
        let sendShutdownSignal = (_: any) => {};
        const waitForShutdown = new Promise(resolve => {
            sendShutdownSignal = resolve;
        });

        // Register the shutdown dispatch with the abort controller so that the
        // promise resolves when abort is called.
        signal?.addEventListener('abort', () => {
            sendShutdownSignal({});
        });

        await waitForShutdown;

        // Send the shutdown signal to the server.
        yield {
            shutdown: {},
        };
    };

    // NOTE: The abortSignal is not passed to the actual request here. This is
    // because for bidirectional streams the closing of the websocket is
    // somehow done correctly (it is not for server streams). The signal is
    // used to send a shutdown signal to the server.
    for await (const entry of client.fetchSensorDataSingleSensorStream(stream(abortSignal))) {
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
}
