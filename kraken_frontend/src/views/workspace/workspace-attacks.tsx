import React from "react";
import { UUID } from "../../api/api";

type WorkspaceAttacksProps = {
    uuid: UUID;
};
type WorkspaceAttacksState = {};

export default class WorkspaceAttacks extends React.Component<WorkspaceAttacksProps, WorkspaceAttacksState> {
    constructor(props: WorkspaceAttacksProps) {
        super(props);

        this.state = {};
    }

    render() {
        return <div></div>;
    }
}
