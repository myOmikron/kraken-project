import React, { useEffect } from "react";
import { Api, UUID } from "../../api/api";
import { FullWorkspace } from "../../api/generated";
import "../../styling/workspace.css";
import { handleApiError } from "../../utils/helper";
import WorkspaceHeading from "./components/workspace-heading";
import WorkspaceMenu from "./components/workspace-menu";

export type WorkspaceContext = { workspace: FullWorkspace };
export const WORKSPACE_CONTEXT = React.createContext<WorkspaceContext>({
    workspace: {
        name: "",
        description: "",
        uuid: "",
        owner: { uuid: "", username: "", displayName: "" },
        attacks: [],
        members: [],
        notes: "",
        archived: false,
        createdAt: new Date(),
    },
});
WORKSPACE_CONTEXT.displayName = "WorkspaceContext";

type WorkspaceProps = {
    uuid: UUID;
    view: WorkspaceView;
    children: React.ReactNode;
};

export type WorkspaceView = "search" | "attacks" | "findings" | "data" | "hosts" | "settings" | "notes";

export default function Workspace(props: WorkspaceProps) {
    const [workspace, setWorkspace] = React.useState<FullWorkspace | null>(null);

    useEffect(() => {
        Api.workspaces.get(props.uuid).then(handleApiError((workspace) => setWorkspace(workspace)));
    }, [props.uuid]);

    return (
        <div className={"workspace-container"}>
            <WorkspaceHeading uuid={props.uuid} name={workspace !== null ? workspace.name : "Loading .."} />
            <WorkspaceMenu
                uuid={props.uuid}
                owner={workspace !== null ? workspace.owner.uuid : ""}
                active={props.view}
            />
            {workspace && (
                <WORKSPACE_CONTEXT.Provider value={{ workspace: workspace }}>
                    {props.children}
                </WORKSPACE_CONTEXT.Provider>
            )}
        </div>
    );
}
