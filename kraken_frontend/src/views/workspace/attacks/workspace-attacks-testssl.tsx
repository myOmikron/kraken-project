import React from "react";
import { Api, UUID } from "../../../api/api";
import Input from "../../../components/input";
import StartAttack from "../components/start-attack";
import "../../../styling/workspace-attacks-host-alive.css";
import { toast } from "react-toastify";
import CollapseIcon from "../../../svg/collapse";
import ExpandIcon from "../../../svg/expand";
import { WORKSPACE_CONTEXT } from "../workspace";
import { PrefilledAttackParams, TargetType } from "../workspace-attacks";
import { handleApiError } from "../../../utils/helper";
import { StartTLSProtocol } from "../../../api/generated";
import Checkbox from "../../../components/checkbox";
import Select, { SingleValue } from "react-select";
import { selectStyles } from "../../../components/select-menu";

type WorkspaceAttacksTestsslProps = { prefilled: PrefilledAttackParams; targetType: TargetType | null };
type WorkspaceAttacksTestsslState = {
    domain: string;
    host: string;
    port: string;
    showAdvanced: boolean;
    basicAuth: boolean;
    basicAuthUser: string;
    basicAuthPass: string;
    startTls: StartTLSProtocol | null;
    connectTimeout: string;
    opensslTimeout: string;
};

export default class WorkspaceAttacksTestssl extends React.Component<
    WorkspaceAttacksTestsslProps,
    WorkspaceAttacksTestsslState
> {
    static contextType = WORKSPACE_CONTEXT;
    declare context: React.ContextType<typeof WORKSPACE_CONTEXT>;

    constructor(props: WorkspaceAttacksTestsslProps) {
        super(props);

        this.state = {
            domain: "",
            host: "",
            port: "443",
            showAdvanced: false,
            basicAuth: false,
            basicAuthUser: "",
            basicAuthPass: "",
            startTls: null,
            connectTimeout: "",
            opensslTimeout: "",
        };
    }

    componentDidUpdate(prevProps: Readonly<WorkspaceAttacksTestsslProps>) {}

    startAttack() {
        const {
            domain,
            host,
            port,
            basicAuth,
            basicAuthUser,
            basicAuthPass,
            startTls,
            connectTimeout,
            opensslTimeout,
        } = this.state;

        const p = Number(port);
        if (p === null || !Number.isSafeInteger(p) || (p <= 0 && p <= 65535)) {
            toast.error("Port is invalid");
            return;
        }

        const c = connectTimeout.length > 0 ? Number(connectTimeout) : null;
        if (c !== null && (!Number.isSafeInteger(c) || c < 0)) {
            toast.error("Connect timeout is invalid");
            return;
        }

        const o = opensslTimeout.length > 0 ? Number(opensslTimeout) : null;
        if (o !== null && (!Number.isSafeInteger(o) || o < 0)) {
            toast.error("Openssl timeout is invalid");
            return;
        }

        Api.attacks
            .testssl({
                workspaceUuid: this.context.workspace.uuid,
                uri: domain,
                host: host,
                port: p,
                basicAuth: basicAuth ? [basicAuthUser, basicAuthPass] : null,
                connectTimeout: c,
                opensslTimeout: o,

                starttls: startTls,
            })
            .then(handleApiError((_) => toast.success("Attack started")));
    }

    render() {
        return (
            <form
                className={"workspace-attacks-host-alive-container"}
                onSubmit={(event) => {
                    event.preventDefault();
                    this.startAttack();
                }}
            >
                <div className={"workspace-attacks-host-alive"}>
                    <label>Domain</label>
                    <Input
                        required
                        autoFocus
                        value={this.state.domain}
                        onChange={(domain) => this.setState({ domain })}
                    />
                    <label>IP</label>
                    <Input required value={this.state.host} onChange={(host) => this.setState({ host })} />
                    <label>Port</label>
                    <Input required value={this.state.port} onChange={(port) => this.setState({ port })} />
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
                        <label>HTTP Basic Auth</label>
                        <Checkbox value={this.state.basicAuth} onChange={(basicAuth) => this.setState({ basicAuth })} />
                        <label>Username</label>
                        <Input
                            placeholder={"Basic Auth Username"}
                            value={this.state.basicAuthUser}
                            onChange={(basicAuthUser) => this.setState({ basicAuthUser, basicAuth: true })}
                        />
                        <label>Password</label>
                        <Input
                            placeholder={"Basic Auth Password"}
                            value={this.state.basicAuthPass}
                            onChange={(basicAuthPass) => this.setState({ basicAuthPass, basicAuth: true })}
                        />
                        <label>StartTLS</label>
                        <Select
                            placeholder={"Protocol..."}
                            isClearable
                            styles={selectStyles("default")}
                            options={Object.values(StartTLSProtocol).map((s) => ({ value: s, label: s }))}
                            value={this.state.startTls && { value: this.state.startTls, label: this.state.startTls }}
                            // @ts-ignore
                            onChange={(value: SingleValue<{ key: StartTLSProtocol; value: StartTLSProtocol }>) => {
                                this.setState({ startTls: value && value.value });
                            }}
                        />
                        <label>Connect timeout</label>
                        <Input
                            value={this.state.connectTimeout}
                            onChange={(connectTimeout) => this.setState({ connectTimeout })}
                        />
                        <label>Openssl timeout</label>
                        <Input
                            value={this.state.opensslTimeout}
                            onChange={(opensslTimeout) => this.setState({ opensslTimeout })}
                        />
                    </div>
                </div>
                <StartAttack />
            </form>
        );
    }
}
