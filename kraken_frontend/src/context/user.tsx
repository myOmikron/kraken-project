import React from "react";
import { GetUser } from "../api/generated/models";
import { Api } from "../api/api";
import Loading from "../components/loading";
import { StatusCode } from "../api/error";
import { toast } from "react-toastify";
import Login from "../views/login";

export type UserContext = {
    user: GetUser;
    resetUser(): void;
};
export const USER_CONTEXT = React.createContext<UserContext>({
    user: { username: "", displayName: "", uuid: "", createdAt: new Date(0), admin: false, lastLogin: null },
    resetUser() {},
});
USER_CONTEXT.displayName = "UserContext";

type UserProviderProps = {
    children?: React.ReactNode;
};

/** Component for managing and providing the {@link UserContext} */
export function UserProvider(props: UserProviderProps) {
    type UserState = GetUser | "unauthenticated" | null;
    const [user, setUser] = React.useState<UserState>(null);

    React.useEffect(() => {
        if (user == null)
            Api.user.get().then((result) =>
                result.match(
                    (user) => setUser(user),
                    (error) => {
                        switch (error.status_code) {
                            case StatusCode.Unauthenticated:
                                setUser("unauthenticated");
                                break;
                            default:
                                toast.error(error.message);
                                break;
                        }
                    }
                )
            );
    }, [user]);

    const resetUser = React.useCallback(
        function () {
            setUser(null);
        },
        [setUser]
    );

    switch (user) {
        case null:
            return <Loading />;
        case "unauthenticated":
            return <Login onLogin={resetUser} />;
        default:
            return (
                <USER_CONTEXT.Provider
                    value={{
                        user,
                        resetUser,
                    }}
                >
                    {props.children}
                </USER_CONTEXT.Provider>
            );
    }
}
