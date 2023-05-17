import React from "react";
import { Api } from "../../api/api";
import { toast } from "react-toastify";
import { SimpleWorkspace } from "../../api/generated/models";
import Loading from "../../components/loading";
import { handleApiError } from "../../utils/helper";

type AdminWorkspacesProps = {};
type AdminWorkspacesState = {
    // queried data
    workspaces?: Array<SimpleWorkspace>;
};

/** View to expose the `/api/v1/admin/workspaces` endpoints */
export default class AdminWorkspaces extends React.Component<AdminWorkspacesProps, AdminWorkspacesState> {
    state: AdminWorkspacesState = {};

    componentDidMount() {
        this.fetchState();
    }

    fetchState() {
        Api.admin.workspaces.all().then(
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
                            <th>ID</th>
                            <th>Name</th>
                            <th>Description</th>
                            <th>Owner</th>
                        </tr>
                    </thead>
                    <tbody>
                        {workspaces.map((workspace) => (
                            <tr key={workspace.id}>
                                <td>{workspace.id}</td>
                                <td>{workspace.name}</td>
                                <td>{workspace.description || ""}</td>
                                <td>{workspace.owner.displayName}</td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>
        );
    }
}
