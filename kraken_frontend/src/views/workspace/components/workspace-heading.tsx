import React, { useEffect } from "react";
import Select, { components } from "react-select";
import { Api, UUID } from "../../../api/api";
import { SimpleWorkspace } from "../../../api/generated";
import { clearSelectStyles } from "../../../components/select-menu";
import { ROUTES } from "../../../routes";
import "../../../styling/workspace-heading.css";
import CopyIcon from "../../../svg/copy";
import { copyToClipboard, handleApiError } from "../../../utils/helper";

/** React props for [`<WorkspaceHeading />`]{@link WorkspaceHeading} */
type WorkspaceHeadingProps = {
    /**
     * current workspace UUID
     */
    uuid: UUID;
    /**
     * current workspace name
     */
    name: string;
};

/**
 * Pane used as heading throughout the view for a specific workspace
 *
 * <Select /> to quickly change between workspaces
 */
export default function WorkspaceHeading(props: WorkspaceHeadingProps) {
    const [workspaces, setWorkspaces] = React.useState<Array<SimpleWorkspace>>([]);

    useEffect(() => {
        Api.workspaces.all().then(handleApiError(({ workspaces }) => setWorkspaces(workspaces)));
    }, []);

    return (
        <div className={"pane workspace-heading"}>
            <Select<{
                /**
                 * react select label to switch between workspaces
                 */
                label: string;
                /**
                 * react select value to switch between workspaces
                 */
                value: string;
            }>
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
                    workspaces
                        ?.filter((e) => {
                            return e.uuid !== props.uuid;
                        })
                        .map((w) => ({
                            label: w.name,
                            value: w.uuid,
                        })) ?? []
                }
                value={{
                    label: props.name,
                    value: props.uuid,
                }}
                isClearable={false}
                autoFocus={false}
                styles={clearSelectStyles()}
            />
            <div className={"workspace-heading-uuid"}>
                {props.uuid}
                <button
                    className={"icon-button"}
                    onClick={async () => {
                        await copyToClipboard(props.uuid);
                    }}
                >
                    <CopyIcon />
                </button>
            </div>
        </div>
    );
}
