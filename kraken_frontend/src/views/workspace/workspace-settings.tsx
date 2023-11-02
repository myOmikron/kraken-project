import React from "react";
import { FullWorkspace } from "../../api/generated";
import "../../styling/workspace-settings.css";
import Input from "../../components/input";
import { Api } from "../../api/api";
import Textarea from "../../components/textarea";
import { toast } from "react-toastify";
import Tag from "../../components/tag";
import CloseIcon from "../../svg/close";
import Popup from "reactjs-popup";
import { ROUTES } from "../../routes";
import SelectMenu from "../../components/select-menu";
import Bubble from "../../components/bubble";

type WorkspaceSettingsProps = {
    workspace: FullWorkspace;
};
type WorkspaceSettingsState = {
    workspaceName: string;
    workspaceDescription: string | null;
    invitePopup: boolean;
    deleteUserPopup: boolean;
    selected: boolean;
    deleteWorkspacePopup: boolean;
    transferOwnershipPopup: boolean;
    memberName: string;
    transferList: Array<SelectValue>;
    inviteList: Array<SelectValue>;
    selectedUser: null | SelectValue;
};

type SelectValue = {
    label: string;
    value: string;
};

export default class WorkspaceSettings extends React.Component<WorkspaceSettingsProps, WorkspaceSettingsState> {
    constructor(props: WorkspaceSettingsProps) {
        super(props);

        this.state = {
            workspaceName: this.props.workspace.name,
            workspaceDescription:
                this.props.workspace.description === undefined ? null : this.props.workspace.description,
            invitePopup: false,
            deleteUserPopup: false,
            deleteWorkspacePopup: false,
            transferOwnershipPopup: false,
            selected: false,
            memberName: "",
            transferList: [],
            inviteList: [],
            selectedUser: null,
        };
        this.createTransferList().then();
        this.createInviteList().then();
    }

    async updateWorkspace() {
        let update: { name: null | string; description: null | string } = { name: null, description: null };

        if (this.state.workspaceName !== this.props.workspace.name && this.state.workspaceName !== "") {
            update = { ...update, name: this.state.workspaceName };
        }

        if (this.state.workspaceDescription !== this.props.workspace.description) {
            update = { ...update, description: this.state.workspaceDescription };
        }

        (await Api.workspaces.update(this.props.workspace.uuid, update)).match(
            () => toast.success("Workspace updated"),
            (err) => toast.error(err.message)
        );
    }

    async deleteWorkspace() {
        (await Api.workspaces.delete(this.props.workspace.uuid)).match(
            () => toast.success("Deleted Workspace "),
            (err) => toast.error(err.message)
        );
    }

    async createTransferList() {
        (await Api.user.all()).match(
            (u) => {
                u.users
                    .filter((s) => {
                        return this.props.workspace.owner.uuid !== s.uuid;
                    })
                    .map((s) => {
                        let member = { label: s.displayName + " (" + s.username + ") ", value: s.uuid };
                        this.state.transferList.push(member);
                    });
            },
            (err) => toast.error(err.message)
        );
    }

    async createInviteList() {
        (await Api.user.all()).match(
            (u) => {
                u.users
                    .filter((s) => {
                        let users = [
                            ...this.props.workspace.members.map((x) => x.uuid),
                            this.props.workspace.owner.uuid,
                        ];
                        return !users.some((x) => x === s.uuid);
                    })
                    .map((s) => {
                        let member = { label: s.displayName + " (" + s.username + ") ", value: s.uuid };
                        this.state.inviteList.push(member);
                    });
            },
            (err) => toast.error(err.message)
        );
    }

    async inviteUser() {
        if (this.state.selectedUser === null) {
            toast.error("No user selected");
            return;
        }

        (await Api.workspaces.invitations.create(this.props.workspace.uuid, this.state.selectedUser.value)).match(
            (_) => {
                toast.success("Invitation was sent");
                this.setState({ selectedUser: null, invitePopup: false });
            },
            (err) => toast.error(err.message)
        );
    }

