import React from "react";
import { GetUser } from "../api/generated/models";
import { Api } from "../api/api";
import { toast } from "react-toastify";
import "../styling/me.css";
import Input from "../components/input";
import { ROUTES } from "../routes";
import { check } from "../utils/helper";
import Loading from "../components/loading";

type MeProps = {};
type MeState = {
    // queried data
    user?: GetUser;

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

    componentDidMount() {
        Api.user.get().then((result) =>
            result.match(
                (user) => this.setState({ user }),
                (err) => toast.error(err.message)
            )
        );
    }

    render() {
        const user = this.state.user;
        if (user === undefined) return <Loading />;

        return (
            <div className="pane me">
                <h1 className="heading neon">{user.displayName}</h1>
                <h2 className="heading neon">{user.username}</h2>
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
                    <label>New Password (repeated):</label>
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
                    ROUTES.LOGIN.visit({});
                },
                (err) => toast.error(err.message)
            )
        );
    }
}
