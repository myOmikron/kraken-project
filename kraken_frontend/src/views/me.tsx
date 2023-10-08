import React from "react";

import { Api, UUID } from "../api/api";
import { toast } from "react-toastify";
import "../styling/me.css";
import Input from "../components/input";
import { check, handleApiError } from "../utils/helper";
import USER_CONTEXT, { resetUser } from "../context/user";
import UserSettingsIcon from "../svg/user_settings";
import { FullApiKey, GetUser } from "../api/generated";
import CopyIcon from "../svg/copy";
import CloseIcon from "../svg/close";
import { CrossIcon } from "react-select/dist/declarations/src/components/indicators";

type MeProps = {};
type MeState = {
    // controlled state
    /** Old password */
    oldPwd: string;
    /** New password */
    newPwd: string;
    /** Repeated new password */
    repPwd: string;
    /** The username */
    username: string;
    /** The display name */
    displayName: string;
    /** api key name */
    apiKeyName: string;

    apiKeys: Array<FullApiKey>;
    user: GetUser;
};

export default class Me extends React.Component<MeProps, MeState> {
    state: MeState = {
        oldPwd: "",
        newPwd: "",
        repPwd: "",
        username: "",
        displayName: "",
        apiKeyName: "",
        apiKeys: [],
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
        this.retrieveApiKeys().then();
        this.setState({ username: user.username, displayName: user.displayName, user });
    }

    async createApiKey() {
        if (this.state.apiKeyName === "") {
            toast.error("Name must not be empty");
        }

        (await Api.user.apiKeys.create(this.state.apiKeyName)).match(
            async (_) => {
                toast.success("Created api key");
                this.setState({ apiKeyName: "" });
                await this.retrieveApiKeys();
            },
            (err) => toast.error(err.message)
        );
    }

    async retrieveApiKeys() {
        (await Api.user.apiKeys.all()).match(
            (keys) => {
                this.setState({ apiKeys: keys.keys });
            },
            (err) => toast.error(err.message)
        );
    }

    async deleteApiKey(uuid: UUID) {
        (await Api.user.apiKeys.delete(uuid)).match(
            async (_) => {
                toast.success("Deleted api key");
                await this.retrieveApiKeys();
            },
            (err) => toast.error(err.message)
        );
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
            <div className="me-container">
                <div className={"me-heading pane"}>
                    <h2 className={"sub-heading"}>User Settings</h2>
                </div>
                <form
                    className={"pane me-settings"}
                    method="post"
                    onSubmit={async (e) => {
                        e.preventDefault();
                        await this.updateAccount();
                    }}
                >
                    <h2 className={"sub-heading"}>Profile settings</h2>
                    <label htmlFor={"username"}>Username</label>
                    <Input
                        id={"username"}
                        value={this.state.displayName}
                        onChange={(v) => {
                            this.setState({ displayName: v });
                        }}
                    />
                    <label htmlFor={"display-name"}>Displayname</label>
                    <Input
                        id={"display-name"}
                        value={this.state.username}
                        onChange={(v) => {
                            this.setState({ username: v });
                        }}
                    />
                    <button className={"button"}>Save</button>
                </form>
                <form
                    className="pane me-change-pwd"
                    onSubmit={(e) => {
                        e.preventDefault();
                        this.changePwd();
                    }}
                >
                    <h2 className="sub-heading">Change Password</h2>
                    <label htmlFor={"curr-pw"}>Current Password</label>
                    <Input
                        id={"curr-pw"}
                        type="password"
                        value={this.state.oldPwd}
                        onChange={(oldPwd) => this.setState({ oldPwd })}
                    />
                    <label htmlFor={"new-pw"}>New Password</label>
                    <Input
                        id={"new-pw"}
                        type="password"
                        value={this.state.newPwd}
                        onChange={(newPwd) => this.setState({ newPwd })}
                    />
                    <label htmlFor={"new-pw-2"}>Confirm Password</label>
                    <Input
                        id={"new-pw-2"}
                        type="password"
                        value={this.state.repPwd}
                        onChange={(repPwd) => this.setState({ repPwd })}
                    />
                    <button type="submit" className="button">
                        Change
                    </button>
                </form>
                <div className={"pane me-api-keys"}>
                    <h2 className={"sub-heading"}>API keys</h2>
                    <form
                        method={"post"}
                        className={"me-api-keys-create"}
                        onSubmit={async (e) => {
                            e.preventDefault();
                            await this.createApiKey();
                        }}
                    >
                        <label htmlFor={"api-key-name"}>Name</label>
                        <Input
                            id={"api-key-name"}
                            value={this.state.apiKeyName}
                            onChange={(v) => this.setState({ apiKeyName: v })}
                        />
                        <button className={"button"}>Create</button>
                    </form>
                    <div className={"me-api-keys-table neon"}>
                        <div className={"me-api-keys-row"}>
                            <span>Name</span>
                            <span>Key</span>
                            <span>Copy</span>
                            <span>Delete</span>
                        </div>
                        {this.state.apiKeys.map((x) => (
                            <div key={x.uuid} className={"me-api-keys-row"}>
                                <span>{x.name}</span>
                                <span>{x.key}</span>
                                <span>
                                    <button
                                        className={"icon-button"}
                                        onClick={async () => {
                                            await navigator.clipboard.writeText(x.key);
                                            toast.success("Copied to clipboard");
                                        }}
                                    >
                                        <CopyIcon />
                                    </button>
                                </span>
                                <span>
                                    <button
                                        className={"icon-button"}
                                        onClick={async () => await this.deleteApiKey(x.uuid)}
                                    >
                                        <CloseIcon />
                                    </button>
                                </span>
                            </div>
                        ))}
                    </div>
                </div>
            </div>
        );
    }
}
