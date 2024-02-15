'use client';

import { User, UserContext } from '@/store/user';
import {
    Button,
    Modal,
    Label,
    TextInput,
} from 'flowbite-react';
import { useContext, useState } from 'react';

interface LoginModalProps {
    isLoginModalOpen: boolean;
    closeLoginModal: () => void;
}

function LoginModal({ isLoginModalOpen, closeLoginModal }: LoginModalProps) {
    const [userState, dispatch] = useContext(UserContext);

    const [username, setUsername] = useState("");
    const [password, setPassword] = useState("");

    const handleLoginButtonClick = () => {
        // hardcoded for the time being, to be replaced with gRPC call
        const token = "SOME JWT";
        const user: User = {
            username: 'SOME USERNAME'
        }

        // context: application state is lost on refresh, we are required to store the user's authorization token somewhere

        // storing JWT as httponly cookie (set by server) vs localStorage
        // the main disadvantage of using localStorage is that malicious third party scripts can access the contents of localStorage (such as in XSS)
        // on the other hand: httponly cookies make it more difficult for non-browser clients to send requests to authenticated endpoints
        // lastly: browser clients cannot authenticate/authorize gRPC calls, because they cannot read httponly cookies. gRPC does not 'support cookies'

        // using localStorage for now

        localStorage.setItem("authToken", token);

        dispatch({ type: 'login', token, user })

        closeLoginModal();
    }

    const handleCancelButtonClick = () => {
        setUsername("");
        setPassword("");
        closeLoginModal();
    }

    return (
        <>
            <Modal show={isLoginModalOpen} onClose={closeLoginModal}>
                <Modal.Header>Login</Modal.Header>
                <Modal.Body>
                    <div>
                        <div className="mb-2 block">
                            <Label htmlFor="email" value="Your email" />
                        </div>
                        <TextInput id="email" type="email" value={username} placeholder="email" required onChange={(e) => setUsername(e.target.value)} />
                    </div>
                    <div>
                        <div className="mb-2 block">
                            <Label htmlFor="password" value="Your password" />
                        </div>
                        <TextInput id="password" type="password" value={password} required onChange={(e) => setPassword(e.target.value)} />
                    </div>
                </Modal.Body>
                <Modal.Footer>
                    <Button onClick={handleLoginButtonClick}>Login</Button>
                    <Button color="gray" onClick={handleCancelButtonClick}>Cancel</Button>
                </Modal.Footer>
            </Modal>
        </>
    );
}

export default LoginModal;

