import React from "react";
import { GetUser } from "../api/generated/models";
import { Api } from "../api/api";
import Loading from "../components/loading";
import { StatusCode } from "../api/error";
import { toast } from "react-toastify";
import { ROUTES } from "../routes";

export type UserContext = {
    user: GetUser;
};
export const USER_CONTEXT = React.createContext<UserContext>({
    user: { username: "", displayName: "", uuid: "", createdAt: new Date(0), admin: false, lastLogin: null },
});
USER_CONTEXT.displayName = "UserContext";

type UserProviderProps = {
    children?: React.ReactNode;
};

/** Component for managing and providing the {@link UserContext} */
export function UserProvider(props: UserProviderProps) {
    const [user, setUser] = React.useState<GetUser | null>(null);
    React.useEffect(() => {
        Api.user.get().then((result) =>
            result.match(
                (user) => setUser(user),
                (error) => {
                    switch (error.status_code) {
                        case StatusCode.Unauthenticated:
                            ROUTES.LOGIN.visit({});
                            break;
                        default:
                            toast.error(error.message);
                            break;
                    }
                }
            )
        );
    }, []);

    if (user === null) return <Loading />;
    else return <USER_CONTEXT.Provider value={{ user }}>{props.children}</USER_CONTEXT.Provider>;
}
