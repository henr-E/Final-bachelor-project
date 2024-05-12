import React from 'react';
import { toast, Toaster } from 'react-hot-toast';

type ToastType = 'success' | 'error' | 'warning' | 'info';

function ToastNotification(
    type: ToastType,
    message: string,
    onClickAction: () => void = () => {}
): void {
    const options = {
        duration: 4000, // Equivalent to autoClose
        // React Hot Toast does not use a hideProgressBar option; progress bars are not a built-in feature
        iconTheme: {
            primary: '#000', // Icon color
            secondary: '#fff', // Icon background color; adjust as needed for light/dark themes
        },
    };

    switch (type) {
        case 'success':
            toast(<div onClick={onClickAction}>{message}</div>, {
                ...options,
                style: { background: '#80D05A', color: '#fff' },
            }); // Custom styling for warning; adjust as needed
            break;
        case 'error':
            toast(message, { ...options, style: { background: '#EC5A53', color: '#fff' } }); // Custom styling for warning; adjust as needed
            break;
        case 'warning':
            toast(message, { ...options, style: { background: '#f1c40f', color: '#fff' } }); // Custom styling for warning; adjust as needed
            break;
        case 'info':
            toast(message, {
                ...options,
                icon: 'ℹ️',
            });
    }
}

// This component could be included at the top level of your app, typically inside the component that wraps your app's content.
export function ToastContainer() {
    const toastContainerStyles = {
        zIndex: 100001, //yes, this should be 100001 because reactour tutorial has zIndex 100000
    };
    return (
        <Toaster
            position='bottom-right'
            reverseOrder={false}
            containerStyle={toastContainerStyles}
        />
    );
}

export default ToastNotification;
