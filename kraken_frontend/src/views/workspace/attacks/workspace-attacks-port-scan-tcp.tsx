import React from "react";
import { Api, UUID } from "../../../api/api";
import "../../../styling/workspace-attacks-pst.css";
import StartAttack from "../components/start-attack";
import Input from "../../../components/input";
import CollapseIcon from "../../../svg/collapse";
import ExpandIcon from "../../../svg/expand";
import Checkbox from "../../../components/checkbox";
import { toast } from "react-toastify";
import { WORKSPACE_CONTEXT } from "../workspace";
import { PrefilledAttackParams } from "../workspace-attacks";
import { handleApiError } from "../../../utils/helper";

type WorkspaceAttacksPortScanTcpProps = { prefilled: PrefilledAttackParams };
type WorkspaceAttacksPortScanTcpState = {
    ipAddInput: string;
    showAdvanced: boolean;

    timeout: number;
    taskLimit: number;
    retries: number;
    interval: number;
    skipIcmpCheck: boolean;

    ips: Array<string>;
};

export default class WorkspaceAttacksPortScanTcp extends React.Component<
    WorkspaceAttacksPortScanTcpProps,
    WorkspaceAttacksPortScanTcpState
> {
    static contextType = WORKSPACE_CONTEXT;
    declare context: React.ContextType<typeof WORKSPACE_CONTEXT>;

    constructor(props: WorkspaceAttacksPortScanTcpProps) {
        super(props);

        this.state = {
            ipAddInput: this.props.prefilled.ipAddr || "",
            showAdvanced: false,
            interval: 100,
            retries: 6,
            taskLimit: 500,
            timeout: 1000,
            skipIcmpCheck: false,
            ips: [],
        };
    }

    componentDidUpdate(prevProps: Readonly<WorkspaceAttacksPortScanTcpProps>) {
        if (this.props.prefilled.ipAddr !== undefined && this.props.prefilled.ipAddr !== prevProps.prefilled.ipAddr)
            this.setState({ ipAddInput: this.props.prefilled.ipAddr });
    }

    startAttack() {
        Api.attacks
            .scanTcpPorts({
                ports: ["1-65535"],
                timeout: this.state.timeout,
                concurrentLimit: this.state.taskLimit,
                maxRetries: this.state.retries,
                workspaceUuid: this.context.workspace.uuid,
                skipIcmpCheck: this.state.skipIcmpCheck,
                targets: [this.state.ipAddInput],
                retryInterval: this.state.interval,
            })
            .then(handleApiError((_) => toast.success("Attack started")));
    }

    render() {
        return (
            <form
                className={"workspace-attacks-pst-container"}
                onSubmit={(event) => {
                    event.preventDefault();
                    this.startAttack();
                }}
            >
                <div className={"workspace-attacks-pst"}>
                    <label htmlFor={"cidr"}>IP / net in cidr</label>
                    <Input
                        id={"cidr"}
                        value={this.state.ipAddInput}
                        onChange={(ipAddInput) => this.setState({ ipAddInput })}
                    />
                    <label htmlFor={"skip-icmp"}>Skip icmp check</label>
                    <Checkbox
                        id={"skip-icmp"}
                        value={this.state.skipIcmpCheck}
                        onChange={() => {
                            this.setState({ skipIcmpCheck: !this.state.skipIcmpCheck });
                        }}
                    />
                    <span
                        className={"neon workspace-attacks-pst-advanced-button"}
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
                                ? "workspace-attacks-pst-advanced workspace-attacks-pst-advanced-open"
                                : "workspace-attacks-pst-advanced"
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
                        <label htmlFor={"retries"}>Retries</label>
                        <Input
                            id={"retries"}
                            placeholder={"retries"}
                            value={this.state.retries.toString()}
                            onChange={(retries) => {
                                const n = Number(retries);
                                if (n === null || !Number.isSafeInteger(n) || n < 0) {
                                    return;
                                }

                                this.setState({ retries: n });
                            }}
                        />
                        <label htmlFor={"interval"}>Interval (in ms)</label>
                        <Input
                            id={"interval"}
                            placeholder={"interval in ms"}
                            value={this.state.interval.toString()}
                            onChange={(interval) => {
                                const n = Number(interval);
                                if (n === null || !Number.isSafeInteger(n) || n <= 0) {
                                    return;
                                }

                                this.setState({ interval: n });
                            }}
                        />
                        <label htmlFor={"task-limit"}>Task limit</label>
                        <Input
                            id={"task-limit"}
                            placeholder={"task limit"}
                            value={this.state.taskLimit.toString()}
                            onChange={(taskLimit) => {
                                const n = Number(taskLimit);
                                if (n === null || !Number.isSafeInteger(n) || n <= 0) {
                                    return;
                                }

                                this.setState({ taskLimit: n });
                            }}
                        />
                    </div>
                </div>
                <StartAttack active={this.state.ipAddInput !== ""} />
            </form>
        );
    }
}
