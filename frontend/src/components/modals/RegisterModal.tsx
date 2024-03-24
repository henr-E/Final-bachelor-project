'use client';

import { Button, Modal, Label, TextInput } from 'flowbite-react';

import { useState } from 'react';
import { register } from '@/lib/authentication';
import { FaUser } from 'react-icons/fa';
import { RiLockPasswordFill } from 'react-icons/ri';
import { removeCookie } from 'typescript-cookie';
import ToastNotification from '@/components/notification/ToastNotification';

interface RegisterModalProps {
    isRegisterModalOpen: boolean;
    closeRegisterModal: () => void;
}

export function RegisterModal({ isRegisterModalOpen, closeRegisterModal }: RegisterModalProps) {
    const [username, setUsername] = useState('');
    const [password, setPassword] = useState('');
    const [confirmPassword, setConfirmPassword] = useState('');

    const validatePassword = () => {
        const hasNumber = /\d/.test(password);
        const hasCapitalLetter = /[A-Z]/.test(password);
        const hasSpecialChar = /[^A-Za-z0-9]/.test(password);
        if (password !== confirmPassword) {
            ToastNotification('error', "password don't match");
            return false;
        }
        if (password.length < 8) {
            ToastNotification('error', 'Password must be at least 8 characters long');
            return false;
        }
        if (username.length < 3) {
            ToastNotification('error', 'username must be at least 3 characters long');
        }
        if (!hasNumber) {
            ToastNotification('error', 'Password must include at least one number');
            return false;
        }
        if (!hasCapitalLetter) {
            ToastNotification('error', 'Password must include at least one uppercase letter');
            return false;
        }
        if (!hasSpecialChar) {
            ToastNotification('error', 'Password must include at least one special character');
            return false;
        }
        return true;
    };

    const handleSubmit = async () => {
        if (!validatePassword()) {
            return;
        }

        let formdata = new FormData();

        formdata.append('username', username);
        formdata.append('password', password);

        const response = await register(formdata);

        if (response.err) {
            ToastNotification('error', response.val.message);
            return;
        }

        if (response.ok) {
            // remove the old cookie
            removeCookie('auth');
            ToastNotification('success', 'you are now a proud member of the digital twin family');
            closeRegisterModal();
        }
        setToDefault();
    };

    const setToDefault = () => {
        setConfirmPassword('');
        setPassword('');
        setUsername('');
    };

    const handleCancelButtonClick = () => {
        // reseting all the states to default value
        setToDefault();
        closeRegisterModal();
    };

    return (
        <>
            <Modal show={isRegisterModalOpen} onClose={closeRegisterModal}>
                <form action={handleSubmit}>
                    <Modal.Header>Register</Modal.Header>
                    <Modal.Body>
                        <div>
                            <div className='mb-2 block'>
                                <Label htmlFor='username' value='Your username' />
                            </div>
                            <TextInput
                                id='name'
                                type='username'
                                icon={FaUser}
                                value={username}
                                placeholder='username'
                                required
                                onChange={e => setUsername(e.target.value)}
                                style={{ marginBottom: '4px' }}
                            />
                        </div>
                        <div>
                            <div className='mb-2 block'>
                                <Label htmlFor='password' value='Your password' />
                            </div>
                            <TextInput
                                id='password'
                                type='password'
                                icon={RiLockPasswordFill}
                                value={password}
                                placeholder='password'
                                required
                                onChange={e => setPassword(e.target.value)}
                                style={{ marginBottom: '4px' }}
                            />
                        </div>
                        <div>
                            <div className='mb-2 block'>
                                <Label htmlFor='password' value='Repeat password' />
                            </div>
                            <TextInput
                                id='confirm password'
                                type='password'
                                icon={RiLockPasswordFill}
                                value={confirmPassword}
                                placeholder='password'
                                required
                                onChange={e => {
                                    setConfirmPassword(e.target.value);
                                }}
                                style={{ marginBottom: '4px' }}
                            />
                        </div>
                    </Modal.Body>
                    <Modal.Footer>
                        <Button color='green' type='submit'>
                            Register
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

export default RegisterModal;
