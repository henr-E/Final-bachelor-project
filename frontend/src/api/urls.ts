const userServiceUrl = process.env.NEXT_PUBLIC_USER_SERVICE_URL || '';
const sensorServiceUrl = process.env.NEXT_PUBLIC_SENSOR_SERVICE_URL || '';
const twinServiceUrl = process.env.NEXT_PUBLIC_TWIN_SERVICE_URL || '';

let uiBackendServiceUrl = process.env.NEXT_PUBLIC_GEO_SERVICE_URL || '';

if (process.env.NODE_ENV === 'test') {
    if (!uiBackendServiceUrl) {
        uiBackendServiceUrl = 'http://127.0.0.1:8081';
    }
}

// ADD BACKEND URLs PER SERVICE

export { userServiceUrl, sensorServiceUrl, twinServiceUrl, uiBackendServiceUrl };
