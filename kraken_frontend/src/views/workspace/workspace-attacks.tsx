import React from "react";
import "../../styling/workspace-attacks.css";
import { FullWorkspace } from "../../api/generated";
import AttacksIcon from "../../svg/attacks";
import WorkspaceAttacksDehashed from "./attacks/workspace-attacks-dehashed";
import WorkspaceAttacksPortScanTcp from "./attacks/workspace-attacks-port-scan-tcp";
import WorkspaceAttacksCT from "./attacks/workspace-attacks-certificate-transparency";
import WorkspaceAttacksHostAlive from "./attacks/workspace-attacks-host-alive";

export type AttackType =
    | "bruteforce_subdomains"
    | "certificate_transparency"
    | "whois"
    | "host_alive"
    | "tcp_con"
    | "service_detection"
    | "dehashed";

export type AttackCategory = "domains" | "hosts" | "ports" | "services" | "other";

export type AttackInfo = {
    name: string;
    description: string;
};

type WorkspaceAttacksProps = {
    workspace: FullWorkspace;
};
type WorkspaceAttacksState = {
    selectedAttack: AttackType | null;
    hoverAttack: AttackType | null;
    activeAttackCategory: AttackCategory | null;
    attackInfo: null | AttackInfo;
};

const ATTACK_INFOS: { [key: string]: AttackInfo } = {
    bruteforce_subdomains: {
        name: "Bruteforce Subdomains",
        description: `Query a DNS server for all combinations of the given domain and the entries from the provided wordlist. 
        The entries of the wordlist will be prepended as subdomains.`,
    },
    certificate_transparency: {
        name: "Certificate Transparency",
        description: `Certificate transparency is a concept that was created to monitor the certificates that were signed by a CA.
        This attack will query the logs of a certificate transparency log collector to retrieve certificates with the given name in it.`,
    },
    whois: {
        name: "Whois",
        description: `Queries the RIPE database for the given IP address to find issued subnets for a company.`,
    },
    host_alive: {
        name: "Host alive",
        description: `Performs multiple scan techniques on an IP or a net to determine if a host is online.`,
    },
    tcp_con: {
        name: "TCP port scan",
        description: `Determine if a port is accepting TCP connections.`,
    },
    service_detection: {
        name: "Service Detection",
        description: `Try to determine which service is running on a specific port.`,
    },
    dehashed: {
        name: "Dehashed",
        description: `Dehashed provides an API to retrieve passwords (hashed and clear) and other information when querying a domain or an email.`,
    },
};

export default class WorkspaceAttacks extends React.Component<WorkspaceAttacksProps, WorkspaceAttacksState> {
    constructor(props: WorkspaceAttacksProps) {
        super(props);

        this.state = {
            selectedAttack: null,
            hoverAttack: null,
            activeAttackCategory: null,
            attackInfo: null,
        };
    }

    async startAttack() {}

    render() {
        return (
            <div className={"workspace-attacks-container"}>
                <div className={"pane workspace-attacks-info"}>
                    <h2 className={"sub-heading"}>Attack Info</h2>
                    {this.state.attackInfo !== null ? (
                        <>
                            <h3 className={"heading"}>{this.state.attackInfo.name}</h3>
                            <span className={""}>{this.state.attackInfo.description}</span>
                        </>
                    ) : (
                        <div className={"workspace-attacks-info-empty"}>
                            <span>- Hover over an attack to display information -</span>
                        </div>
                    )}
                </div>
                <div className={"pane workspace-attacks"}>
                    <AttacksIcon
                        onAttackHover={(x) => {
                            let info;
                            if (x !== null) {
                                info = ATTACK_INFOS[x];
                            } else {
                                if (this.state.selectedAttack !== null) {
                                    info = ATTACK_INFOS[this.state.selectedAttack];
                                } else {
                                    info = null;
                                }
                            }
                            this.setState({ hoverAttack: x, attackInfo: info });
                        }}
                        activeAttack={this.state.selectedAttack}
                        onAttackSelect={(x) => this.setState({ selectedAttack: x })}
                        activeAttackCategory={
                            this.state.hoverAttack === "bruteforce_subdomains" ||
                            this.state.hoverAttack === "certificate_transparency"
                                ? "domains"
                                : this.state.hoverAttack === "whois" || this.state.hoverAttack === "host_alive"
                                ? "hosts"
                                : this.state.hoverAttack === "tcp_con"
                                ? "ports"
                                : this.state.hoverAttack === "service_detection"
                                ? "services"
                                : this.state.hoverAttack === "dehashed"
                                ? "other"
                                : null
                        }
                    />
                </div>
                <div className={"pane workspace-attacks-details"}>
                    <h2 className={"sub-heading"}>Attack details</h2>
                    {this.state.selectedAttack === null ? (
                        <div className={"workspace-attacks-details-empty"}>
                            <span> - Click on an attack to start - </span>
                        </div>
                    ) : this.state.selectedAttack === "dehashed" ? (
                        <WorkspaceAttacksDehashed workspaceUuid={this.props.workspace.uuid} />
                    ) : this.state.selectedAttack === "tcp_con" ? (
                        <WorkspaceAttacksPortScanTcp workspaceUuid={this.props.workspace.uuid} />
                    ) : this.state.selectedAttack === "certificate_transparency" ? (
                        <WorkspaceAttacksCT workspaceUuid={this.props.workspace.uuid} />
                    ) : this.state.selectedAttack === "host_alive" ? (
                        <WorkspaceAttacksHostAlive workspaceUuid={this.props.workspace.uuid} />
                    ) : (
                        <span>Not implemented yet</span>
                    )}
                </div>
            </div>
        );
    }
}
