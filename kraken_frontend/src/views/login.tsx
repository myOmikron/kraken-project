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

export default function Login(props: LoginProps) {
    const { onLogin } = props;
    const [username, setUsername] = React.useState<string>("");
    const [password, setPassword] = React.useState<string>("");

    function performLogin() {
        Api.auth.login(username, password).then(
            handleApiError(() => {
                toast.success("Authenticated successfully");
                onLogin();
            }),
        );
    }

    return (
        <>
            <div className="login-container">
                <form
                    className="pane login"
                    method="post"
                    onSubmit={(e) => {
                        e.preventDefault();
                        performLogin();
                    }}
                >
                    <LoginLogoIcon />
                    <h1 className="heading">Login</h1>
                    <Input
                        required
                        value={username}
                        onChange={(v: string) => {
                            setUsername(v);
                        }}
                    />
                    <Input
                        required
                        type="password"
                        value={password}
                        onChange={(v: string) => {
                            setPassword(v);
                        }}
                    />
                    <button className="button">Login</button>
                </form>
            </div>
        </>
    );
}
