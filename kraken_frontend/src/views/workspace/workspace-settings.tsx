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
import Select from "react-select";
import { ROUTES } from "../../routes";

type WorkspaceSettingsProps = {
    workspace: FullWorkspace;
};
type WorkspaceSettingsState = {
    workspaceName: string;
    workspaceDescription: string | null | undefined;
    invitePopup: boolean;
    deleteUserPopup: boolean;
    deleteWorkspacePopup: boolean;
    transferOwnershipPopup: boolean;
    memberName: string;
    type: null | SelectValue;
    memberList: Array<SelectValue>;
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
            workspaceDescription: this.props.workspace.description,
            invitePopup: false,
            deleteUserPopup: false,
            deleteWorkspacePopup: false,
            transferOwnershipPopup: false,
            memberName: "",
            type: null,
            memberList: [],
        };

        this.createMemberList().then();
    }

    async updateWorkspace() {
        (
            await Api.workspaces.update(this.props.workspace.uuid, {
                name: this.state.workspaceName,
                description: this.state.workspaceDescription,
            })
        ).match(
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

    async createMemberList() {
        (await Api.user.all()).match(
            (u) => {
                u.users.map((s) => {
                    let member = { label: s.displayName + " (" + s.username + ") ", value: s.uuid };
                    this.state.memberList.push(member);
                });
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
                                    <Tag name={"owner"} color={"primary"} />
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
                        this.setState({ invitePopup: false, type: null });
                    }}
                >
                    <form className="workspace-settings-popup pane">
                        <div className="workspace-setting-popup">
                            <h2 className="sub-heading"> Invite member</h2>
                            <Select
                                options={this.state.memberList}
                                onChange={(type) => {
                                    this.setState({ type });
                                }}
                                value={this.state.type}
                            />
                            <button
                                className="button"
                                onClick={() => {
                                    this.setState({ invitePopup: false, type: null });
                                }}
                            >
                                Invite
                                {/*TODO send out invitation*/}
                            </button>
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
                        this.setState({ transferOwnershipPopup: false, type: null });
                    }}
                >
                    <form className="workspace-settings-popup danger pane">
                        <div className="workspace-setting-popup">
                            <h2 className="sub-heading"> Transfer ownership</h2>
                            <Select
                                options={this.state.memberList}
                                onChange={(type) => {
                                    this.setState({ type });
                                }}
                                value={this.state.type}
                            />
                            <button
                                className="workspace-settings-red-button button"
                                onClick={() => {
                                    this.setState({ transferOwnershipPopup: false, type: null });
                                }}
                            >
                                Select
                            </button>
                        </div>
                        {/*TODO more ownership transfer stuff and another confirmation*/}
                        <div> Are you sure you want to transfer the ownership to {this.state.type?.label} ?</div>
                        <button className="workspace-settings-red-button button">Yes</button>
                        <button className="workspace-settings-red-button button">No</button>
                    </form>
                </Popup>
            </>
        );
    }
}
