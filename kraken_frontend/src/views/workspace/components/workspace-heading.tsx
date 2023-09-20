import React from "react";
import { copyToClipboard } from "../../../utils/helper";
import CopyIcon from "../../../svg/copy";
import { UUID } from "../../../api/api";
import "../../../styling/workspace-heading.css";

type WorkspaceHeadingProps = {
    uuid: UUID;
    name: string;
};
type WorkspaceHeadingState = {};

export default class WorkspaceHeading extends React.Component<WorkspaceHeadingProps, WorkspaceHeadingState> {
    static state: {};
    render() {
        return (
            <div className={"pane workspace-heading"}>
                <h2 className={"heading"}>{this.props.name}</h2>
                <div className={"workspace-heading-uuid"}>
                    {this.props.uuid}
                    <button
                        className={"icon-button"}
                        onClick={async () => {
                            await copyToClipboard(this.props.uuid);
                        }}
                    >
                        <CopyIcon />
                    </button>
                </div>
            </div>
        );
    }
}
