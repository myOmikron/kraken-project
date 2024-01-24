import React from "react";
import { Api, UUID } from "../../../api/api";
import "../../../styling/workspace-attacks-dns-txt-scan.css";
import StartAttack from "../components/start-attack";
import Input from "../../../components/input";
import Checkbox from "../../../components/checkbox";
import CollapseIcon from "../../../svg/collapse";
import ExpandIcon from "../../../svg/expand";
import { toast } from "react-toastify";
import { WORKSPACE_CONTEXT } from "../workspace";
import { PrefilledAttackParams } from "../workspace-attacks";
import { handleApiError } from "../../../utils/helper";

type WorkspaceAttacksDnsTxtScanProps = { prefilled: PrefilledAttackParams };
type WorkspaceAttacksDnsTxtScanState = {
    domain: string;
};

export default class WorkspaceAttacksDnsTxtScan extends React.Component<
    WorkspaceAttacksDnsTxtScanProps,
    WorkspaceAttacksDnsTxtScanState
> {
    static contextType = WORKSPACE_CONTEXT;
    declare context: React.ContextType<typeof WORKSPACE_CONTEXT>;

    constructor(props: WorkspaceAttacksDnsTxtScanProps) {
        super(props);

        this.state = {
            domain: this.props.prefilled.domain || "",
        };
    }

    componentDidUpdate(prevProps: Readonly<WorkspaceAttacksDnsTxtScanProps>) {
        if (this.props.prefilled.domain !== undefined && this.props.prefilled.domain !== prevProps.prefilled.domain)
            this.setState({ domain: this.props.prefilled.domain });
    }

    startAttack() {
        Api.attacks
            .dnsTxtScan({
                targets: [this.state.domain],
                workspaceUuid: this.context.workspace.uuid,
            })
            .then(handleApiError((_) => toast.success("Attack started")));
    }

    render() {
        return (
            <form
                className={"workspace-attacks-dns-txt-scan-container"}
                onSubmit={(event) => {
                    event.preventDefault();
                    this.startAttack();
                }}
            >
                <div className={"workspace-attacks-dns-txt-scan"}>
                    <label htmlFor={"domain"}>Domain</label>
                    <Input
                        id={"domain"}
                        required
                        autoFocus
                        value={this.state.domain}
                        onChange={(domain) => this.setState({ domain })}
                    />
                </div>
                <StartAttack />
            </form>
        );
    }
}
