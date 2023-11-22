import React from "react";
import { Api, UUID } from "../../../api/api";
import Input from "../../../components/input";
import StartAttack from "../components/start-attack";
import "../../../styling/workspace-attacks-host-alive.css";
import { toast } from "react-toastify";
import CollapseIcon from "../../../svg/collapse";
import ExpandIcon from "../../../svg/expand";
import { WORKSPACE_CONTEXT } from "../workspace";
import { PrefilledAttackParams } from "../workspace-attacks";

type WorkspaceAttacksHostAliveProps = { prefilled: PrefilledAttackParams };
type WorkspaceAttacksHostAliveState = {
    target: string;
    timeout: number;
    concurrentLimit: number;
    showAdvanced: boolean;
};

export default class WorkspaceAttacksHostAlive extends React.Component<
    WorkspaceAttacksHostAliveProps,
    WorkspaceAttacksHostAliveState
> {
    static contextType = WORKSPACE_CONTEXT;
    declare context: React.ContextType<typeof WORKSPACE_CONTEXT>;

    constructor(props: WorkspaceAttacksHostAliveProps) {
        super(props);

        this.state = {
            target: this.props.prefilled.ipAddr || "",
            timeout: 1000,
            concurrentLimit: 50,
            showAdvanced: false,
        };
    }

    componentDidUpdate(prevProps: Readonly<WorkspaceAttacksHostAliveProps>) {
        if (this.props.prefilled.ipAddr !== undefined && this.props.prefilled.ipAddr !== prevProps.prefilled.ipAddr)
            this.setState({ target: this.props.prefilled.ipAddr });
    }

    async startAttack() {
        (
            await Api.attacks.hostAlive({
                timeout: this.state.timeout,
                concurrentLimit: this.state.concurrentLimit,
                targets: [this.state.target],
                workspaceUuid: this.context.workspace.uuid,
            })
        ).match(
            (_) => toast.success("Attack started"),
            (err) => toast.error(err.message),
        );
    }

    render() {
        return (
            <div className={"workspace-attacks-host-alive-container"}>
                <div className={"workspace-attacks-host-alive"}>
                    <label htmlFor={"cidr"}>IP / net in cidr</label>
                    <Input
                        id={"cidr"}
                        value={this.state.target}
                        onChange={(target) => {
                            this.setState({ target });
                        }}
                    />
                    <span
                        className={"neon workspace-attacks-host-alive-advanced-button"}
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
                                ? "workspace-attacks-host-alive-advanced workspace-attacks-host-alive-advanced-open"
                                : "workspace-attacks-host-alive-advanced"
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
                        <label htmlFor={"task-limit"}>Task limit</label>
                        <Input
                            id={"task-limit"}
                            placeholder={"task limit"}
                            value={this.state.concurrentLimit.toString()}
                            onChange={(taskLimit) => {
                                const n = Number(taskLimit);
                                if (n === null || !Number.isSafeInteger(n) || n <= 0) {
                                    return;
                                }

                                this.setState({ concurrentLimit: n });
                            }}
                        />
                    </div>
                </div>
                <StartAttack
                    active={this.state.target !== ""}
                    onClick={() => {
                        this.startAttack().then();
                    }}
                />
            </div>
        );
    }
}
