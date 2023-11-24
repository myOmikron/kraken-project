import React from "react";
import { Api, UUID } from "../../../api/api";
import "../../../styling/workspace-attacks-dns-resolution.css";
import StartAttack from "../components/start-attack";
import Input from "../../../components/input";
import Checkbox from "../../../components/checkbox";
import CollapseIcon from "../../../svg/collapse";
import ExpandIcon from "../../../svg/expand";
import { toast } from "react-toastify";
import { WORKSPACE_CONTEXT } from "../workspace";
import { PrefilledAttackParams } from "../workspace-attacks";
import { handleApiError } from "../../../utils/helper";

type WorkspaceAttacksDnsResolutionProps = { prefilled: PrefilledAttackParams };
type WorkspaceAttacksDnsResolutionState = {
    domain: string;
};

export default class WorkspaceAttacksDnsResolution extends React.Component<
    WorkspaceAttacksDnsResolutionProps,
    WorkspaceAttacksDnsResolutionState
> {
    static contextType = WORKSPACE_CONTEXT;
    declare context: React.ContextType<typeof WORKSPACE_CONTEXT>;

    constructor(props: WorkspaceAttacksDnsResolutionProps) {
        super(props);

        this.state = {
            domain: this.props.prefilled.domain || "",
        };
    }

    componentDidUpdate(prevProps: Readonly<WorkspaceAttacksDnsResolutionProps>) {
        if (this.props.prefilled.domain !== undefined && this.props.prefilled.domain !== prevProps.prefilled.domain)
            this.setState({ domain: this.props.prefilled.domain });
    }

    startAttack() {
        Api.attacks
            .dnsResolution({
                targets: [this.state.domain],
                concurrentLimit: 1,
                workspaceUuid: this.context.workspace.uuid,
            })
            .then(handleApiError((_) => toast.success("Attack started")));
    }

    render() {
        return (
            <form
                className={"workspace-attacks-dns-resolution-container"}
                onSubmit={(event) => {
                    event.preventDefault();
                    this.startAttack();
                }}
            >
                <div className={"workspace-attacks-dns-resolution"}>
                    <label htmlFor={"domain"}>Domain</label>
                    <Input
                        id={"domain"}
                        required
                        value={this.state.domain}
                        onChange={(domain) => this.setState({ domain })}
                    />
                </div>
                <StartAttack />
            </form>
        );
    }
}
