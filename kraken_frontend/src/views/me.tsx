import React from "react";
import { toast } from "react-toastify";
import { Api, UUID } from "../api/api";
import { FullApiKey, FullUser } from "../api/generated";
import Input from "../components/input";
import USER_CONTEXT from "../context/user";
import "../styling/me.css";
import CloseIcon from "../svg/close";
import CopyIcon from "../svg/copy";
import { check, handleApiError } from "../utils/helper";

export default function Me() {
    const context = React.useContext(USER_CONTEXT);

    // controlled state
    /** Old password */
    const [oldPwd, setOldPwd] = React.useState<string>("");
    /** New password */
    const [newPwd, setNewPwd] = React.useState<string>("");
    /** Repeated new password */
    const [repPwd, setRepPwd] = React.useState<string>("");
    /** The username */
    const [username, setUsername] = React.useState<string>("");
    /** The display name */
    const [displayName, setDisplayName] = React.useState<string>("");
    /** api key name */
    const [apiKeyName, setApiKeyName] = React.useState<string>("");

    const [apiKeys, setApiKeys] = React.useState<Array<FullApiKey>>([]);
    const [user, setUser] = React.useState<FullUser>(context.user);

    async function retrieveApiKeys() {
        await Api.user.apiKeys.all().then(
            handleApiError((keys) => {
                setApiKeys(keys.keys);
            }),
        );
    }

    async function createApiKey() {
        if (apiKeyName === "") {
            toast.error("Name must not be empty");
        }

        await Api.user.apiKeys.create(apiKeyName).then(
            handleApiError(async (_) => {
                toast.success("Created api key");
                setApiKeyName("");
                await retrieveApiKeys();
            }),
        );
    }

    React.useEffect(() => {
        retrieveApiKeys().then();
        setUser(context.user);
        setUsername(context.user.username);
        setDisplayName(context.user.displayName);
    }, []);

    async function deleteApiKey(uuid: UUID) {
        Api.user.apiKeys.delete(uuid).then(
            handleApiError(async (_) => {
                toast.success("Deleted api key");
                await retrieveApiKeys();
            }),
        );
    }

    async function updateAccount() {
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

        const changes = {
            username: username !== user.username ? username : null,
            displayName: displayName !== user.displayName ? displayName : null,
        };

        await Api.user.update(changes).then(
            handleApiError((_) => {
                setUser({ ...user, displayName, username });
                toast.success("Account data updated");
            }),
        );
    }

    function changePwd() {
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
                context.reset();
            }),
        );
    }

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
                    await updateAccount();
                }}
            >
                <h2 className={"sub-heading"}>Profile settings</h2>
                <label htmlFor={"username"}>Username</label>
                <Input
                    id={"username"}
                    value={displayName}
                    onChange={(v) => {
                        setDisplayName(v);
                    }}
                />
                <label htmlFor={"display-name"}>Displayname</label>
                <Input
                    id={"display-name"}
                    value={username}
                    onChange={(v) => {
                        setUsername(v);
                    }}
                />
                <button className={"button"}>Save</button>
            </form>
            <form
                className="pane me-change-pwd"
                onSubmit={(e) => {
                    e.preventDefault();
                    changePwd();
                }}
            >
                <h2 className="sub-heading">Change Password</h2>
                <label htmlFor={"curr-pw"}>Current Password</label>
                <Input id={"curr-pw"} type="password" value={oldPwd} onChange={(oldPwd) => setOldPwd(oldPwd)} />
                <label htmlFor={"new-pw"}>New Password</label>
                <Input id={"new-pw"} type="password" value={newPwd} onChange={(newPwd) => setNewPwd(newPwd)} />
                <label htmlFor={"new-pw-2"}>Confirm Password</label>
                <Input id={"new-pw-2"} type="password" value={repPwd} onChange={(repPwd) => setRepPwd(repPwd)} />
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
                        await createApiKey();
                    }}
                >
                    <label htmlFor={"api-key-name"}>Name</label>
                    <Input id={"api-key-name"} value={apiKeyName} onChange={(v) => setApiKeyName(v)} />
                    <button className={"button"}>Create</button>
                </form>
                <div className={"me-api-keys-table neon"}>
                    <div className={"me-api-keys-row"}>
                        <span>Name</span>
                        <span>Key</span>
                        <span>Copy</span>
                        <span>Delete</span>
                    </div>
                    {apiKeys.map((x) => (
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
                                <button className={"icon-button"} onClick={async () => await deleteApiKey(x.uuid)}>
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
