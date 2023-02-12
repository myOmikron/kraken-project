import React from "react";
import ReactDOM from "react-dom/client";
import "./index.css";

import { toast, ToastContainer } from "react-toastify";
import "react-toastify/dist/ReactToastify.css";
import "./toastify.css";

type RouterProps = {};
type RouterState = {};

class Router extends React.Component<RouterProps, RouterState> {
    constructor(props: RouterProps) {
        super(props);

        this.state = {};
    }

    render() {
        return <div></div>;
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
    </>
);
