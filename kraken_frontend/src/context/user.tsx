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

    /** Reload the user's information */
    reset: () => void;
};

/** {@link React.Context} to access {@link FullUser user information} */
const USER_CONTEXT = React.createContext<UserContext>({
    user: {
        username: "",
        displayName: "",
        uuid: "",
        createdAt: new Date(0),
        permission: UserPermission.ReadOnly,
        lastLogin: null,
    },
    reset: () => {},
});
USER_CONTEXT.displayName = "UserContext";
export default USER_CONTEXT;

type UserProviderProps = {
    children?: React.ReactNode;
};
type UserProviderState = {
    user: FullUser | "unauthenticated" | "loading";
};

/**
 * Component for managing and providing the {@link UserContext}
 *
 * This is a **singleton** only use at most **one** instance in your application.
 */
export class UserProvider extends React.Component<UserProviderProps, UserProviderState> {
    state: UserProviderState = { user: "loading" };

    fetching: boolean = false;
    fetchUser = () => {
        // Guard against a lot of calls
        if (this.fetching) return;
        this.fetching = true;

        this.setState({ user: "loading" });

        Api.user.get().then((result) => {
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
                },
            );
            // Clear guard against a lot of calls
            this.fetching = false;
        });
    };

    componentDidMount() {
        this.fetchUser();

        // Register as global singleton
        // eslint-disable-next-line @typescript-eslint/no-this-alias
        if (USER_PROVIDER === null) USER_PROVIDER = this;
        else if (USER_PROVIDER === this) console.error("UserProvider did mount twice");
        else console.error("Two instances of UserProvider are used");

        // Report websocket state changes using toasts
        const errorToast = [
            "Connecting websocket...",
            {
                closeButton: false,
                closeOnClick: false,
                autoClose: false,
                isLoading: true,
            },
        ] as const;
        const successToast = ["Websocket has connected", { autoClose: 1000 }] as const;
        let runningToast: string | number | null = toast.warn(...errorToast);
        WS.addEventListener("state", (newState) => {
            switch (newState) {
                case "connected":
                    if (runningToast !== null) {
                        toast.dismiss(runningToast);
                        runningToast = null;
                    }
                    toast.success(...successToast);
                    break;
                default:
                    if (runningToast === null) runningToast = toast.error(...errorToast);
            }
        });
    }

    componentWillUnmount() {
        // Deregister as global singleton
        if (USER_PROVIDER === this) USER_PROVIDER = null;
        else if (USER_PROVIDER === null) console.error("UserProvider instance did unmount twice");
        else console.error("Two instances of UserProvider are used");
    }

    render() {
        switch (this.state.user) {
            case "loading":
                return <Loading />;
            case "unauthenticated":
                return (
                    <Login
                        onLogin={() => {
                            this.fetchUser();
                        }}
                    />
                );
            default:
                return (
                    <USER_CONTEXT.Provider
                        value={{
                            user: this.state.user,
                            reset: this.fetchUser,
                        }}
                    >
                        {this.props.children}
                    </USER_CONTEXT.Provider>
                );
        }
    }
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
