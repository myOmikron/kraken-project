import React from "react";
import { Api } from "../../api/api";
import { SimpleWorkspace } from "../../api/generated";
import Loading from "../../components/loading";
import { handleApiError } from "../../utils/helper";

/**
 *  View to expose the `/api/v1/admin/workspaces` endpoints
 *
 * @returns JSX.Element
 */
export default function AdminWorkspaces() {
    const [workspaces, setWorkspaces] = React.useState<Array<SimpleWorkspace> | undefined>(undefined);

    React.useEffect(() => {
        Api.admin.workspaces.all().then(
            handleApiError(({ workspaces }) => {
                setWorkspaces(workspaces);
            }),
        );
    });

    return (
        <>
            {workspaces === undefined ? (
                <Loading />
            ) : (
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
                                <tr key={workspace.uuid}>
                                    <td>{workspace.uuid}</td>
                                    <td>{workspace.name}</td>
                                    <td>{workspace.description || ""}</td>
                                    <td>{workspace.owner.displayName}</td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                </div>
            )}
        </>
    );
}
