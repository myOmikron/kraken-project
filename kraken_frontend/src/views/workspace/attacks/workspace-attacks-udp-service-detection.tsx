import React from "react";
import { toast } from "react-toastify";
import { Api } from "../../../api/api";
import Input from "../../../components/input";
import "../../../styling/workspace-attacks-svd.css";
import CollapseIcon from "../../../svg/collapse";
import ExpandIcon from "../../../svg/expand";
import { handleApiError } from "../../../utils/helper";
import { parseUserPorts } from "../../../utils/ports";
import StartAttack from "../components/start-attack";
import { WORKSPACE_CONTEXT } from "../workspace";
import { PrefilledAttackParams } from "../workspace-attacks";

type WorkspaceAttacksUdpServiceDetectionProps = { prefilled: PrefilledAttackParams };
type WorkspaceAttacksUdpServiceDetectionState = {
    address: string;
    ports: string;
    timeout: number;
    maxRetries: number;
    retryInterval: number;
    concurrentLimit: number;

    showAdvanced: boolean;
};

export default class WorkspaceAttacksUdpServiceDetection extends React.Component<
    WorkspaceAttacksUdpServiceDetectionProps,
    WorkspaceAttacksUdpServiceDetectionState
> {
    static contextType = WORKSPACE_CONTEXT;
    declare context: React.ContextType<typeof WORKSPACE_CONTEXT>;

    constructor(props: WorkspaceAttacksUdpServiceDetectionProps) {
        super(props);

        this.state = {
            address: this.props.prefilled.ipAddr || "",
            ports: (this.props.prefilled.port && String(this.props.prefilled.port)) || "1-65535",
            timeout: 1000,
            maxRetries: 5,
            retryInterval: 200,
            concurrentLimit: 1024,
            showAdvanced: false,
        };
    }

    componentDidUpdate(prevProps: Readonly<WorkspaceAttacksUdpServiceDetectionProps>) {
        if (this.props.prefilled.ipAddr !== undefined && this.props.prefilled.ipAddr !== prevProps.prefilled.ipAddr)
            this.setState({ address: this.props.prefilled.ipAddr });
        if (this.props.prefilled.port !== undefined && this.props.prefilled.port !== prevProps.prefilled.port)
            this.setState({ ports: String(this.props.prefilled.port) });
    }

    startAttack() {
        const { address, ports, timeout, retryInterval, concurrentLimit, maxRetries } = this.state;

        parseUserPorts(ports).match((ports) => {
            Api.attacks
                .udpServiceDetection({
                    workspaceUuid: this.context.workspace.uuid,
                    address,
                    ports,
                    timeout,
                    retryInterval,
                    concurrentLimit,
                    maxRetries,
                })
                .then(handleApiError((_) => toast.success("Attack started")));
        }, (portError) => {
            toast.error("Port is invalid: " + portError);
        });
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
                    <label htmlFor={"ports"}>Ports</label>
                    <Input
                        id={"ports"}
                        required
                        placeholder={"Ports"}
                        value={this.state.ports}
                        onChange={(v) => this.setState({ ports: v })}
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
                        <label htmlFor={"maxRetries"}>Maximum Retries</label>
                        <Input
                            id={"maxRetries"}
                            placeholder={"maxRetries"}
                            value={this.state.maxRetries.toString()}
                            onChange={(maxRetries) => {
                                const n = Number(maxRetries);
                                if (n === null || !Number.isSafeInteger(n) || n < 0) {
                                    return;
                                }

                                this.setState({ maxRetries: n });
                            }}
                        />
                        <label htmlFor={"retryInterval"}>Retry interval (in ms)</label>
                        <Input
                            id={"retryInterval"}
                            placeholder={"Retry interval in ms"}
                            value={this.state.retryInterval.toString()}
                            onChange={(retryInterval) => {
                                const n = Number(retryInterval);
                                if (n === null || !Number.isSafeInteger(n) || n <= 0) {
                                    return;
                                }

                                this.setState({ retryInterval: n });
                            }}
                        />
                        <label htmlFor={"task-limit"}>Task limit</label>
                        <Input
                            id={"task-limit"}
                            placeholder={"task limit"}
                            value={this.state.concurrentLimit.toString()}
                            onChange={(concurrentLimit) => {
                                const n = Number(concurrentLimit);
                                if (n === null || !Number.isSafeInteger(n) || n <= 0) {
                                    return;
                                }

                                this.setState({ concurrentLimit: n });
                            }}
                        />
                    </div>
                </div>
                <StartAttack />
            </form>
        );
    }
}
