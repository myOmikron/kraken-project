import React from "react";
import { Api } from "../api/api";
import { check, handleApiError } from "../utils/helper";
import { GetWorkspace } from "../api/generated/models";
import Loading from "../components/loading";
import Popup from "reactjs-popup";
import Input from "../components/input";
import Textarea from "../components/textarea";
import { toast } from "react-toastify";

type WorkspacesProps = {};
type WorkspacesState = {
    /** Toggle modal to create a new workspace */
    createNew: boolean;

    /** Store a workspace to ask for confirmation before deleting it */
    confirmDelete: GetWorkspace | null;

    // queried data
    workspaces?: Array<GetWorkspace>;

    // controlled state
    /** New workspace's name */
    newName: string;
    /** New workspace's description */
    newDesc: string;
};

/** View to expose the `/api/v1/workspaces` endpoints */
export default class Workspaces extends React.Component<WorkspacesProps, WorkspacesState> {
    state: WorkspacesState = {
        confirmDelete: null,
        createNew: false,
        newDesc: "",
        newName: "",
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

    render() {
        const { workspaces } = this.state;
        if (workspaces === undefined) return <Loading />;

        return (
            <div className="pane">
                <table>
                    <thead>
                        <tr>
                            <th>Name</th>
                            <th>Description</th>
                            <th>Action</th>
                        </tr>
                    </thead>
                    <tbody>
                        {workspaces.map((workspace) => (
                            <tr key={workspace.id}>
                                <td>{workspace.name}</td>
                                <td>{workspace.description || ""}</td>
                                <td>
                                    <button
                                        type="button"
                                        className="button"
                                        onClick={() => this.setState({ confirmDelete: workspace })}
                                    >
                                        Delete
                                    </button>
                                </td>
                            </tr>
                        ))}
                    </tbody>
                </table>
                <button className="button" type="button" onClick={() => this.setState({ createNew: true })}>
                    Create new workspace
                </button>
                <Popup
                    modal
                    nested
                    open={this.state.confirmDelete !== null}
                    onClose={() => this.setState({ confirmDelete: null })}
                >
                    <div className="popup-content pane">
                        <h1 className="heading neon">
                            Are you sure you want to delete {this.state.confirmDelete?.name || ""}?
                        </h1>
                        <button className="button" type="button" onClick={() => this.deleteWorkspace()}>
                            Delete
                        </button>
                        <button className="button" type="button" onClick={() => this.setState({ confirmDelete: null })}>
                            Chancel
                        </button>
                    </div>
                </Popup>
                <Popup modal nested open={this.state.createNew} onClose={() => this.setState({ createNew: false })}>
                    <form
                        className="popup-content pane"
                        onSubmit={(e) => {
                            e.preventDefault();
                            this.createWorkspace();
                        }}
                    >
                        <h1 className="heading neon">New Workspace</h1>
                        <label>Name:</label>
                        <Input value={this.state.newName} onChange={(newName) => this.setState({ newName })} />
                        <label>Description:</label>
                        <Textarea value={this.state.newDesc} onChange={(newDesc) => this.setState({ newDesc })} />
                        <button type="submit" className="button">
                            Create workspace
                        </button>
                    </form>
                </Popup>
            </div>
        );
    }

    deleteWorkspace() {
        const { confirmDelete } = this.state;
        if (confirmDelete === null) return;

        Api.workspaces.delete(confirmDelete.id).then(
            handleApiError(() => {
                toast.success("Deleted workspace");
                this.setState({ confirmDelete: null });
                this.fetchState();
            })
        );
    }

    createWorkspace() {
        const { newName, newDesc } = this.state;
        if (!check([[newName.length > 0, "Empty name"]])) return;

        Api.workspaces.create({ name: newName, description: newDesc.length > 0 ? newDesc : null }).then(
            handleApiError(() => {
                toast.success("Created new workspace");
                this.setState({ newName: "", newDesc: "", createNew: false });
                this.fetchState();
            })
        );
    }
}
