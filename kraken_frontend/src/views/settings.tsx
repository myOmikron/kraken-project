import React from "react";
import { toast } from "react-toastify";
import { Api } from "../api/api";
import { FullOauthClient, FullWordlist, SettingsFull } from "../api/generated";
import Input from "../components/input";
import Textarea from "../components/textarea";
import "../styling/settings.css";
import CloseIcon from "../svg/close";
import CopyIcon from "../svg/copy";
import { copyToClipboard, handleApiError } from "../utils/helper";

export default function Settings() {
    const [settings, setSettings] = React.useState<SettingsFull | null>(null);
    const [oauthApplications, setOauthApplications] = React.useState<Array<FullOauthClient>>([]);
    const [wordlists, setWordlists] = React.useState<Array<FullWordlist>>([]);
    const [newOAuthAppName, setNewOAuthAppName] = React.useState<string>("");
    const [newOAuthAppRedirectUrl, setNewOAuthAppRedirectUrl] = React.useState<string>("");
    const [wordlistName, setWordlistName] = React.useState<string>("");
    const [wordlistPath, setWordlistPath] = React.useState<string>("");
    const [wordlistDescription, setWordlistDescription] = React.useState<string>("");

    async function retrieveSettings() {
        await Api.admin.settings.get().then(handleApiError((settings) => setSettings(settings)));
    }

    async function getOAuthApps() {
        await Api.admin.oauthApplications.all().then(
            handleApiError((apps) => {
                setOauthApplications(apps.apps);
            }),
        );
    }

    async function updateWordlists() {
        await Api.admin.wordlists.all().then(
            handleApiError((wordlists) => {
                setWordlists(wordlists.wordlists);
            }),
        );
    }

    React.useEffect(() => {
        Promise.all([getOAuthApps(), retrieveSettings(), updateWordlists()]).then();
    }, []);

    async function createWordlist() {
        if (wordlistName === "") {
            toast.error("Name of the wordlist must not be empty");
            return;
        }

        if (wordlistPath === "") {
            toast.error("Path of the wordlist must not be empty");
        }

        await Api.admin.wordlists
            .create({
                name: wordlistName,
                path: wordlistPath,
                description: wordlistDescription,
            })
            .then(handleApiError((_) => toast.success("Created wordlist")));
    }

    async function saveSettings() {
        if (settings === null) {
            return;
        }

        await Api.admin.settings.update(settings).then(handleApiError((_) => toast.success("Settings updated")));
    }

    async function createOAuthApp() {
        if (newOAuthAppName === "" || newOAuthAppRedirectUrl === "") {
            toast.error("App name and redirect uri must not be empty");
            return;
        }

        await Api.admin.oauthApplications
            .create({ name: newOAuthAppName, redirectUri: newOAuthAppRedirectUrl })
            .then(handleApiError((_) => toast.success("OAuth application created")));
    }

    return (
        <>
            <div className={"settings-container"}>
                <div className={"settings-heading pane"}>
                    <h1 className={"heading"}>Kraken Settings</h1>
                </div>
                {settings !== null ? (
                    <form
                        className={"settings-dehashed pane"}
                        method={"post"}
                        onSubmit={async (x) => {
                            x.preventDefault();
                            await saveSettings();
                        }}
                    >
                        <h2 className={"heading"}>Dehashed API</h2>
                        <Input
                            value={
                                settings.dehashedEmail !== null && settings.dehashedEmail !== undefined
                                    ? settings.dehashedEmail
                                    : ""
                            }
                            onChange={(v) => {
                                if (settings !== null) {
                                    settings.dehashedEmail = v;
                                }
                                setSettings(settings);
                            }}
                            placeholder={"email"}
                        />
                        <Input
                            value={
                                settings.dehashedApiKey !== null && settings.dehashedApiKey !== undefined
                                    ? settings.dehashedApiKey
                                    : ""
                            }
                            onChange={(v) => {
                                if (settings !== null) {
                                    settings.dehashedApiKey = v;
                                }
                                setSettings(settings);
                            }}
                            placeholder={"api-key"}
                        />
                        <button className={"button"}>Save</button>
                    </form>
                ) : (
                    <div className={"pane"}>Loading ...</div>
                )}
                <div className={"pane settings-wordlists"}>
                    <h2 className={"heading"}>Wordlists</h2>
                    <form
                        className={"settings-wordlists-creation"}
                        method={"post"}
                        onSubmit={async (event) => {
                            event.preventDefault();
                            await createWordlist();
                            await updateWordlists();
                        }}
                    >
                        <h3 className={"sub-heading"}>Create wordlist</h3>
                        <label htmlFor={"wordlist-name"}>Name</label>
                        <Input
                            id={"wordlist-name"}
                            required={true}
                            value={wordlistName}
                            onChange={(v) => setWordlistName(v)}
                        />
                        <label htmlFor={"wordlist-path"}>Path</label>
                        <Input
                            id={"wordlist-path"}
                            required={true}
                            value={wordlistPath}
                            onChange={(v) => setWordlistPath(v)}
                        />
                        <label htmlFor={"wordlist-description"}>Description</label>
                        <Textarea
                            id={"wordlist-description"}
                            value={wordlistDescription}
                            onChange={(v) => setWordlistDescription(v)}
                        />
                        <button className={"button"}>Create</button>
                    </form>
                    <h3 className={"sub-heading"}>Existing wordlists</h3>
                    <div className={"settings-wordlists-list"}>
                        <span>Name</span>
                        <span>Path</span>
                        <span>Description</span>
                        <span>Delete</span>
                        {wordlists.map((x) => (
                            <>
                                <span>{x.name}</span>
                                <span>{x.path}</span>
                                <span>{x.description}</span>
                                <button
                                    className={"icon-button"}
                                    onClick={async () => {
                                        await Api.admin.wordlists.delete(x.uuid).then(
                                            handleApiError(async () => {
                                                toast.success("Wordlist deleted");
                                                await updateWordlists();
                                            }),
                                        );
                                    }}
                                >
                                    <CloseIcon />
                                </button>
                            </>
                        ))}
                    </div>
                </div>
                <div className={"pane settings-oauth"}>
                    <h2 className={"heading"}>OAuth applications</h2>
                    <form
                        className={"settings-oauth-app-creation"}
                        method={"post"}
                        onSubmit={async (event) => {
                            event.preventDefault();
                            await createOAuthApp();
                            await getOAuthApps();
                        }}
                    >
                        <h3 className={"sub-heading"}>Create app</h3>
                        <Input
                            placeholder={"app name"}
                            value={newOAuthAppName}
                            onChange={(v) => setNewOAuthAppName(v)}
                        />
                        <Input
                            placeholder={"redirect url"}
                            value={newOAuthAppRedirectUrl}
                            onChange={(v) => setNewOAuthAppRedirectUrl(v)}
                        />
                        <button className={"button"}>Create</button>
                    </form>

                    <h3 className={"sub-heading"}>Existing apps</h3>
                    <div className={"settings-oauth-applications"}>
                        <div className={"settings-oauth-applications-row"}>
                            <div>Name</div>
                            <div>Redirect URL</div>
                            <div>Client ID</div>
                            <div>Secret Key</div>
                            <div>Delete</div>
                        </div>

                        {oauthApplications.map((x) => (
                            <div key={x.uuid} className={"settings-oauth-applications-row"}>
                                <div>{x.name}</div>
                                <span>{x.redirectUri}</span>
                                <button
                                    className={"icon-button"}
                                    onClick={async () => {
                                        await copyToClipboard(x.uuid);
                                    }}
                                >
                                    <CopyIcon />
                                </button>
                                <button
                                    className={"icon-button"}
                                    onClick={async () => {
                                        await copyToClipboard(x.secret);
                                    }}
                                >
                                    <CopyIcon />
                                </button>
                                <button
                                    className={"icon-button"}
                                    onClick={async () => {
                                        await Api.admin.oauthApplications.delete(x.uuid).then(
                                            handleApiError(async (_) => {
                                                toast.success(`Deleted application ${x.name}`);
                                                await getOAuthApps();
                                            }),
                                        );
                                    }}
                                >
                                    <CloseIcon />
                                </button>
                            </div>
                        ))}
                    </div>
                </div>
            </div>
        </>
    );
}
