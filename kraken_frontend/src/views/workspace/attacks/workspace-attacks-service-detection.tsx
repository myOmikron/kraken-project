import React from "react";
import { Api, UUID } from "../../../api/api";
import StartAttack from "../components/start-attack";
import "../../../styling/workspace-attacks-svd.css";
import Input from "../../../components/input";
import CollapseIcon from "../../../svg/collapse";
import ExpandIcon from "../../../svg/expand";
import { toast } from "react-toastify";
import { WORKSPACE_CONTEXT } from "../workspace";
import { PrefilledAttackParams } from "../workspace-attacks";
import { handleApiError } from "../../../utils/helper";

type WorkspaceAttacksServiceDetectionProps = { prefilled: PrefilledAttackParams };
type WorkspaceAttacksServiceDetectionState = {
    address: string;
    port: string;
    timeout: number;

    showAdvanced: boolean;
};

export default class WorkspaceAttacksServiceDetection extends React.Component<
    WorkspaceAttacksServiceDetectionProps,
    WorkspaceAttacksServiceDetectionState
> {
    static contextType = WORKSPACE_CONTEXT;
    declare context: React.ContextType<typeof WORKSPACE_CONTEXT>;

    constructor(props: WorkspaceAttacksServiceDetectionProps) {
        super(props);

        this.state = {
            address: this.props.prefilled.ipAddr || "",
            port: (this.props.prefilled.port && String(this.props.prefilled.port)) || "",
            timeout: 500,
            showAdvanced: false,
        };
    }

    componentDidUpdate(prevProps: Readonly<WorkspaceAttacksServiceDetectionProps>) {
        if (this.props.prefilled.ipAddr !== undefined && this.props.prefilled.ipAddr !== prevProps.prefilled.ipAddr)
            this.setState({ address: this.props.prefilled.ipAddr });
        if (this.props.prefilled.port !== undefined && this.props.prefilled.port !== prevProps.prefilled.port)
            this.setState({ port: String(this.props.prefilled.port) });
    }

    startAttack() {
        const { address, port, timeout } = this.state;

        const p = Number(port);
        if (p === null || !Number.isSafeInteger(p) || (p <= 0 && p <= 65535)) {
            toast.error("Port is invalid");
            return;
        }

        Api.attacks
            .serviceDetection({
                workspaceUuid: this.context.workspace.uuid,
                address,
                port: p,
                timeout,
            })
            .then(handleApiError((_) => toast.success("Attack started")));
    }

    render() {
        return (
            <form
                className={"workspace-attacks-svd-container"}
                onSubmit={(event) => {
                    event.preventDefault();
                    this.startAttack();
                }}
            >
                <div className={"workspace-attacks-svd"}>
                    <label htmlFor={"ip"}>IP</label>
                    <Input
                        id={"ip"}
                        required
                        autoFocus
                        placeholder={"IP address"}
                        value={this.state.address}
                        onChange={(v) => this.setState({ address: v })}
                    />
                    <label htmlFor={"port"}>Port</label>
                    <Input
                        id={"port"}
                        required
                        placeholder={"Port"}
                        value={this.state.port}
                        onChange={(v) => this.setState({ port: v })}
                    />
                    <span
                        className={"neon workspace-attacks-svd-advanced-button"}
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
                                ? "workspace-attacks-svd-advanced workspace-attacks-svd-advanced-open"
                                : "workspace-attacks-svd-advanced"
                        }
                    >
                        <label htmlFor={"timeout"}>Timeout (in ms)</label>
                        <Input
                            id={"timeout"}
                            value={this.state.timeout.toString()}
                            placeholder={"timeout in ms"}
                            onChange={(timeout) => {
                                const n = Number(timeout);
                                if (n === null || !Number.isSafeInteger(n) || n <= 0) {
                                    return;
                                }

                                this.setState({ timeout: n });
                            }}
                        />
                    </div>
                </div>
                <StartAttack />
            </form>
        );
    }
}
