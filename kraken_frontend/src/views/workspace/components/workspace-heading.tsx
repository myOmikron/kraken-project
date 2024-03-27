import React from "react";
import Select, { components } from "react-select";
import { Api, UUID } from "../../../api/api";
import { SimpleWorkspace } from "../../../api/generated";
import { clearSelectStyles } from "../../../components/select-menu";
import { ROUTES } from "../../../routes";
import "../../../styling/workspace-heading.css";
import CopyIcon from "../../../svg/copy";
import { copyToClipboard, handleApiError } from "../../../utils/helper";

type WorkspaceHeadingProps = {
    uuid: UUID;
    name: string;
};
type WorkspaceHeadingState = {
    workspaces?: Array<SimpleWorkspace>;
};

export default class WorkspaceHeading extends React.Component<WorkspaceHeadingProps, WorkspaceHeadingState> {
    constructor(props: WorkspaceHeadingProps) {
        super(props);

        this.state = {};
    }

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

    render() {
        return (
            <div className={"pane workspace-heading"}>
                <Select<{ label: string; value: string }>
                    className={"dropdown"}
                    components={{
                        Option: (props) => (
                            <div
                                onMouseDown={(e) => {
                                    if ((e.ctrlKey && e.button == 0) || e.button == 1) {
                                        e.stopPropagation();
                                        ROUTES.WORKSPACE_HOSTS.open({ uuid: props.data.value });
                                    }
                                }}
                            >
                                <components.Option {...props} />
                            </div>
                        ),
                    }}
                    onChange={(t) => {
                        if (!t) return;
                        ROUTES.WORKSPACE_HOSTS.visit({ uuid: t.value });
                    }}
                    options={
                        this.state.workspaces
                            ?.filter((e) => {
                                return e.uuid !== this.props.uuid;
                            })
                            .map((w) => ({
                                label: w.name,
                                value: w.uuid,
                            })) ?? []
                    }
                    value={{
                        label: this.props.name,
                        value: this.props.uuid,
                    }}
                    isClearable={false}
                    autoFocus={false}
                    styles={clearSelectStyles()}
                />
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
