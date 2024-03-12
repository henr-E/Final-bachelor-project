'use client';
import {PredictionMapProps} from "@/components/maps/PredictionMap";
import {useContext} from "react";
import {TwinContext} from "@/store/twins";
import dynamic from "next/dynamic";
import {PredictionMapMode} from "@/app/dashboard/GlobalVariables";

const PredictionMapImport = dynamic<PredictionMapProps>(() => import("@/components/maps/PredictionMap"), {ssr: false});

function RealTimePage() {
    const [twinState, dispatch] = useContext(TwinContext);

    if (!twinState.current) {
        return <h1>Please select a Twin</h1>
    }

    return (
        <>
            <PredictionMapImport twin={twinState.current} mode={PredictionMapMode.RealtimeMode}/>
        </>
    )
}

export default RealTimePage
