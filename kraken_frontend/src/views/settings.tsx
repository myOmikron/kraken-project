import React from "react";
import { toast } from "react-toastify";
import { Api } from "../api/api";
import {
    FullOauthClient,
    FullWordlist,
    SettingsFull,
    SimpleFindingDefinition,
    type FindingFactoryIdentifier,
} from "../api/generated";
import Input from "../components/input";
import Textarea from "../components/textarea";
import "../styling/settings.css";
import CloseIcon from "../svg/close";
import CopyIcon from "../svg/copy";
import { ObjectFns, copyToClipboard, handleApiError } from "../utils/helper";
import CollapsibleSection from "./workspace/components/collapsible-section";
import SelectFindingDefinition from "./workspace/components/select-finding-definition";

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
                <SettingsFindingFactory />
            </div>
        </>
    );
}

/** React props for [`<SettingsFindingFactory />`]{@link SettingsFindingFactory} */
type SettingsFindingFactoryProps = {};

/** An expandable panel in [`<Settings />`]{@link Settings} */
export function SettingsFindingFactory(props: SettingsFindingFactoryProps) {
    const [assignedEntries, setAssignedEntries] = React.useState<
        Partial<Record<FindingFactoryIdentifier, SimpleFindingDefinition>>
    >({});

    React.useEffect(() => {
        Api.admin.findingFactory.get().then(handleApiError(({ entries }) => setAssignedEntries(entries)));
    }, []);

    return (
        <div className={"pane settings-finding-factory"}>
            <h2 className={"heading"}>Finding Factory Entries</h2>
            {Object.entries(FINDING_FACTORY_SECTIONED_ENTRIES).map(([key, { heading, entries }]) => (
                <CollapsibleSection summary={heading} key={key}>
                    {ObjectFns.entries(entries).map(([identifier, { label }]) => (
                        <label key={identifier}>
                            <span>{label}</span>
                            <SelectFindingDefinition
                                selected={assignedEntries[identifier]?.uuid}
                                onSelect={(finding) => {
                                    Api.admin.findingFactory.set(identifier, finding?.uuid ?? null).then(
                                        handleApiError(() => {
                                            setAssignedEntries(({ [identifier]: _old, ...other }) =>
                                                finding ? { [identifier]: finding, ...other } : other,
                                            );
                                        }),
                                    );
                                }}
                                onHover={() => {}}
                                isClearable={true}
                            />
                        </label>
                    ))}
                </CollapsibleSection>
            ))}
        </div>
    );
}

/**
 * Helper type for [`FINDING_FACTORY_SECTIONED_ENTRIES`]{@link FINDING_FACTORY_SECTIONED_ENTRIES}.
 *
 * Please read `FINDING_FACTORY_SECTIONED_ENTRIES`'s docs an update `FindingFactorySection`'s docs,
 * if you reuse it outside `FINDING_FACTORY_SECTIONED_ENTRIES`.
 */
type FindingFactorySection = "TestSsl" | "ServiceDetection" | "KrakenAssi";
() => {
    // Checks whether there are any sections missing in `FindingFactorySection`.
    // It will fail to compile if there exists any `FindingFactoryIdentifier` which is not prefixed by any section.
    //
    // The error following error tells you to add `"NewSection"` to `FindingFactorySection`:
    // ```
    // Type 'FindingFactoryIdentifier' is not assignable to type '[...]'.
    // Type '"NewSectionNewIdentifier"' is not assignable to type '[...]'.
    // ```
    const _: `${FindingFactorySection}${string}` = undefined as unknown as FindingFactoryIdentifier;
};

/**
 * This object groups [`FindingFactoryIdentifier`]{@link FindingFactoryIdentifier} into sections
 * and associates them with meta information like their display labels.
 *
 * A "section" is simply a common prefix shared by several identifiers
 * which symbolizes some other connection between those identifiers.
 *
 * This object uses a few tricks to type check its completeness:
 * The list of all sections is stored in the helper type [`FindingFactorySection`]{@link FindingFactorySection}.
 * This list is type-checked to contain a prefix for every identifier.
 * This object requires a property for every section in this list which contains all identifier prefixed by it.
 * -> Using this setup, missing a new section / identifier will fail to type check
 */
const FINDING_FACTORY_SECTIONED_ENTRIES: {
    [Section in FindingFactorySection]: {
        /** The section's heading */
        heading: string;
        /** The identifiers contained in this section */
        entries: {
            [Identifier in Extract<FindingFactoryIdentifier, `${Section}${string}`>]: {
                /** The identifier's label */
                label: string;
            };
        };
    };
} = {
    TestSsl: {
        heading: "TestSsl",
        entries: {
            TestSslNullCiphers: { label: "Offers NULL Ciphers" },
            TestSslExportCiphers: { label: "Offers Export ciphers:" },
        },
    },
    ServiceDetection: {
        heading: "ServiceDetection",
        entries: {
            ServiceDetectionPostgres: { label: "Exposed postgres:" },
            ServiceDetectionMySql: { label: "Exposed MySQL:" },
        },
    },
    KrakenAssi: {
        heading: "KrakenAssi",
        entries: {
            KrakenAssiCertLeak: { label: "Cert Leak:" },
            KrakenAssiConnDowngrade: { label: "Connection Downgrade:" },
            KrakenAssiMissingHSTS: { label: "Missing HSTS:" },
        },
    },
};
