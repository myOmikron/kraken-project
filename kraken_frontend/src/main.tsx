import React from "react";
import ReactDOM from "react-dom/client";

import { ToastContainer } from "react-toastify";
import "react-toastify/dist/ReactToastify.css";
import "./toastify.css";

import "./index.css";
import Home from "./views/home";
import Background from "./views/background";
import Login from "./views/login";

type RouterProps = {};
type RouterState = {
    path: Array<String>;
};

class Router extends React.Component<RouterProps, RouterState> {
    constructor(props: RouterProps) {
        super(props);

        this.state = {
            path: [],
        };
    }

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
        const { path } = this.state;

        if (path[0] && path[0] === "login") {
            return <Login />;
        }

        let content = (function () {
            switch (path[0]) {
                // To "check" if the path doesn't continue and a certain depth use
                // ```
                // case undefined: // end of array
                // case "":        // trailing slash
                // ```
                case "":
                case undefined:
                    return <Home />;
                default:
                    break;
            }
        })();

        if (content === undefined) {
            content = <div>Unknown route</div>;
        }

        return (
            <>
                <div className="content-container">{content}</div>
            </>
        );
    }
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <>
        <Router />
        <ToastContainer
            autoClose={3500}
            theme="dark"
            toastClassName="toast-pane"
            progressClassName="toast-neon toast-progress"
        />
        <Background />
    </>
);
