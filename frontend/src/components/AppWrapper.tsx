'use client';
import 'react-toastify/dist/ReactToastify.css';


import ToastNotification, {ToastContainer} from "@/components/notification/ToastNotification";



function AppWrapper({ children }: { children: React.ReactNode }) {



    return (
        <>
            <ToastContainer />
            {children}
        </>
    );
}

export default AppWrapper;
