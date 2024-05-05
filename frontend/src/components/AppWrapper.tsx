'use client';
import 'react-toastify/dist/ReactToastify.css';
import ToastNotification, { ToastContainer } from '@/components/notification/ToastNotification';
import { UserFromProvider, UserContext } from '@/store/user';
import { useContext, useEffect } from 'react';
import { jwtDecode, InvalidTokenError } from 'jwt-decode';
import { usePathname, useRouter } from 'next/navigation';

function AppWrapper({ children }: { children: React.ReactNode }) {
    const [userState, dispatchUser] = useContext(UserContext);

    const router = useRouter();
    const pathName = usePathname();

    useEffect(() => {
        if (!userState.token) {
            const token = localStorage.getItem('authToken');

            if (token) {
                try {
                    const decoded = jwtDecode(token) as any;

                    const user: UserFromProvider = {
                        username: decoded.username,
                    };

                    dispatchUser({ type: 'login', token, user });
                } catch (e) {
                    localStorage.removeItem('authToken');
                    router.replace('/');
                }
            } else if (pathName.startsWith('/dashboard')) {
                router.replace('/');
            }
        }
    }, [dispatchUser, userState.token, userState.user, pathName, router]);
    return (
        <>
            <ToastContainer />
            {children}
        </>
    );
}

export default AppWrapper;
