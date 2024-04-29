import React, { useEffect } from "react";
import Select from "react-select";
import { toast } from "react-toastify";
import Popup from "reactjs-popup";
import { Api } from "../../api/api";
import { FullWorkspaceInvitation } from "../../api/generated";
import Bubble from "../../components/bubble";
import Input from "../../components/input";
import { selectStyles } from "../../components/select-menu";
import Tag from "../../components/tag";
import Textarea from "../../components/textarea";
import USER_CONTEXT from "../../context/user";
import { ROUTES } from "../../routes";
import "../../styling/workspace-settings.css";
import CloseIcon from "../../svg/close";
import { handleApiError } from "../../utils/helper";
import { WORKSPACE_CONTEXT } from "./workspace";

/**
 * Type for React Select
 */
type SelectValue = {
    /** select label */
    label: string;
    /** select value */
    value: string;
};

/**
 * Page including all settings for current workspace
 *
 * @returns JSX.Element setting page
 */
export default function WorkspaceSettings() {
    const { workspace } = React.useContext(WORKSPACE_CONTEXT);

    const [workspaceName, setWorkspaceName] = React.useState("");
    const [workspaceDescription, setWorkspaceDescription] = React.useState<string | null>("");
    const [invitePopup, setInvitePopup] = React.useState(false);
    const [deleteUserPopup, setDeleteUserPopup] = React.useState(false);
    const [isArchived, setIsArchived] = React.useState(false);
    const [deleteWorkspacePopup, setDeleteWorkspacePopup] = React.useState(false);
    const [archiveWorkspacePopup, setArchiveWorkspacePopup] = React.useState(false);
    const [unarchiveWorkspacePopup, setUnarchiveWorkspacePopup] = React.useState(false);
    const [transferOwnershipPopup, setTransferOwnershipPopup] = React.useState(false);
    const [selected, setSelected] = React.useState(false);
    const [memberName, setMemberName] = React.useState("");
    const [transferList, setTransferList] = React.useState<Array<SelectValue>>([]);
    const [inviteList, setInviteList] = React.useState<Array<SelectValue>>([]);
    const [selectedUser, setSelectedUser] = React.useState<null | SelectValue>(null);
    const [invitedUsers, setInvitedUsers] = React.useState<Array<FullWorkspaceInvitation>>([]);

    useEffect(() => {
        setWorkspaceName(workspace.name);
        setWorkspaceDescription(workspace.description || null);
    }, [workspace]);

    /**
     * Api call to get all invited users for current workspace
     *
     * @returns Promise<void>
     */
    function updateInvitedUsers() {
        return Api.workspaces.invitations
            .all(workspace.uuid)
            .then(handleApiError((x) => setInvitedUsers(x.invitations)));
    }

    /**
     * Api call if current workspace is archived
     *
     * @returns Promise<void>
     */
    function fetchWorkspace() {
        return Api.workspaces.get(workspace.uuid).then(handleApiError((x) => setIsArchived(x.archived)));
    }

    /**
     * Api call to update current workspace name and description
     *
     * @returns Promise<void>
     */
    function updateWorkspace() {
        let update: {
            /**
             * new workspace name
             */
            name: null | string;
            /**
             * new workspace description
             */
            description: null | string;
        } = { name: null, description: null };

        if (workspaceName !== workspace.name && workspaceName !== "") {
            update = { ...update, name: workspaceName };
        }

        if (workspaceDescription !== workspace.description) {
            update = { ...update, description: workspaceDescription };
        }

        return Api.workspaces
            .update(workspace.uuid, update)
            .then(handleApiError(() => toast.success("Workspace updated")));
    }

    /**
     * Api call to delete the current workspace and
     * redirect to workspace overview
     *
     * @returns Promise<void>
     */
    function deleteWorkspace() {
        toast.loading("Deleting workspace");
        return Api.workspaces.delete(workspace.uuid).then(
            handleApiError(() => {
                toast.success("Deleted Workspace");
                ROUTES.WORKSPACES.visit({});
            }),
        );
    }

    /**
     * Api call to archive the current workspace and
     * redirect to workspace overview
     */
    function archiveWorkspace() {
        toast.promise(
            Api.workspaces.archive(workspace.uuid).then(
                handleApiError(() => {
                    ROUTES.WORKSPACES.visit({});
                }),
            ),
            {
                pending: "Archiving workspace...",
                error: "Failed to archive workspace!",
                success: "Archived workspace",
            },
        );
    }

    /**
     * Api call to un-archive current workspace
     */
    function unarchiveWorkspace() {
        toast.promise(
            Api.workspaces.unarchive(workspace.uuid).then(
                handleApiError(() => {
                    setIsArchived(false);
                    setUnarchiveWorkspacePopup(false);
                }),
            ),
            {
                pending: "Unarchiving workspace...",
                error: "Failed to unarchive workspace!",
                success: "Unarchived workspace",
            },
        );
    }

    /**
     * Api call to get a list of all possible users
     * who the ownership of the current workspace can be transferred to
     *
     * @returns Promise<void>
     */
    function createTransferList() {
        return Api.user.all().then(
            handleApiError((u) => {
                u.users
                    .filter((s) => {
                        return workspace.owner.uuid !== s.uuid;
                    })
                    .map((s) => {
                        const member = { label: s.displayName + " (" + s.username + ") ", value: s.uuid };
                        setTransferList((l) => [...l, member]);
                    });
            }),
        );
    }

    /**
     * Api call to get a list of all possible users
     * who can be invited to current workspace
     *
     * @returns Promise<void>
     */
    function createInviteList() {
        return Api.user.all().then(
            handleApiError((u) => {
                u.users
                    .filter((s) => {
                        const users = [...workspace.members.map((x) => x.uuid), workspace.owner.uuid];
                        return !users.some((x) => x === s.uuid);
                    })
                    .map((s) => {
                        const member = { label: s.displayName + " (" + s.username + ") ", value: s.uuid };
                        setInviteList((l) => [...l, member]);
                    });
            }),
        );
    }

    /**
     * Api call to delete a user from current workspace
     */
    function deleteUser() {
        /*TODO delete member*/
    }

    /**
     * Api call to invite a user to the current workspace
     * calls to update list of all invited users
     *
     * @returns Promise<void>
     */
    function inviteUser() {
        if (selectedUser === null) {
            toast.error("No user selected");
            return;
        }

        return Api.workspaces.invitations.create(workspace.uuid, selectedUser.value).then(
            handleApiError(async () => {
                toast.success("Invitation was sent");
                setSelectedUser(null);
                setInvitePopup(false);
                return updateInvitedUsers();
            }),
        );
    }

    /**
     * Api call to transfer the ownership of the
     * current workspace to a different user
     *
     * @returns Promise<void>
     */
    function transferOwnership() {
        if (selectedUser === null) {
            toast.error("No user selected");
            return;
        }
        return Api.workspaces.transferOwnership(workspace.uuid, selectedUser.value).then(
            handleApiError(() => {
                toast.success("Transfer was successful");
                setSelectedUser(null);
                setTransferOwnershipPopup(false);
                setSelected(false);
            }),
        );
    }

    useEffect(() => {
        createTransferList();
        createInviteList();
        updateInvitedUsers();
        fetchWorkspace();
    }, []);

    return (
        <>
            <USER_CONTEXT.Consumer>
                {(ctx) => {
                    if (workspace.owner.uuid !== ctx.user.uuid) {
                        ROUTES.WORKSPACE_HOSTS.visit({ uuid: workspace.uuid });
                    }
                    return null;
                }}
            </USER_CONTEXT.Consumer>
            <div className={"workspace-settings-layout"}>
                <div className="workspace-settings-row">
                    <form
                        className="pane workspace-settings-container"
                        method={"post"}
                        onSubmit={async (x) => {
                            x.preventDefault();
                            await updateWorkspace();
                        }}
                    >
                        <h2 className={"sub-heading"}> Workspace Settings </h2>
                        <div className={"workspace-settings-table"}>
                            <span>Name</span>
                            <Input value={workspaceName} onChange={setWorkspaceName} placeholder={workspace.name} />
                            <span>Description</span>
                            <Textarea
                                value={
                                    workspaceDescription !== null && workspaceDescription !== undefined
                                        ? workspaceDescription
                                        : ""
                                }
                                onChange={setWorkspaceDescription}
                                placeholder={"Description"}
                            />
                        </div>
                        <button className={"button"}>Save</button>
                    </form>
                    <div className="workspace-settings-container danger pane">
                        <h2 className={"sub-heading"}>Danger Zone </h2>
                        <div className="workspace-settings-danger">
                            <span>Transfer ownership</span>
                            <button
                                className="workspace-settings-red-button button"
                                onClick={() => {
                                    setTransferOwnershipPopup(true);
                                }}
                            >
                                Transfer
                            </button>
                            <span>Delete this workspace</span>
                            <button
                                className="workspace-settings-red-button button"
                                onClick={() => {
                                    setDeleteWorkspacePopup(true);
                                }}
                            >
                                Delete
                            </button>
                            <span>{isArchived ? "Unarchive this workspace" : "Archive this workspace"}</span>
                            <button
                                className="workspace-settings-red-button button"
                                onClick={() => {
                                    if (isArchived) setUnarchiveWorkspacePopup(true);
                                    else setArchiveWorkspacePopup(true);
                                }}
                            >
                                {isArchived ? "Unarchive" : "Archive"}
                            </button>
                        </div>
                    </div>
                </div>
                <div className="pane workspace-settings-container">
                    <h2 className={"sub-heading"}>User control</h2>
                    <div className={"workspace-settings-container"}>
                        <div className={"workspace-settings-invite"}>
                            <button
                                className={"workspace-settings-button button"}
                                onClick={() => {
                                    setInvitePopup(true);
                                }}
                            >
                                Invite
                            </button>
                        </div>
                        <div className={"workspace-settings-user-table-heading neon"}>
                            <span>Username</span>
                            <span>Role</span>
                            <span>Delete</span>
                        </div>
                        <div className={"workspace-settings-user-table-entry neon"}>
                            <span>{workspace.owner.displayName}</span>
                            <div className={"workspace-settings-tag-container"}>
                                <Bubble name={"owner"} color={"primary"} />
                            </div>
                        </div>
                        {workspace.members.map((m) => (
                            <div key={m.uuid} className={"workspace-settings-user-table-entry neon"}>
                                <span>{m.displayName}</span>
                                <div className={"workspace-settings-tag-container"}>
                                    <Tag name={"member"} />
                                </div>
                                <span>
                                    <button
                                        className={"icon-button"}
                                        onClick={() => {
                                            setDeleteUserPopup(true);
                                            setMemberName(m.displayName);
                                        }}
                                    >
                                        <CloseIcon />
                                    </button>
                                </span>
                            </div>
                        ))}
                        {invitedUsers.map((i) => (
                            <div key={i.uuid} className={"workspace-settings-user-table-entry neon"}>
                                <span>{i.target.displayName}</span>
                                <div className={"workspace-settings-tag-container"}>
                                    <Tag name={"invited"} />
                                </div>
                                <span>
                                    <button
                                        className={"icon-button"}
                                        onClick={async () => {
                                            await Api.workspaces.invitations.retract(workspace.uuid, i.uuid).then(
                                                handleApiError(async () => {
                                                    toast.success("Invitation retracted");
                                                    await updateInvitedUsers();
                                                }),
                                            );
                                        }}
                                    >
                                        <CloseIcon />
                                    </button>
                                </span>
                            </div>
                        ))}
                    </div>
                </div>
                <div className="pane workspace-settings-container">
                    <h2 className={"sub-heading"}>Linked OAuth applications </h2>
                    <div className={"workspace-settings-oauth-table-heading neon"}>
                        <span>Date</span>
                        <span>Name</span>
                        <span>User</span>
                    </div>
                    {/*TODO show linked apps*/}
                </div>
            </div>
            <Popup
                modal={true}
                nested={true}
                open={invitePopup}
                onClose={() => {
                    setInvitePopup(false);
                    setSelectedUser(null);
                }}
            >
                <form
                    method={"post"}
                    className="workspace-settings-popup pane"
                    onSubmit={async (e) => {
                        e.preventDefault();
                        await inviteUser();
                    }}
                >
                    <div className="workspace-setting-popup">
                        <h2 className="sub-heading"> Invite member</h2>
                        <Select<SelectValue>
                            options={inviteList}
                            styles={selectStyles("default")}
                            value={selectedUser}
                            onChange={(type) => {
                                if (type) setSelectedUser(type);
                            }}
                        />
                        <button className="button">Invite</button>
                    </div>
                </form>
            </Popup>
            <Popup
                modal={true}
                nested={true}
                open={deleteUserPopup}
                onClose={() => {
                    setDeleteUserPopup(false);
                    setMemberName("");
                }}
            >
                <div className="popup-content pane">
                    <div className="workspace-setting-popup">
                        <h2 className="sub-heading">Delete {memberName} from this workspace?</h2>
                        <button
                            className="button"
                            onClick={() => {
                                setDeleteUserPopup(false);
                                setMemberName("");
                            }}
                        >
                            No
                        </button>
                        <button
                            className="button"
                            onClick={(x) => {
                                {
                                    x.preventDefault();
                                    deleteUser();
                                }
                            }}
                        >
                            Yes
                        </button>
                    </div>
                </div>
            </Popup>
            <Popup
                modal={true}
                nested={true}
                open={deleteWorkspacePopup}
                onClose={() => {
                    setDeleteWorkspacePopup(false);
                }}
            >
                <div className="popup-content pane danger">
                    <div className="workspace-setting-popup">
                        <h2 className="sub-heading"> Are you sure to delete this workspace?</h2>
                        <span>All data will be lost upon deletion!</span>
                        <button
                            className="workspace-settings-red-button button"
                            onClick={() => {
                                setDeleteWorkspacePopup(false);
                            }}
                        >
                            No
                        </button>
                        <button
                            className="workspace-settings-red-button button"
                            onClick={async (x) => {
                                {
                                    x.preventDefault();
                                    await deleteWorkspace();
                                }
                            }}
                        >
                            Yes
                        </button>
                    </div>
                </div>
            </Popup>
            <Popup
                modal={true}
                nested={true}
                open={archiveWorkspacePopup}
                onClose={() => {
                    setArchiveWorkspacePopup(false);
                }}
            >
                <div className="popup-content pane">
                    <div className="workspace-setting-popup">
                        <h2 className="sub-heading">Are you sure you want to archive this workspace?</h2>
                        <p>Once archived, you may unarchive this workspace from these settings later on.</p>
                        <p>
                            Some data in the workspace may be automatically deleted after a certain period of
                            inactivity.
                        </p>
                        <button
                            className="button"
                            onClick={() => {
                                setArchiveWorkspacePopup(false);
                            }}
                        >
                            No
                        </button>
                        <button
                            className="workspace-settings-red-button button"
                            onClick={(x) => {
                                x.preventDefault();
                                archiveWorkspace();
                            }}
                        >
                            Yes
                        </button>
                    </div>
                </div>
            </Popup>
            <Popup
                modal={true}
                nested={true}
                open={unarchiveWorkspacePopup}
                onClose={() => {
                    setUnarchiveWorkspacePopup(false);
                }}
            >
                <div className="popup-content pane">
                    <div className="workspace-setting-popup">
                        <h2 className="sub-heading">Are you sure you want to unarchive this workspace?</h2>
                        <p>The workspace will become visible by default to users again and can be edited again.</p>
                        <button
                            className="button"
                            onClick={() => {
                                setUnarchiveWorkspacePopup(false);
                            }}
                        >
                            No
                        </button>
                        <button
                            className="workspace-settings-red-button button"
                            onClick={(x) => {
                                x.preventDefault();
                                unarchiveWorkspace();
                            }}
                        >
                            Yes
                        </button>
                    </div>
                </div>
            </Popup>
            <Popup
                modal={true}
                nested={true}
                open={transferOwnershipPopup}
                onClose={() => {
                    setTransferOwnershipPopup(false);
                    setSelectedUser(null);
                    setSelected(false);
                }}
            >
                {selected && selectedUser !== null ? (
                    <form
                        className="workspace-setting-popup danger pane"
                        method={"post"}
                        onSubmit={async (e) => {
                            e.preventDefault();
                            await transferOwnership();
                            ROUTES.WORKSPACES.visit({});
                        }}
                    >
                        <h2 className="sub-heading">Transfer the ownership to {selectedUser?.label}?</h2>
                        <span> You will loose access to this workspace!</span>
                        <button className="workspace-settings-red-button button">Transfer</button>
                        <button
                            className="workspace-settings-red-button button"
                            onClick={() => {
                                setTransferOwnershipPopup(false);
                                setSelectedUser(null);
                                setSelected(false);
                            }}
                        >
                            Abort
                        </button>
                    </form>
                ) : (
                    <div className="workspace-settings-popup danger pane">
                        <div className="workspace-setting-popup">
                            <h2 className="sub-heading">Transfer ownership</h2>
                            <Select<SelectValue>
                                options={transferList}
                                styles={selectStyles("default")}
                                value={selectedUser}
                                onChange={setSelectedUser}
                            />
                            <button
                                className="workspace-settings-red-button button"
                                onClick={() => {
                                    if (selectedUser === null) {
                                        toast.error("No user selected");
                                        return;
                                    } else {
                                        setSelected(true);
                                    }
                                }}
                            >
                                Select
                            </button>
                        </div>
                    </div>
                )}
            </Popup>
        </>
    );
}
