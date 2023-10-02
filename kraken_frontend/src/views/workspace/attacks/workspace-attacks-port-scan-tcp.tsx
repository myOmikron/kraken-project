import React from "react";
import { UUID } from "../../../api/api";
import "../../../styling/workspace-attacks-pst.css";
import StartAttack from "../components/start-attack";
import Input from "../../../components/input";
import CollapseIcon from "../../../svg/collapse";
import ExpandIcon from "../../../svg/expand";

type WorkspaceAttacksPortScanTcpProps = {
    workspaceUuid: UUID;
};
type WorkspaceAttacksPortScanTcpState = {
    ipAddInput: string;
    showAdvanced: boolean;

    timeout: number;
    taskLimit: number;
    retries: number;
    interval: number;

    ips: Array<string>;
};

const v4 = "(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}";

const v6segment = "[a-fA-F\\d]{1,4}";

const v6 = `
(?:
(?:${v6segment}:){7}(?:${v6segment}|:)|                                    // 1:2:3:4:5:6:7::  1:2:3:4:5:6:7:8
(?:${v6segment}:){6}(?:${v4}|:${v6segment}|:)|                             // 1:2:3:4:5:6::    1:2:3:4:5:6::8   1:2:3:4:5:6::8  1:2:3:4:5:6::1.2.3.4
(?:${v6segment}:){5}(?::${v4}|(?::${v6segment}){1,2}|:)|                   // 1:2:3:4:5::      1:2:3:4:5::7:8   1:2:3:4:5::8    1:2:3:4:5::7:1.2.3.4
(?:${v6segment}:){4}(?:(?::${v6segment}){0,1}:${v4}|(?::${v6segment}){1,3}|:)| // 1:2:3:4::        1:2:3:4::6:7:8   1:2:3:4::8      1:2:3:4::6:7:1.2.3.4
(?:${v6segment}:){3}(?:(?::${v6segment}){0,2}:${v4}|(?::${v6segment}){1,4}|:)| // 1:2:3::          1:2:3::5:6:7:8   1:2:3::8        1:2:3::5:6:7:1.2.3.4
(?:${v6segment}:){2}(?:(?::${v6segment}){0,3}:${v4}|(?::${v6segment}){1,5}|:)| // 1:2::            1:2::4:5:6:7:8   1:2::8          1:2::4:5:6:7:1.2.3.4
(?:${v6segment}:){1}(?:(?::${v6segment}){0,4}:${v4}|(?::${v6segment}){1,6}|:)| // 1::              1::3:4:5:6:7:8   1::8            1::3:4:5:6:7:1.2.3.4
(?::(?:(?::${v6segment}){0,5}:${v4}|(?::${v6segment}){1,7}|:))             // ::2:3:4:5:6:7:8  ::2:3:4:5:6:7:8  ::8             ::1.2.3.4
)(?:%[0-9a-zA-Z]{1,})?                                             // %eth0            %1
`
    .replace(/\s*\/\/.*$/gm, "")
    .replace(/\n/g, "")
    .trim();

// Pre-compile only the exact regexes because adding a global flag make regexes stateful
const v46Exact = new RegExp(`(?:^${v4}$)|(?:^${v6}$)`);
const v4exact = new RegExp(`^${v4}$`);
const v6exact = new RegExp(`^${v6}$`);

export default class WorkspaceAttacksPortScanTcp extends React.Component<
    WorkspaceAttacksPortScanTcpProps,
    WorkspaceAttacksPortScanTcpState
> {
    constructor(props: WorkspaceAttacksPortScanTcpProps) {
        super(props);

        this.state = {
            ipAddInput: "",
            showAdvanced: false,
            interval: 100,
            retries: 6,
            taskLimit: 500,
            timeout: 1000,
            ips: [],
        };
    }

    addIp() {}

    render() {
        return (
            <div className={"workspace-attacks-pst-container"}>
                <div className={"workspace-attacks-pst"}>
                    <form
                        method={"post"}
                        onSubmit={(e) => {
                            e.preventDefault();
                            this.addIp();
                        }}
                    >
                        <Input
                            placeholder={"IP in CIDR notation"}
                            value={this.state.ipAddInput}
                            onChange={(ipAddInput) => this.setState({ ipAddInput })}
                        />
                    </form>
                    <div className={"workspace-attacks-pst-ips"}></div>
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
                        <Input
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
                        <Input
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
                        <Input
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
                        <Input
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
                <StartAttack active={true} onClick={() => {}} />
            </div>
        );
    }
}
