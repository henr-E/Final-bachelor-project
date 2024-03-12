'use client';

import { User, UserContext } from "@/store/user";
import { useContext, useEffect } from "react";
import { ToastContainer } from 'react-toastify';

function AppWrapper({ children }: { children: React.ReactNode }) {
    const [userState, dispatchUser] = useContext(UserContext);

    useEffect(() => {
        if (!userState.token && !userState.user) {
            const token = localStorage.getItem("authToken");

            // TODO: replace with response of API call to fetch user data (endpoint such as /me)
            const user: User = {
                username: 'SOME USERNAME'
            }

            if (token) {
                dispatchUser({ type: 'login', token, user });
            }
        }
    }, [dispatchUser, userState.token, userState.user]);

    return (
        <>
            <ToastContainer />
            {children}
        </>
    );
}

export default AppWrapper;
