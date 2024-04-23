'use client';
import dynamic from 'next/dynamic';

function EditorPage() {
    const MapEditor = dynamic(() => import('@/components/maps/MapEditor'), {
        ssr: false,
    });
    return (
        <div className='h-screen w-full'>
            <div style={{ height: '75%' }}>
                <MapEditor></MapEditor>
            </div>
        </div>
    );
}

export default EditorPage;
