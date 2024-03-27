import React from "react";
import { toast } from "react-toastify";
import { Api } from "../api/api";

import Input from "../components/input";
import "../styling/login.css";
import LoginLogoIcon from "../svg/login_logo";
import { handleApiError } from "../utils/helper";

type LoginProps = {
    onLogin(): void;
};
type LoginState = {
    username: string;
    password: string;
};

export default class Login extends React.Component<LoginProps, LoginState> {
    state: LoginState = {
        username: "",
        password: "",
    };

    performLogin() {
        Api.auth.login(this.state.username, this.state.password).then(
            handleApiError(() => {
                toast.success("Authenticated successfully");
                this.props.onLogin();
            }),
        );
    }

    render() {
        return (
            <>
                <div className="login-container">
                    <form
                        className="pane login"
                        method="post"
                        onSubmit={(e) => {
                            e.preventDefault();
                            this.performLogin();
                        }}
                    >
                        <LoginLogoIcon />
                        <h1 className="heading">Login</h1>
                        <Input
                            required
                            value={this.state.username}
                            onChange={(v: string) => {
                                this.setState({ username: v });
                            }}
                        />
                        <Input
                            required
                            type="password"
                            value={this.state.password}
                            onChange={(v: string) => {
                                this.setState({ password: v });
                            }}
                        />
                        <button className="button">Login</button>
                    </form>
                </div>
            </>
        );
    }
}
