import React from "react";
import { copyToClipboard, handleApiError } from "../../../utils/helper";
import CopyIcon from "../../../svg/copy";
import { Api, UUID } from "../../../api/api";
import "../../../styling/workspace-heading.css";
import ArrowDownIcon from "../../../svg/arrow-down";
import ArrowUpIcon from "../../../svg/arrow-up";
import { SimpleWorkspace } from "../../../api/generated";
import { ROUTES } from "../../../routes";

type WorkspaceHeadingProps = {
    uuid: UUID;
    name: string;
};
type WorkspaceHeadingState = {
    dropdownOpen: boolean;
    workspaces?: Array<SimpleWorkspace>;
};

export default class WorkspaceHeading extends React.Component<WorkspaceHeadingProps, WorkspaceHeadingState> {
    constructor(props: WorkspaceHeadingProps) {
        super(props);

        this.state = { dropdownOpen: false };
    }

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
        console.log(this.props.name);
        return (
            <div className={"pane workspace-heading"}>
                {this.state.dropdownOpen ? (
                    <div className="workspace-heading-dropdown">
                        <div
                            className="workspace-heading-dropdown-open"
                            onClick={() => {
                                this.setState({ dropdownOpen: false });
                            }}
                        >
                            <h2 className={"sub-heading"}>{this.props.name}</h2>
                            <ArrowUpIcon />
                        </div>
                        <div className="workspace-heading-dropdown-content">
                            {this.state.workspaces
                                ?.filter((e) => {
                                    {
                                        /*TODO sort where you are owner/member*/
                                    }
                                    return e.name !== this.props.name;
                                })
                                .map((w) => {
                                    return (
                                        <div
                                            onClick={() => {
                                                ROUTES.WORKSPACE_HOSTS.visit({ uuid: w.uuid });
                                                this.setState({ dropdownOpen: false });
                                            }}
                                        >
                                            {w.name}
                                        </div>
                                    );
                                })}
                        </div>
                    </div>
                ) : (
                    <div className="workspace-heading-dropdown">
                        <div
                            className="workspace-heading-dropdown-heading"
                            onClick={() => {
                                this.setState({ dropdownOpen: true });
                            }}
                        >
                            <h2 className={"sub-heading"}>{this.props.name}</h2>
                            <ArrowDownIcon />
                        </div>
                    </div>
                )}
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
