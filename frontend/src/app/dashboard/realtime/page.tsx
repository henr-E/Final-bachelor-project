import MapDataRequest from "@/app/dashboard/realtime/mapDataRequest";
import StringListRequest from "@/app/dashboard/realtime/stringRequest";

function RealtimePage() {
    return (
        <>
            <p> map data:</p>
            <MapDataRequest></MapDataRequest>

            <h1>..............................</h1>


            <p> string data:</p>
            <StringListRequest></StringListRequest>
        </>
    );
}

export default RealtimePage
