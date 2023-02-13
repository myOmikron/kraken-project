import { login, test, registerKey, authenticate, logout } from "./auth";

export const Api = {
    auth: {
        login,
        logout,
        test,
        registerKey,
        authenticate,
    },
};