    render() {
        return (
            <>
                <div className={"workspace-settings-layout"}>
                    <div className="workspace-settings-row">
                        <form
                            className="pane workspace-settings-container"
                            method={"post"}
                            onSubmit={async (x) => {
                                x.preventDefault();
                                await this.updateWorkspace();
                            }}
                        >
                            <h2 className={"sub-heading"}> Workspace Settings </h2>
                            <div className={"workspace-settings-table"}>
                                <span>Name</span>
                                <Input
                                    value={this.state.workspaceName}
                                    onChange={(v) => {
                                        this.setState({ workspaceName: v });
                                    }}
                                    placeholder={this.props.workspace.name}
                                />
                                <span>Description</span>
                                <Textarea
                                    value={
                                        this.state.workspaceDescription !== null &&
                                        this.state.workspaceDescription !== undefined
                                            ? this.state.workspaceDescription
                                            : ""
                                    }
                                    onChange={(v) => {
                                        this.setState({ workspaceDescription: v });
                                    }}
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
                                        this.setState({ transferOwnershipPopup: true });
                                    }}
                                >
                                    Transfer
                                </button>
                                <span>Delete this workspace</span>
                                <button
                                    className="workspace-settings-red-button button"
                                    onClick={() => {
                                        this.setState({ deleteWorkspacePopup: true });
                                    }}
                                >
                                    Delete
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
                                        this.setState({ invitePopup: true });
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
                                <span>{this.props.workspace.owner.displayName}</span>
                                <div className={"workspace-settings-tag-container"}>
                                    <Bubble name={"owner"} color={"primary"} />
                                </div>
                            </div>
                            {this.props.workspace.members.map((m) => (
                                <div key={m.uuid} className={"workspace-settings-user-table-entry neon"}>
                                    <span>{m.displayName}</span>
                                    <div className={"workspace-settings-tag-container"}>
                                        <Tag name={"member"} />
                                    </div>
                                    <span>
                                        <button
                                            className={"icon-button"}
                                            onClick={() => {
                                                this.setState({
                                                    deleteUserPopup: true,
                                                    memberName: m.displayName,
                                                });
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
                        <div className={"workspace-settings-oauth-table-entry neon"}>
                            {/*TODO show linked apps*/}
                            <span>19/01/2023</span>
                            <span>Karla</span>
                            <span>dino</span>
                        </div>
                    </div>
                </div>
                <Popup
                    modal={true}
                    nested={true}
                    open={this.state.invitePopup}
                    onClose={() => {
                        this.setState({ invitePopup: false, selectedUser: null });
                    }}
                >
                    <form
                        method={"post"}
                        className="workspace-settings-popup pane"
                        onSubmit={async (e) => {
                            e.preventDefault();
                            await this.inviteUser();
                        }}
                    >
                        <div className="workspace-setting-popup">
                            <h2 className="sub-heading"> Invite member</h2>
                            <SelectMenu
                                options={this.state.inviteList}
                                theme={"default"}
                                value={this.state.selectedUser}
                                onChange={(type) => {
                                    this.setState({ selectedUser: type });
                                }}
                            />
                            <button className="button">Invite</button>
                        </div>
                    </form>
                </Popup>
                <Popup
                    modal={true}
                    nested={true}
                    open={this.state.deleteUserPopup}
                    onClose={() => {
                        this.setState({ deleteUserPopup: false, memberName: "" });
                    }}
                >
                    <div className="popup-content pane">
                        <div className="workspace-setting-popup">
                            <h2 className="sub-heading">Delete {this.state.memberName} from this workspace?</h2>
                            <button
                                className="button"
                                onClick={() => {
                                    this.setState({ deleteUserPopup: false, memberName: "" });
                                }}
                            >
                                No
                            </button>
                            <button
                                className="button"
                                onClick={() => {
                                    {
                                        /*TODO delete member*/
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
                    open={this.state.deleteWorkspacePopup}
                    onClose={() => {
                        this.setState({ deleteWorkspacePopup: false });
                    }}
                >
                    <div className="popup-content pane danger">
                        <div className="workspace-setting-popup">
                            <h2 className="sub-heading"> Are you sure to delete this workspace?</h2>
                            <span>All data will be lost upon deletion!</span>
                            <button
                                className="workspace-settings-red-button button"
                                onClick={() => {
                                    this.setState({ deleteWorkspacePopup: false });
                                }}
                            >
                                No
                            </button>
                            <button
                                className="workspace-settings-red-button button"
                                onClick={async (x) => {
                                    {
                                        x.preventDefault();
                                        await this.deleteWorkspace();
                                        ROUTES.WORKSPACES.visit({});
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
                    open={this.state.transferOwnershipPopup}
                    onClose={() => {
                        this.setState({ transferOwnershipPopup: false, selectedUser: null, selected: false });
                    }}
                >
                    {this.state.selected && this.state.selectedUser !== null ? (
                        <form className="workspace-setting-popup danger pane">
                            <span>Are you sure you want to transfer the ownership to</span>
                            <span> {this.state.selectedUser?.label} ?</span>
                            <button className="workspace-settings-red-button button">
                                {/*TODO more ownership transfer stuff */}Yes
                            </button>
                            <button
                                className="workspace-settings-red-button button"
                                onClick={() => {
                                    this.setState({
                                        transferOwnershipPopup: false,
                                        selectedUser: null,
                                        selected: false,
                                    });
                                }}
                            >
                                No
                            </button>
                        </form>
                    ) : (
                        <form className="workspace-settings-popup danger pane">
                            <div className="workspace-setting-popup">
                                <h2 className="sub-heading"> Transfer ownership</h2>
                                <SelectMenu
                                    options={this.state.transferList}
                                    theme={"red"}
                                    value={this.state.selectedUser}
                                    onChange={(type) => {
                                        this.setState({ selectedUser: type });
                                    }}
                                />
                                <button
                                    className="workspace-settings-red-button button"
                                    onClick={() => {
                                        this.setState({ selected: true });
                                    }}
                                >
                                    Select
                                </button>
                            </div>
                        </form>
                    )}
                </Popup>
            </>
        );
    }
}
