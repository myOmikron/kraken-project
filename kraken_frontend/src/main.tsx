import React from "react";
import ReactDOM from "react-dom/client";

import { ToastContainer } from "react-toastify";
import "react-toastify/dist/ReactToastify.css";
import "./styling/toastify.css";
import "./styling/components.css";

import "./index.css";
import Background from "./views/background";
import { ROUTER } from "./routes";
import Menu from "./views/menu";
import { UserProvider } from "./context/user";

type RouterProps = {};
type RouterState = {
    path: Array<string>;
};

class Router extends React.Component<RouterProps, RouterState> {
    state: RouterState = {
        path: [],
    };

    componentDidMount() {
        // Update state to match url
        const setPath = () => {
            const rawPath = window.location.hash;

            // Ensure well-formed path i.e. always have a #/
            if (!rawPath.startsWith("#/")) {
                window.location.hash = "#/";

                // this method will be immediately triggered again
                return;
            }

            // Split everything after #/
            const path = rawPath.substring(2).split("/");

            // #/ should result in [] not [""]
            if (path.length === 1 && path[0] === "") {
                path.shift();
            }

            this.setState({ path });
        };

        setPath();
        window.addEventListener("hashchange", setPath);
    }

    render() {
        return (
            <UserProvider>
                <div className="content-container">
                    {ROUTER.matchAndRender(this.state.path) || <div>Unknown route</div>}
                </div>
                <Menu />
            </UserProvider>
        );
    }
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <>
        <Background />
        <Router />
        <ToastContainer
            autoClose={3500}
            theme="dark"
            toastClassName="toast-pane"
            progressClassName="toast-neon toast-progress"
        />
    </>
);
