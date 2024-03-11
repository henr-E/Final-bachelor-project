const userServiceUrl = process.env.NEXT_PUBLIC_USER_SERVICE_URL || "USER SERVICE URL";
const sensorServiceUrl = process.env.NEXT_PUBLIC_SENSOR_SERVICE_URL || "SENSOR SERVICE URL";
const twinServiceUrl = process.env.NEXT_PUBLIC_TWIN_SERVICE_URL || "YOUR TWIN SERVICE URL";

const geoServiceUrl = process.env.NEXT_PUBLIC_GEO_SERVICE_URL || "http://127.0.0.1:8081";


// ADD BACKEND URLs PER SERVICE

export {
    userServiceUrl,
    sensorServiceUrl,
    twinServiceUrl,
    geoServiceUrl
};
