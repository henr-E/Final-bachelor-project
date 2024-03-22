'use client';


import { ToastContainer } from 'react-toastify';


function AppWrapper({ children }: { children: React.ReactNode }) {



    return (
        <>
            <ToastContainer />
            {children}
        </>
    );
}

export default AppWrapper;
