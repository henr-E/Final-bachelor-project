'use client';
import dynamic from 'next/dynamic';

function EditorPage() {
    const MapEditor = dynamic(() => import('@/components/maps/MapEditor'), {
        ssr: false,
    });
    return (
        <div className='flex flex-col h-full'>
            <MapEditor></MapEditor>
        </div>
    );
}

export default EditorPage;
