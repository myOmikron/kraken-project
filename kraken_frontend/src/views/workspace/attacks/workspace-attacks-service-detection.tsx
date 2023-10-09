import React from "react";
import { Api, UUID } from "../../../api/api";
import StartAttack from "../components/start-attack";
import "../../../styling/workspace-attacks-svd.css";
import Input from "../../../components/input";
import CollapseIcon from "../../../svg/collapse";
import ExpandIcon from "../../../svg/expand";
import { toast } from "react-toastify";

type WorkspaceAttacksServiceDetectionProps = {
    workspaceUuid: UUID;
};
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
    constructor(props: WorkspaceAttacksServiceDetectionProps) {
        super(props);

        this.state = {
            address: "",
            port: "",
            timeout: 500,
            showAdvanced: false,
        };
    }

    async startAttack() {
        const { address, port, timeout } = this.state;

        const p = Number(port);
        if (p === null || !Number.isSafeInteger(p) || (p <= 0 && p <= 65535)) {
            toast.error("Port is invalid");
            return;
        }

        (
            await Api.attacks.serviceDetection({ workspaceUuid: this.props.workspaceUuid, address, port: p, timeout })
        ).match(
            (_) => toast.success("Started service detection"),
            (err) => toast.error(err.message)
        );
    }

    render() {
        return (
            <div className={"workspace-attacks-svd-container"}>
                <div className={"workspace-attacks-svd"}>
                    <label htmlFor={"ip"}>IP</label>
                    <Input
                        id={"ip"}
                        placeholder={"IP address"}
                        value={this.state.address}
                        onChange={(v) => this.setState({ address: v })}
                    />
                    <label htmlFor={"port"}>Port</label>
                    <Input
                        id={"port"}
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
                <StartAttack
                    active={this.state.address !== "" && this.state.port !== ""}
                    onClick={async () => await this.startAttack()}
                />
            </div>
        );
    }
}
