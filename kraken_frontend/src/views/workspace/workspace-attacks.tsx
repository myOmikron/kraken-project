import React from "react";
import "../../styling/workspace-attacks.css";
import { FullDomain, FullHost, FullPort, FullService, FullWorkspace } from "../../api/generated";
import AttacksIcon from "../../svg/attacks";
import WorkspaceAttacksDehashed from "./attacks/workspace-attacks-dehashed";
import WorkspaceAttacksPortScanTcp from "./attacks/workspace-attacks-port-scan-tcp";
import WorkspaceAttacksCT from "./attacks/workspace-attacks-certificate-transparency";
import WorkspaceAttacksHostAlive from "./attacks/workspace-attacks-host-alive";
import WorkspaceAttacksServiceDetection from "./attacks/workspace-attacks-service-detection";
import WorkspaceAttacksUdpServiceDetection from "./attacks/workspace-attacks-udp-service-detection";
import WorkspaceAttacksBruteforceSubdomains from "./attacks/workspace-attacks-bsd";
import { WORKSPACE_CONTEXT } from "./workspace";
import { Api } from "../../api/api";
import { handleApiError } from "../../utils/helper";
import CloseIcon from "../../svg/close";
import { ROUTES } from "../../routes";
import WorkspaceAttacksDnsResolution from "./attacks/workspace-attacks-dns-resolution";
import WorkspaceAttacksDnsTxtScan from "./attacks/workspace-attacks-dns-txt-scan";

export enum AttackCategory {
    Domains = "domains",
    Hosts = "hosts",
    Ports = "ports",
    Services = "services",
    Other = "other",
}

export enum AttackType {
    Dehashed = "dehashed",
    CertificateTransparency = "certificate_transparency",
    HostAlive = "host_alive",
    ServiceDetection = "service_detection",
    UdpServiceDetection = "udp_service_detection",
    Whois = "whois",
    BruteforceSubdomains = "bruteforce_subdomains",
    TcpCon = "tcp_con",
    DnsResolution = "dns_resolution",
    DnsTxtScan = "dns_txt_scan",
}

const ATTACKS: Record<
    AttackType,
    {
        /** A full name to show on hover */
        name: string;
        /** A short description to show on hover */
        description: string;
        /** The category this attack belongs to */
        category: AttackCategory;
        /** The React component which renders the form */
        form: React.JSXElementConstructor<{ prefilled: PrefilledAttackParams; targetType: TargetType | null }>;
    }
> = {
    bruteforce_subdomains: {
        name: "Bruteforce Subdomains",
        description: `Query a DNS server for all combinations of the given domain and the entries from the provided wordlist. 
        The entries of the wordlist will be prepended as subdomains.`,
        category: AttackCategory.Domains,
        form: WorkspaceAttacksBruteforceSubdomains,
    },
    certificate_transparency: {
        name: "Certificate Transparency",
        description: `Certificate transparency is a concept that was created to monitor the certificates that were signed by a CA.
        This attack will query the logs of a certificate transparency log collector to retrieve certificates with the given name in it.`,
        category: AttackCategory.Domains,
        form: WorkspaceAttacksCT,
    },
    dns_resolution: {
        name: "Dns Resolution",
        description: "Query a DNS server to resolve a given domain",
        category: AttackCategory.Domains,
        form: WorkspaceAttacksDnsResolution,
    },
    dns_txt_scan: {
        name: "DNS TXT Scan",
        description: "Scans the given domain's DNS TXT entries for known patterns",
        category: AttackCategory.Domains,
        form: WorkspaceAttacksDnsTxtScan,
    },
    whois: {
        name: "Whois",
        description: `Queries the RIPE database for the given IP address to find issued subnets for a company.`,
        category: AttackCategory.Hosts,
        form: () => <span>Not implemented yet</span>,
    },
    host_alive: {
        name: "Host alive",
        description: `Performs multiple scan techniques on an IP or a net to determine if a host is online.`,
        category: AttackCategory.Hosts,
        form: WorkspaceAttacksHostAlive,
    },
    tcp_con: {
        name: "TCP port scan",
        description: `Determine if a port is accepting TCP connections.`,
        category: AttackCategory.Ports,
        form: WorkspaceAttacksPortScanTcp,
    },
    service_detection: {
        name: "Service Detection",
        description: `Try to determine which service is running on a specific port.`,
        category: AttackCategory.Services,
        form: WorkspaceAttacksServiceDetection,
    },
    udp_service_detection: {
        name: "UDP Service Detection",
        description: `Try to determine which UDP service is running on a host on the given ports.`,
        category: AttackCategory.Services,
        form: WorkspaceAttacksUdpServiceDetection,
    },
    dehashed: {
        name: "Dehashed",
        description: `Dehashed provides an API to retrieve passwords (hashed and clear) and other information when querying a domain or an email.`,
        category: AttackCategory.Other,
        form: WorkspaceAttacksDehashed,
    },
};

const TARGET_TYPE = ["domain", "host", "port", "service"] as const;
/**
 * An attack target's type
 *
 * Used in combination with an uuid to identify an attack's target
 */
export type TargetType = (typeof TARGET_TYPE)[number];
export function TargetType(value: string): TargetType {
    // @ts-ignore
    if (TARGET_TYPE.indexOf(value) >= 0) return value;
    else throw Error(`Got invalid target type: ${value}`);
}

