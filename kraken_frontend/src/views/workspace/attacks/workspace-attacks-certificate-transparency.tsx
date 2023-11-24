import React from "react";
import { Api, UUID } from "../../../api/api";
import "../../../styling/workspace-attacks-ct.css";
import StartAttack from "../components/start-attack";
import Input from "../../../components/input";
import Checkbox from "../../../components/checkbox";
import CollapseIcon from "../../../svg/collapse";
import ExpandIcon from "../../../svg/expand";
import { toast } from "react-toastify";
import { WORKSPACE_CONTEXT } from "../workspace";
import { PrefilledAttackParams } from "../workspace-attacks";
import { handleApiError } from "../../../utils/helper";

type WorkspaceAttacksCTProps = { prefilled: PrefilledAttackParams };
type WorkspaceAttacksCTState = {
    domain: string;
    includeExpired: boolean;
    retryInterval: number;
    maxRetries: number;

    showAdvanced: boolean;
};

export default class WorkspaceAttacksCT extends React.Component<WorkspaceAttacksCTProps, WorkspaceAttacksCTState> {
    static contextType = WORKSPACE_CONTEXT;
    declare context: React.ContextType<typeof WORKSPACE_CONTEXT>;

    constructor(props: WorkspaceAttacksCTProps) {
        super(props);

        this.state = {
            domain: this.props.prefilled.domain || "",
            includeExpired: false,
            maxRetries: 3,
            retryInterval: 500,

            showAdvanced: false,
        };
    }

    componentDidUpdate(prevProps: Readonly<WorkspaceAttacksCTProps>) {
        if (this.props.prefilled.domain !== undefined && this.props.prefilled.domain !== prevProps.prefilled.domain)
            this.setState({ domain: this.props.prefilled.domain });
    }

    startAttack() {
        Api.attacks
            .queryCertificateTransparency({
                includeExpired: this.state.includeExpired,
                workspaceUuid: this.context.workspace.uuid,
                target: this.state.domain,
                maxRetries: this.state.maxRetries,
                retryInterval: this.state.retryInterval,
            })
            .then(handleApiError((ok) => toast.success("Attack started")));
    }

    render() {
        return (
            <form
                className={"workspace-attacks-ct-container"}
                onSubmit={(event) => {
                    event.preventDefault();
                    this.startAttack();
                }}
            >
                <div className={"workspace-attacks-ct"}>
                    <label htmlFor={"domain"}>Domain</label>
                    <Input
                        id={"domain"}
                        required
                        value={this.state.domain}
                        onChange={(domain) => this.setState({ domain })}
                    />
                    <div className={"workspace-attacks-ct-include-expired"}>
                        <label htmlFor={"include-expired"}>Include expired certificates</label>
                        <Checkbox
                            id={"include-expired"}
                            value={this.state.includeExpired}
                            onChange={(v) => this.setState({ includeExpired: v })}
                        />
                    </div>
                    <span
                        className={"neon workspace-attacks-ct-advanced-button"}
                        onClick={() => {
                            this.setState({ showAdvanced: !this.state.showAdvanced });
                        }}
                    >
                        Advanced
                        {this.state.showAdvanced ? <CollapseIcon /> : <ExpandIcon />}
                    </span>
                    <div
                        className={
                            this.state.showAdvanced
                                ? "workspace-attacks-ct-advanced workspace-attacks-ct-advanced-open"
                                : "workspace-attacks-ct-advanced"
                        }
                    >
                        <label htmlFor={"retry-interval"}>Retry interval (in ms)</label>
                        <Input
                            id={"retry-interval"}
                            value={this.state.retryInterval.toString()}
                            onChange={(retryInterval) => {
                                const n = Number(retryInterval);
                                if (n === null || !Number.isSafeInteger(n) || n <= 0) {
                                    return;
                                }

                                this.setState({ retryInterval: n });
                            }}
                        />
                        <label htmlFor={"max-retries"}>Maximal retries</label>
                        <Input
                            id={"max-retries"}
                            value={this.state.maxRetries.toString()}
                            onChange={(maxRetries) => {
                                const n = Number(maxRetries);
                                if (n === null || !Number.isSafeInteger(n) || n < 0) {
                                    return;
                                }

                                this.setState({ maxRetries: n });
                            }}
                        />
                    </div>
                </div>
                <StartAttack />
            </form>
        );
    }
}
