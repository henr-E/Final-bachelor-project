'use client';

import { User, UserContext } from '@/store/user';
import {
    Button,
    Modal,
    Label,
    TextInput,
} from 'flowbite-react';
import { useContext, useState } from 'react';
import { jwtDecode } from 'jwt-decode';

interface RegisterModalProps {
    isRegisterModalOpen: boolean;
    closeRegisterModal: () => void;
}

function RegisterModal({ isRegisterModalOpen, closeRegisterModal }: RegisterModalProps) {
    const [userState, dispatch] = useContext(UserContext);

    const [username, setUsername] = useState("");
    const [password, setPassword] = useState("");

    const handleRegisterButtonClick = async () => {
        // TODO: handle registering
        closeRegisterModal();
    }

    const handleCancelButtonClick = () => {
        closeRegisterModal();
    }

    return (
        <>
            <Modal show={isRegisterModalOpen} onClose={closeRegisterModal}>
                <Modal.Header>Register</Modal.Header>
                <Modal.Body>
                    <div>
                        <div className="mb-2 block" >
                            <Label htmlFor="email" value="Your email"/>
                        </div>
                        <TextInput id="email" type="email" value={username} placeholder="email" required
                                   onChange={(e) => setUsername(e.target.value)} style={{ marginBottom: '10px' }}/>
                    </div>
                    <div>
                        <div className="mb-2 block">
                            <Label htmlFor="password" value="Your password"/>
                        </div>
                        <TextInput id="password" type="password" value={password} placeholder="password" required
                                   onChange={(e) => setPassword(e.target.value)} style={{ marginBottom: '10px' }} />
                    </div>
                    <div>
                        <div className="mb-2 block">
                            <Label htmlFor="password" value="Repeat password"/>
                        </div>
                        <TextInput id="password" type="password" value={password} placeholder="password" required
                                   onChange={(e) => setPassword(e.target.value)}/>
                    </div>
                </Modal.Body>
                <Modal.Footer>
                    <Button onClick={handleRegisterButtonClick}>Register</Button>
                    <Button color="gray" onClick={handleCancelButtonClick}>Cancel</Button>
                </Modal.Footer>
            </Modal>
        </>
    );
}

export default RegisterModal;
