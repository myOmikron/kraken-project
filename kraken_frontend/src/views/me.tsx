import React from "react";

import { Api } from "../api/api";
import { toast } from "react-toastify";
import "../styling/me.css";
import Input from "../components/input";
import { check, handleApiError } from "../utils/helper";
import USER_CONTEXT, { resetUser } from "../context/user";
import UserSettingsIcon from "../svg/user_settings";
import { GetUser } from "../api/generated";

type MeProps = {};
type MeState = {
    // controlled state
    /** Old password */
    oldPwd: string;
    /** New password */
    newPwd: string;
    /** Repeated new password */
    repPwd: string;
    username: string;
    displayName: string;
    user: GetUser;
};

export default class Me extends React.Component<MeProps, MeState> {
    state: MeState = {
        oldPwd: "",
        newPwd: "",
        repPwd: "",
        username: "",
        displayName: "",
        user: {
            displayName: "",
            username: "",
            admin: false,
            uuid: "",
            createdAt: new Date(),
            lastLogin: null,
        },
    };

    static contextType = USER_CONTEXT;
    declare context: React.ContextType<typeof USER_CONTEXT>;

    componentDidMount() {
        const { user } = this.context;
        this.setState({ username: user.username, displayName: user.displayName, user });
    }

    async updateAccount() {
        const { username, displayName, user } = this.state;
        if (username.length === 0) {
            toast.error("Username must not be empty");
            return;
        }

        if (displayName.length === 0) {
            toast.error("Display name must not be empty");
            return;
        }

        if (username === user.username && displayName === user.displayName) {
            toast.error("No changes detected");
            return;
        }

        let changes = {
            username: username !== user.username ? username : null,
            displayName: displayName !== user.displayName ? displayName : null,
        };

        (await Api.user.update(changes)).match(
            (_) => {
                user.displayName = displayName;
                user.username = username;
                this.setState({ user });
                toast.success("Account data updated");
            },
            (err) => {
                toast.error(err.message);
            }
        );
    }

    changePwd() {
        const { oldPwd, newPwd, repPwd } = this.state;
        if (
            !check([
                [newPwd.length > 0, "Please enter a new password"],
                [oldPwd.length > 0, "Please enter your old password"],
                [newPwd == repPwd, "The passwords don't match"],
            ])
        )
            return;
        Api.user.setPassword(oldPwd, newPwd).then(
            handleApiError(() => {
                toast.success("Changed password successfully");
                resetUser();
            })
        );
    }

    render() {
        return (
            <div className="pane me">
                <UserSettingsIcon />
                <form
                    method="post"
                    onSubmit={async (e) => {
                        e.preventDefault();
                        await this.updateAccount();
                    }}
                >
                    <h1 className={"heading"}>User settings</h1>
                    <div className={"me-settings"}>
                        <div>Username</div>
                        <Input
                            value={this.state.displayName}
                            onChange={(v) => {
                                this.setState({ displayName: v });
                            }}
                        />
                        <div>Displayname</div>
                        <Input
                            value={this.state.username}
                            onChange={(v) => {
                                this.setState({ username: v });
                            }}
                        />
                        <button className={"button"}>Save</button>
                    </div>
                    <hr />
                    <form
                        className="change-pwd"
                        onSubmit={(e) => {
                            e.preventDefault();
                            this.changePwd();
                        }}
                    >
                        <h2 className="heading neon">Change Password</h2>
                        <label>Current Password:</label>
                        <Input
                            type="password"
                            value={this.state.oldPwd}
                            onChange={(oldPwd) => this.setState({ oldPwd })}
                        />
                        <label>New Password:</label>
                        <Input
                            type="password"
                            value={this.state.newPwd}
                            onChange={(newPwd) => this.setState({ newPwd })}
                        />
                        <label>Confirm Password:</label>
                        <Input
                            type="password"
                            value={this.state.repPwd}
                            onChange={(repPwd) => this.setState({ repPwd })}
                        />
                        <button type="submit" className="button">
                            Change
                        </button>
                    </form>
                </form>
            </div>
        );
    }
}