/** Set of attacks' parameters prefilled based on the target and passed to the attacks' forms */
export type PrefilledAttackParams = { domain?: string; ipAddr?: string; port?: number };

type WorkspaceAttacksProps =
    | {
          targetType?: never;
          targetUuid?: never;
      }
    | {
          targetType: TargetType;
          targetUuid: string;
      };

type WorkspaceAttacksState = {
    selectedAttack: AttackType | null;
    hoverAttack: AttackType | null;
    target: { name: string } & PrefilledAttackParams;
};

export default class WorkspaceAttacks extends React.Component<WorkspaceAttacksProps, WorkspaceAttacksState> {
    static contextType = WORKSPACE_CONTEXT;
    declare context: React.ContextType<typeof WORKSPACE_CONTEXT>;

    state: WorkspaceAttacksState = {
        selectedAttack: null,
        hoverAttack: null,
        target: { name: "Loading..." },
    };

    componentDidMount() {
        this.loadTarget();
    }

    componentDidUpdate(prevProps: Readonly<WorkspaceAttacksProps>) {
        if (this.props.targetType !== prevProps.targetType || this.props.targetUuid !== prevProps.targetUuid)
            this.loadTarget();
    }

    loadTarget() {
        switch (this.props.targetType) {
            case "domain":
                Api.workspaces.domains
                    .get(this.context.workspace.uuid, this.props.targetUuid)
                    .then(handleApiError(({ domain }) => this.setState({ target: { name: domain, domain } })));
                break;
            case "host":
                Api.workspaces.hosts
                    .get(this.context.workspace.uuid, this.props.targetUuid)
                    .then(handleApiError(({ ipAddr }) => this.setState({ target: { name: ipAddr, ipAddr } })));
                break;
            case "port":
                Api.workspaces.ports.get(this.context.workspace.uuid, this.props.targetUuid).then(
                    handleApiError(({ host: { ipAddr }, port }) =>
                        this.setState({
                            target: {
                                name: `${ipAddr}'s port ${port}`,
                                ipAddr,
                                port,
                            },
                        }),
                    ),
                );
                break;
            case "service":
                Api.workspaces.services.get(this.context.workspace.uuid, this.props.targetUuid).then(
                    handleApiError(({ name, host: { ipAddr }, port }) =>
                        this.setState({
                            target: {
                                name: port
                                    ? `${ipAddr}'s service ${name} on port ${port.port}`
                                    : `${ipAddr}'s service ${name}`,
                                ipAddr,
                                port: port?.port,
                            },
                        }),
                    ),
                );
                break;
            default:
                this.setState({ target: { name: "Loading..." } });
                break;
        }
    }

    render() {
        const { hoverAttack, selectedAttack } = this.state;

        const attackInfo = (hoverAttack && ATTACKS[hoverAttack]) || (selectedAttack && ATTACKS[selectedAttack]);
        const AttackForm = selectedAttack && ATTACKS[selectedAttack].form;

        const disabled: Partial<Record<AttackType, boolean>> = {};
        if ("targetType" in this.props) {
            if (this.props.targetType === "domain") {
                disabled.service_detection = true;
                disabled.udp_service_detection = true;
                disabled.whois = true;
            } else {
                disabled.bruteforce_subdomains = true;
                disabled.certificate_transparency = true;
                disabled.dns_resolution = true;
                disabled.dns_txt_scan = true;
            }
        }

        return (
            <div className={"workspace-attacks-container"}>
                <div className={"pane workspace-attacks-info"}>
                    <h2 className={"sub-heading"}>Attack Info</h2>
                    {attackInfo !== null ? (
                        <>
                            <h3 className={"heading"}>{attackInfo.name}</h3>
                            <span className={""}>{attackInfo.description}</span>
                        </>
                    ) : (
                        <div className={"workspace-attacks-info-empty"}>
                            <span>- Hover over an attack to display information -</span>
                        </div>
                    )}
                </div>
                <div className={"workspace-attacks-center-column"}>
                    {"targetType" in this.props ? (
                        <div className={"pane workspace-attacks-target"}>
                            <h2 className={"sub-heading"}>Attacking {this.state.target.name}</h2>
                            <button
                                className={"icon-button"}
                                type={"button"}
                                onClick={() => ROUTES.WORKSPACE_ATTACKS.visit({ uuid: this.context.workspace.uuid })}
                            >
                                <CloseIcon />
                            </button>
                        </div>
                    ) : null}
                    <div className={"pane workspace-attacks"}>
                        <AttacksIcon
                            onAttackHover={(hoverAttack) => this.setState({ hoverAttack })}
                            activeAttack={selectedAttack}
                            onAttackSelect={(selectedAttack) => this.setState({ selectedAttack })}
                            activeAttackCategory={hoverAttack && ATTACKS[hoverAttack].category}
                            disabled={disabled}
                        />
                    </div>
                </div>
                <div className={"pane workspace-attacks-details"}>
                    <h2 className={"sub-heading"}>Attack details</h2>
                    {AttackForm === null ? (
                        <div className={"workspace-attacks-details-empty"}>
                            <span> - Click on an attack to start - </span>
                        </div>
                    ) : (
                        <AttackForm prefilled={this.state.target} targetType={this.props.targetType || null} />
                    )}
                </div>
            </div>
        );
    }
}
