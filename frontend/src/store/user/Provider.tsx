import React, { createContext, useReducer } from "react";

// TODO: replace with auto-gen interface from backend protobuffers
interface User {
    username: string;
}

interface UserState {
    token?: string;
    user?: User;
};

interface LoginAction {
    type: 'login',
    token: string,
    user: User
};

interface LogoutAction {
    type: 'logout'
};

type UserAction = LoginAction | LogoutAction;

function reducer(state: UserState, action: UserAction): UserState {
    switch (action.type) {
        case 'login': {
            return {
                token: action.token,
                user: action.user
                // NOTE: a spread operator is often used here
            };
        }
        case 'logout': {
            return {};
        }
        default: {
            return { ...state };
        }
    }
}

const UserContext = createContext<[UserState, React.Dispatch<UserAction>]>([{}, () => { }]);

function UserProvider({ children }: { children: React.ReactNode }) {
    const [state, dispatch] = useReducer(reducer, {});

    return (
        <UserContext.Provider value={[state, dispatch]}>
            {children}
        </UserContext.Provider>
    );
}

export { type User, UserProvider, UserContext };
