'use client';

import {Button, Label, Modal, TextInput} from 'flowbite-react';
import {useState} from 'react';
import {jwtDecode} from 'jwt-decode';
import {useRouter} from 'next/navigation';
import {login} from '@/lib/authentication';
import {setCookie} from 'typescript-cookie';
import ToastNotification from "@/components/notification/ToastNotification";


interface LoginModalProps {
    isLoginModalOpen: boolean;
    closeLoginModal: () => void;
}

function LoginModal({isLoginModalOpen, closeLoginModal}: LoginModalProps) {
    const router = useRouter();

    const [username, setUsername] = useState('');
    const [password, setPassword] = useState('');


    const handleSubmit = async () => {
        let formdata = new FormData();
        formdata.append('username', username);
        formdata.append('password', password);

        const response = await login(formdata);
        if (response.ok) {
            // set the jwt token in the cookie
            const token = response.val;
            const decoded = jwtDecode(token);
            const expiration_date = decoded.exp;
            setCookie('auth', token, {expires: expiration_date});
            ToastNotification('success', 'welcome ' + username);
            router.push('/dashboard');
            closeLoginModal();
        } else {
            ToastNotification('error', response.val.message);
            return;
        }
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
                    <Modal.Body>
                        <div>
                            <div className='mb-2 block'>
                                <Label htmlFor='username' value='Your username'/>
                            </div>
                            <TextInput
                                id='username'
                                type='username'
                                value={username}
                                placeholder='username'
                                required
                                onChange={e => setUsername(e.target.value)}
                                style={{marginBottom: '10px'}}
                            />
                        </div>
                        <div>
                            <div className='mb-2 block'>
                                <Label htmlFor='password' value='Your password'/>
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
                        <Button color='indigo' type='submit'>
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
