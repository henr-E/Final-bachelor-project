'use client';

import { Button, Modal, Label, TextInput } from 'flowbite-react';
import { useContext, useState } from 'react';
import { jwtDecode } from 'jwt-decode';
import { useRouter } from 'next/navigation';
import { login } from '@/lib/authentication';
import ToastNotification from '@/components/notification/ToastNotification';
import { UserContext } from '@/store/user';
import { TourControlContext } from '@/store/tour';

interface LoginModalProps {
    isLoginModalOpen: boolean;
    closeLoginModal: () => void;
}

function LoginModal({ isLoginModalOpen, closeLoginModal }: LoginModalProps) {
    const router = useRouter();

    const [userState, dispatchUser] = useContext(UserContext);

    const [username, setUsername] = useState('');
    const [password, setPassword] = useState('');
    const tourController = useContext(TourControlContext);

    const handleSubmit = async () => {
        let formdata = new FormData();
        formdata.append('username', username);
        formdata.append('password', password);

        const response = await login(formdata);
        if (response.ok) {
            const token = response.val;
            const decoded = jwtDecode(token) as any;
            localStorage.setItem('authToken', token);

            ToastNotification('success', 'welcome ' + username);
            dispatchUser({ type: 'login', token, user: { username: decoded.username } });
            router.push('/dashboard');
            closeLoginModal();
        } else {
            ToastNotification('error', response.val.message);
            return;
        }
        tourController?.setIsOpen(false);
    };

    const setToDefault = () => {
        setPassword('');
        setUsername('');
    };

    const handleCancelButtonClick = () => {
        setToDefault();
        closeLoginModal();
    };

    return (
        <>
            <Modal show={isLoginModalOpen} onClose={closeLoginModal}>
                <form action={handleSubmit}>
                    <Modal.Header>Login</Modal.Header>
                    <Modal.Body className={'tour-step-6-startup'}>
                        <div>
                            <div className='mb-2 block'>
                                <Label htmlFor='username' value='Your username' />
                            </div>
                            <TextInput
                                id='username'
                                type='username'
                                value={username}
                                placeholder='username'
                                required
                                onChange={e => setUsername(e.target.value)}
                                style={{ marginBottom: '10px' }}
                            />
                        </div>
                        <div>
                            <div className='mb-2 block'>
                                <Label htmlFor='password' value='Your password' />
                            </div>
                            <TextInput
                                id='password'
                                type='password'
                                value={password}
                                placeholder={'password'}
                                required
                                onChange={e => setPassword(e.target.value)}
                            />
                        </div>
                    </Modal.Body>
                    <Modal.Footer>
                        <Button className={'tour-step-7-startup'} color='indigo' type='submit'>
                            Login
                        </Button>
                        <Button color='gray' onClick={handleCancelButtonClick}>
                            Cancel
                        </Button>
                    </Modal.Footer>
                </form>
            </Modal>
        </>
    );
}

export default LoginModal;
