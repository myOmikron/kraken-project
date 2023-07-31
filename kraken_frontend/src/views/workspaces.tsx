import React from "react";
import { Api } from "../api/api";
import { check, handleApiError } from "../utils/helper";
import { SimpleWorkspace } from "../api/generated";
import Loading from "../components/loading";
import Input from "../components/input";
import Textarea from "../components/textarea";
import { toast } from "react-toastify";
import "../styling/workspaces.css";
import WorkspaceIcon from "../svg/workspace";
import Checkbox from "../components/checkbox";

type Sorting = "none" | "name" | "createdAt" | "lastModified";

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

    sorting: Sorting;
};

/** View to expose the `/api/v1/workspaces` endpoints */
export default class Workspaces extends React.Component<WorkspacesProps, WorkspacesState> {
    state: WorkspacesState = {
        createNew: false,
        newDesc: "",
        newName: "",
        search: "",
        onlyOwner: false,
        onlyMember: false,
        sorting: "none",
    };

    componentDidMount() {
        this.fetchState();
    }

    fetchState() {
        Api.workspaces.all().then(
            handleApiError(({ workspaces }) =>
                this.setState({
                    workspaces,
                })
            )
        );
    }

    async createWorkspace() {
        const { newName, newDesc } = this.state;
        if (!check([[newName.length > 0, "Empty name"]])) return;

        (await Api.workspaces.create({ name: newName, description: newDesc.length > 0 ? newDesc : null })).match(
            (_) => {
                toast.success("Created new workspace");
                this.setState({ newName: "", newDesc: "", createNew: false });
                this.fetchState();
            },
            (err) => {
                toast.error(err.message);
            }
        );
    }

    render() {
        const { workspaces } = this.state;
        if (workspaces === undefined) return <Loading />;

        return (
            <>
                <div className={"workspace-outer-container"}>
                    <div className={"workspace-creation pane"}>
                        <WorkspaceIcon />
                        <form
                            className={"workspace-creation-form"}
                            method={"post"}
                            onSubmit={async (e) => {
                                e.preventDefault();
                                await this.createWorkspace();
                            }}
                        >
                            <h2 className={"heading"}>Create a new workspace</h2>
                            <div className={"workspace-creation-table"}>
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
                        <div className={"workspace-filter-ownership"}>
                            <h3 className={"heading"}>Filter</h3>
                            <div className={"workspace-filter-ownership-table"}>
                                <span>Owner</span>
                                <Checkbox
                                    value={this.state.onlyOwner}
                                    onChange={() => {
                                        this.setState({ onlyOwner: !this.state.onlyOwner, onlyMember: false });
                                    }}
                                />
                                <span>Member</span>
                                <Checkbox
                                    value={this.state.onlyMember}
                                    onChange={() => {
                                        this.setState({ onlyOwner: false, onlyMember: !this.state.onlyMember });
                                    }}
                                />
                            </div>
                        </div>
                        <div className={"workspace-sorting"}>
                            <h3 className={"heading"}>Sorting</h3>
                            <div className={"workspace-sorting-table"}>
                                <span>Name</span>
                                <Checkbox
                                    value={this.state.sorting === "name"}
                                    onChange={() => {
                                        this.setState({ sorting: this.state.sorting === "name" ? "none" : "name" });
                                    }}
                                />
                                <div></div>
                                <span>Created timestamp</span>
                                <Checkbox
                                    value={this.state.sorting === "createdAt"}
                                    onChange={() => {
                                        this.setState({
                                            sorting: this.state.sorting === "createdAt" ? "none" : "createdAt",
                                        });
                                    }}
                                />
                                <span>Last modified</span>
                                <Checkbox
                                    value={this.state.sorting === "lastModified"}
                                    onChange={() => {
                                        this.setState({
                                            sorting: this.state.sorting === "lastModified" ? "none" : "lastModified",
                                        });
                                    }}
                                />
                            </div>
                        </div>
                    </div>
                    <div className={"workspace-container"}>
                        {workspaces
                            .filter((e) => {
                                if (this.state.search === "") return true;

                                return e.name.includes(this.state.search);
                            })
                            .map((w) => {
                                return (
                                    <div className={"pane workspace"}>
                                        <h3 className={"heading"}>{w.name}</h3>
                                        <div className={"workspace-table"}>
                                            <span>Owner:</span>
                                            <span>{w.owner.displayName}</span>
                                            <span>Description:</span>
                                            <span>{w.description}</span>
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
