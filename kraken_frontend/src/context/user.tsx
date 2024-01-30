import React from "react";
import { FullUser, UserPermission } from "../api/generated/models";
import { Api } from "../api/api";
import Loading from "../components/loading";
import { ApiError, StatusCode } from "../api/error";
import { toast } from "react-toastify";
import Login from "../views/login";
import WS from "../api/websocket";

/** The global {@link UserProvider} instance */
let USER_PROVIDER: UserProvider | null = null;

/** Data provided by the {@link USER_CONTEXT} */
export type UserContext = {
    user: FullUser;
};

/** {@link React.Context Context} to access {@link FullUser user information} */
const USER_CONTEXT = React.createContext<UserContext>({
    user: {
        username: "",
        displayName: "",
        uuid: "",
        createdAt: new Date(0),
        permission: UserPermission.ReadOnly,
        lastLogin: null,
    },
});
USER_CONTEXT.displayName = "UserContext";
export default USER_CONTEXT;

type UserProviderProps = {
    children?: React.ReactNode;
};
type UserProviderState = {
    user: FullUser | "unauthenticated" | null;
};

/**
 * Component for managing and providing the {@link UserContext}
 *
 * This is a **singleton** only use at most **one** instance in your application.
 */
export class UserProvider extends React.Component<UserProviderProps, UserProviderState> {
    state: UserProviderState = { user: null };

    fetchUser() {
        if (this.state.user == null)
            Api.user.get().then((result) =>
                result.match(
                    (user) => {
                        WS.connect(`${window.location.origin.replace("http", "ws")}/api/v1/ws`);
                        this.setState({ user });
                    },
                    (error) => {
                        switch (error.status_code) {
                            case StatusCode.Unauthenticated:
                                this.setState({ user: "unauthenticated" });
                                break;
                            default:
                                toast.error(error.message);
                                break;
                        }
                    }
                )
            );
    }

    componentDidMount() {
        WS.addEventListener("state.connected", () => toast.success("Websocket has connected", { autoClose: 1000 }));
        if (USER_PROVIDER === null) USER_PROVIDER = this;
        else if (USER_PROVIDER === this) console.error("UserProvider did mount twice");
        else console.error("Two instances of UserProvider are used");

        this.fetchUser();
    }

    componentDidUpdate(prevProps: Readonly<UserProviderProps>, prevState: Readonly<UserProviderState>) {
        this.fetchUser();
    }

    componentWillUnmount() {
        if (USER_PROVIDER === this) USER_PROVIDER = null;
        else if (USER_PROVIDER === null) console.error("UserProvider instance did unmount twice");
        else console.error("Two instances of UserProvider are used");
    }

    render() {
        switch (this.state.user) {
            case null:
                return <Loading />;
            case "unauthenticated":
                return <Login onLogin={() => this.setState({ user: null })} />;
            default:
                return (
                    <USER_CONTEXT.Provider
                        value={{
                            user: this.state.user,
                        }}
                    >
                        {this.props.children}
                    </USER_CONTEXT.Provider>
                );
        }
    }
}

/**
 * Reset the user information provided by {@link USER_CONTEXT}.
 *
 * This triggers an api call and might result in the user having to log in again.
 */
export function resetUser() {
    if (USER_PROVIDER !== null) USER_PROVIDER.setState({ user: null });
    else console.warn("resetUser has been called without a UserProvider");
}

/**
 * Inspect an error and handle the {@link StatusCode.Unauthenticated Unauthenticated} status code by requiring the user to log in again.
 *
 * @param error {@link ApiError} to inspect for {@link StatusCode.Unauthenticated Unauthenticated}
 */
export function inspectError(error: ApiError) {
    switch (error.status_code) {
        case StatusCode.Unauthenticated:
            if (USER_PROVIDER !== null) USER_PROVIDER.setState({ user: "unauthenticated" });
            else console.warn("inspectError has been called without a UserProvider");
            break;
        default:
            break;
    }
}
