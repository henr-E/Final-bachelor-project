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

interface LoginModalProps {
    isLoginModalOpen: boolean;
    closeLoginModal: () => void;
}

function LoginModal({ isLoginModalOpen, closeLoginModal }: LoginModalProps) {
    const [userState, dispatch] = useContext(UserContext);

    const [username, setUsername] = useState("");
    const [password, setPassword] = useState("");

    const handleLoginButtonClick = async () => {
        // no need to validate here, add validation attributes to components (check the username and password fields)

        try {
            const resp = await fetch('/api/v1/session', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ username: username, password: password })
            });

            if (resp.status === 403) {
                // invalid username or password
                // show error message to the user
                return;
            }

            const data = await resp.json();

            // NOTE: a JWT is not 'encrypted', meaning that anyone can decode the JWT claims
            // do not put sensitive information in JWT claims (passwords)
            // even usernames should not be in claims
            const token = data.token;
            const user = jwtDecode<User>(token);

            // context: application state is lost on refresh, we are required to persist the user's authorization token somewhere
            // options include: localStorage, (regular) cookies, HttpOnly cookies...

            // storing JWT as httponly cookie (set by server) vs localStorage
            // the main disadvantage of using localStorage is that malicious third party scripts can access the contents of localStorage (such as in XSS)
            // on the other hand: httponly cookies make it more difficult for non-browser clients to send requests to authenticated endpoints
            // lastly: browser clients cannot authenticate/authorize gRPC calls, because they cannot read httponly cookies. gRPC does not 'support cookies'

            // using localStorage for now

            localStorage.setItem("authToken", token);

            // store token and user data in the application state
            dispatch({ type: 'login', token, user })
        } catch (e) {
            // TODO: show error message to user
        }

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

