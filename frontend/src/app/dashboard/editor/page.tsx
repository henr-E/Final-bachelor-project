'use client';
import dynamic from 'next/dynamic';

function EditorPage() {
    const MapEditor = dynamic(() => import('@/components/maps/MapEditor'), {
        ssr: false,
    });
    return (
        <div style={{ height: '75%' }}>
            <MapEditor></MapEditor>
        </div>
    );
}

export default EditorPage;
