import React from 'react';
import 'react-toastify/dist/ReactToastify.css';
import { toast, ToastContainer, Slide } from 'react-toastify';

type ToastType = 'success' | 'error' | 'warning' | 'info';

function ToastNotification(type: ToastType, message: string): void {
    switch (type) {
        case 'success':
            toast.success(message, {
                position: 'top-right',
                autoClose: 2000,
                hideProgressBar: false,
                closeOnClick: true,
                pauseOnHover: true,
                draggable: true,
                progress: undefined,
                theme: 'light',
            });
            break;
        case 'error':
            toast.error(message, {
                position: 'top-right',
                autoClose: 3000,
                hideProgressBar: false,
                closeOnClick: true,
                pauseOnHover: true,
                draggable: false,
                progress: undefined,
                theme: 'light',
            });
            break;
        case 'warning':
            toast.warn(message, {
                position: 'top-right',
                autoClose: 3000,
                hideProgressBar: false,
                closeOnClick: true,
                pauseOnHover: true,
                draggable: false,
                progress: undefined,
                theme: 'light',
            });
            break;
        case 'info':
            toast.info(message, {
                position: 'top-right',
                autoClose: 3000,
                hideProgressBar: false,
                closeOnClick: true,
                pauseOnHover: true,
                draggable: false,
                progress: undefined,
                theme: 'light',
            });
            break;
    }
}

export default ToastNotification;
