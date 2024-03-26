import React from "react";
import { toast } from "react-toastify";
import { Api } from "../api/api";
import { SimpleWorkspace } from "../api/generated";
import Checkbox from "../components/checkbox";
import Input from "../components/input";
import Loading from "../components/loading";
import Textarea from "../components/textarea";
import USER_CONTEXT, { UserContext } from "../context/user";
import { ROUTES } from "../routes";
import "../styling/workspace-overview.css";
import WorkspaceIcon from "../svg/workspace";
import { check, handleApiError } from "../utils/helper";

type Sorting = "none" | "name" | "createdAt";

type WorkspacesProps = {};
type WorkspacesState = {
    /** Toggle modal to create a new workspace */
    createNew: boolean;

    // queried data
    workspaces?: Array<SimpleWorkspace>;

    // controlled state
    /** New workspace's name */
    newName: string;
    /** New workspace's description */
    newDesc: string;

    /** The search query */
    search: string;

    onlyOwner: boolean;
    onlyMember: boolean;

    onlyArchived: boolean;

    sorting: Sorting;
};

/** View to expose the `/api/v1/workspaces` endpoints */
export default class WorkspaceOverview extends React.Component<WorkspacesProps, WorkspacesState> {
    state: WorkspacesState = {
        createNew: false,
        newDesc: "",
        newName: "",
        search: "",
        onlyOwner: false,
        onlyMember: false,
        onlyArchived: false,
        sorting: "none",
    };

    static contextType = USER_CONTEXT;
    declare context: UserContext;

    componentDidMount() {
        this.fetchState();
    }

    fetchState() {
        Api.workspaces.all().then(
            handleApiError(({ workspaces }) =>
                this.setState({
                    workspaces,
                }),
            ),
        );
    }

    async createWorkspace() {
        const { newName, newDesc } = this.state;
        if (!check([[newName.length > 0, "Empty name"]])) return;

        await Api.workspaces.create({ name: newName, description: newDesc.length > 0 ? newDesc : null }).then(
            handleApiError((_) => {
                toast.success("Created new workspace");
                this.setState({ newName: "", newDesc: "", createNew: false });
                this.fetchState();
            }),
        );
    }

    render() {
        const { workspaces } = this.state;
        if (workspaces === undefined) return <Loading />;

        return (
            <>
                <div className={"workspace-list-outer-container"}>
                    <div className={"workspace-list-creation pane"}>
                        <WorkspaceIcon />
                        <form
                            className={"workspace-list-creation-form"}
                            method={"post"}
                            onSubmit={async (e) => {
                                e.preventDefault();
                                await this.createWorkspace();
                            }}
                        >
                            <h2 className={"heading"}>Create a new workspace</h2>
                            <div className={"workspace-list-creation-table"}>
                                <span>Name</span>
                                <Input
                                    value={this.state.newName}
                                    onChange={(v) => {
                                        this.setState({ newName: v });
                                    }}
                                />
                                <span>Description</span>
                                <Textarea
                                    value={this.state.newDesc}
                                    onChange={(v) => {
                                        this.setState({ newDesc: v });
                                    }}
                                />
                                <button className={"button"}>Create</button>
                            </div>
                        </form>
                    </div>
                    <div className={"workspace-list-filter pane"}>
                        <Input
                            placeholder={"Search"}
                            value={this.state.search}
                            onChange={(v) => {
                                this.setState({ search: v });
                            }}
                        />
                        <div className={"workspace-list-filter-ownership"}>
                            <h3 className={"heading"}>Filter</h3>
                            <div className={"workspace-list-checkbox-table"}>
                                <label>
                                    <Checkbox
                                        value={this.state.onlyOwner}
                                        onChange={(v) => {
                                            this.setState({ onlyOwner: v, onlyMember: false });
                                        }}
                                    />
                                    <span>Owner</span>
                                </label>
                                <label>
                                    <Checkbox
                                        value={this.state.onlyMember}
                                        onChange={(v) => {
                                            this.setState({ onlyOwner: false, onlyMember: v });
                                        }}
                                    />
                                    <span>Member</span>
                                </label>
                                <label>
                                    <Checkbox
                                        value={this.state.onlyArchived}
                                        onChange={(v) => {
                                            this.setState({ onlyArchived: v });
                                        }}
                                    />
                                    <span>Archived</span>
                                </label>
                            </div>
                        </div>
                        <div className={"workspace-list-sorting"}>
                            <h3 className={"heading"}>Sorting</h3>
                            <div className={"workspace-list-checkbox-table"}>
                                <label>
                                    <Checkbox
                                        value={this.state.sorting === "name"}
                                        onChange={() => {
                                            this.setState({ sorting: this.state.sorting === "name" ? "none" : "name" });
                                        }}
                                    />
                                    <span>Name</span>
                                </label>
                                <label>
                                    <Checkbox
                                        value={this.state.sorting === "createdAt"}
                                        onChange={() => {
                                            this.setState({
                                                sorting: this.state.sorting === "createdAt" ? "none" : "createdAt",
                                            });
                                        }}
                                    />
                                    <span>Created timestamp</span>
                                </label>
                            </div>
                        </div>
                    </div>
                    <div className={"workspace-list-container"}>
                        {workspaces
                            .filter((e) => {
                                if (
                                    this.state.search !== "" &&
                                    !e.name.toLowerCase().includes(this.state.search.toLowerCase())
                                )
                                    return false;

                                const isOwner = e.owner.uuid === this.context.user.uuid;

                                if (this.state.onlyOwner && !isOwner) return false;
                                if (this.state.onlyMember && isOwner) return false;

                                if (this.state.onlyArchived != (e.archived ?? false)) return false;

                                return true;
                            })
                            .sort((a, b) => {
                                switch (this.state.sorting) {
                                    case "createdAt":
                                        return a.createdAt.getTime() - b.createdAt.getTime();
                                    case "name":
                                        return a.name.localeCompare(b.name);
                                    case "none":
                                        return 0;
                                }
                            })
                            .map((w) => {
                                return (
                                    <div
                                        className={`pane workspace-list-item ${w.archived ? "archived" : ""}`}
                                        {...ROUTES.WORKSPACE_DATA.clickHandler({ uuid: w.uuid })}
                                    >
                                        <h3 className={"heading"}>{w.name}</h3>
                                        <div className={"workspace-list-table"}>
                                            <span>Owner:</span>
                                            <span>{w.owner.displayName}</span>
                                            <span>Description:</span>
                                            <span>{w.description}</span>
                                            <span>Created at:</span>
                                            <span>{w.createdAt.toLocaleString()}</span>
                                        </div>
                                    </div>
                                );
                            })}
                    </div>
                </div>
            </>
        );
    }
}
