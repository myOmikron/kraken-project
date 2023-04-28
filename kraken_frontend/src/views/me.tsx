import React from "react";

import { Api } from "../api/api";
import { toast } from "react-toastify";
import "../styling/me.css";
import Input from "../components/input";
import { check } from "../utils/helper";
import { USER_CONTEXT } from "../context/user";

type MeProps = {};
type MeState = {
    // controlled state
    /** Old password */
    oldPwd: string;
    /** New password */
    newPwd: string;
    /** Repeated new password */
    repPwd: string;
};

export default class Me extends React.Component<MeProps, MeState> {
    state: MeState = {
        oldPwd: "",
        newPwd: "",
        repPwd: "",
    };

    static contextType = USER_CONTEXT;
    declare context: React.ContextType<typeof USER_CONTEXT>;

    render() {
        const { user } = this.context;

        return (
            <div className="pane me">
                <h1 className="heading neon">{user.displayName}</h1>
                <h2 className="heading neon">{user.username}</h2>
                {user.admin ? <h3 className="heading neon">{"<Admin>"}</h3> : null}
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
                    <Input type="password" value={this.state.oldPwd} onChange={(oldPwd) => this.setState({ oldPwd })} />
                    <label>New Password:</label>
                    <Input type="password" value={this.state.newPwd} onChange={(newPwd) => this.setState({ newPwd })} />
                    <label>Confirm Password:</label>
                    <Input type="password" value={this.state.repPwd} onChange={(repPwd) => this.setState({ repPwd })} />
                    <button type="submit" className="button">
                        Change
                    </button>
                </form>
            </div>
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
        Api.user.setPassword(oldPwd, newPwd).then((result) =>
            result.match(
                () => {
                    toast.success("Changed password successfully");
                    this.context.resetUser();
                },
                (err) => toast.error(err.message)
            )
        );
    }
}
