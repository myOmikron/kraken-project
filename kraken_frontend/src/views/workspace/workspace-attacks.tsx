import React from "react";
import { Api, UUID } from "../../api/api";
import WorkspaceMenu from "./components/workspace-menu";
import "../../styling/workspace-attacks.css";
import WorkspaceHeading from "./components/workspace-heading";
import { FullWorkspace } from "../../api/generated";
import { toast } from "react-toastify";

type WorkspaceAttacksProps = {
    uuid: UUID;
};
type WorkspaceAttacksState = {
    workspace: FullWorkspace | null;
};

export default class WorkspaceAttacks extends React.Component<WorkspaceAttacksProps, WorkspaceAttacksState> {
    constructor(props: WorkspaceAttacksProps) {
        super(props);

        this.state = { workspace: null };
    }

    componentDidMount() {
        Api.workspaces.get(this.props.uuid).then((res) =>
            res.match(
                (workspace) => this.setState({ workspace }),
                (err) => toast.error(err.message)
            )
        );
    }

    render() {
        return <div className={"workspace-attacks-container"}></div>;
    }
}
