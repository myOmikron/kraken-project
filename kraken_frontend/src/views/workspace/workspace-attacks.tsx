import React from "react";
import "../../styling/workspace-attacks.css";
import { AttacksApi, BruteforceSubdomainsRequest, DnsResolutionRequest, DnsTxtScanRequest, FullDomain, FullHost, FullPort, FullService, FullWorkspace, HostsAliveRequest, QueryCertificateTransparencyRequest, QueryDehashedRequest, ScanTcpPortsRequest, ServiceDetectionRequest, UdpServiceDetectionRequest } from "../../api/generated";
import AttacksIcon from "../../svg/attacks";
import { WORKSPACE_CONTEXT } from "./workspace";
import { Api } from "../../api/api";
import { handleApiError } from "../../utils/helper";
import CloseIcon from "../../svg/close";
import { ROUTES } from "../../routes";
import Input from "../../components/input";
import Checkbox from "../../components/checkbox";
import GenericAttackForm from "./attacks/generic-attack-form";
import { AttackInputProps, BooleanAttackInput, DehashedAttackInput, DurationAttackInput, IAttackInputProps, NumberAttackInput, PortListInput, StringAttackInput, WordlistAttackInput } from "./attacks/attack-input"

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
    BruteforceSubdomains = "bruteforce_subdomains",
    TcpCon = "tcp_con",
    DnsResolution = "dns_resolution",
    DnsTxtScan = "dns_txt_scan",
}

type AttackRequestTypes = {
    [AttackType.Dehashed]: QueryDehashedRequest,
    [AttackType.CertificateTransparency]: QueryCertificateTransparencyRequest,
    [AttackType.HostAlive]: HostsAliveRequest,
    [AttackType.ServiceDetection]: ServiceDetectionRequest,
    [AttackType.UdpServiceDetection]: UdpServiceDetectionRequest,
    [AttackType.BruteforceSubdomains]: BruteforceSubdomainsRequest,
    [AttackType.TcpCon]: ScanTcpPortsRequest,
    [AttackType.DnsResolution]: DnsResolutionRequest,
    [AttackType.DnsTxtScan]: DnsTxtScanRequest,
};

export interface IAttackInput {
    label: string,
    /// If true, the value is wrapped as a single-element array. When prefilled,
    /// may contain more than one value.
    /// If false and prefilled with more than one value, multiple requests will
    /// be sent out, one for each value.
    multi?: boolean,
    required?: boolean,
    defaultValue: any,
    prefill?: keyof PrefilledAttackParams | (keyof PrefilledAttackParams)[],
    type: (props: IAttackInputProps) => React.JSX.Element,
    group?: undefined | string;
    renderProps?: React.HTMLProps<HTMLElement>;
}

export interface AttackInput<T, IsMulti extends boolean> extends IAttackInput {
    multi: IsMulti;
    defaultValue: T | undefined,
    type: (props: AttackInputProps<T>) => React.JSX.Element,
}

export type AttackInputs<ReqType extends AttackType> = {
    // see IAttackDescr for docs

    endpoint: keyof AttacksApi,
    jsonKey: string,
    inputs: {
        [T in Exclude<keyof AttackRequestTypes[ReqType], "workspaceUuid" | "leechUuid">]:
            (AttackRequestTypes[ReqType][T] extends readonly (infer ElementType)[]
                ? AttackInput<ElementType, boolean>
                : AttackInput<AttackRequestTypes[ReqType][T], false>
            ) | { fixed: AttackRequestTypes[ReqType][T] }
    };
}

export interface IAttackDescr {
    /** A full name to show on hover */
    name: string;
    /** A short description to show on hover */
    description: string;
    /** The category this attack belongs to */
    category: AttackCategory;
    /** */
    inputs: {
        /**
         * Which API to call, on the raw API
         * */
        endpoint: keyof AttacksApi,
        /**
         * What the key inside the `[AttackName]OperationRequest` is called
         * (first parameter to `AttacksApi.endpoint`)
         * */
        jsonKey: string,
        /**
         * Describes all the available inputs on the request object how to
         * process and send them.
         * */
        inputs: {
            [index: string]: IAttackInput | { fixed: any }
        }
    }
}

export interface AttackDescr<ReqType extends AttackType> extends IAttackDescr {
    /** The React component which renders the form */
    inputs: AttackInputs<ReqType>;
}

export type AllAttackDescr = {
    [T in AttackType]: AttackDescr<T>
};

const ATTACKS: AllAttackDescr = {
    bruteforce_subdomains: {
        name: "Bruteforce Subdomains",
        description: `Query a DNS server for all combinations of the given domain and the entries from the provided wordlist. 
        The entries of the wordlist will be prepended as subdomains.`,
        category: AttackCategory.Domains,
        inputs: {
            endpoint: "bruteforceSubdomains",
            jsonKey: "bruteforceSubdomainsRequest",
            inputs: {
                domain: {
                    label: "Domain",
                    multi: false,
                    required: true,
                    defaultValue: "",
                    prefill: "domain",
                    type: StringAttackInput
                },
                wordlistUuid: {
                    label: "Wordlist",
                    multi: false,
                    required: true,
                    defaultValue: "",
                    type: WordlistAttackInput,
                },
                concurrentLimit: {
                    label: "Concurrency Limit",
                    multi: false,
                    defaultValue: 100,
                    required: true,
                    type: NumberAttackInput,
                    group: "Advanced",
                }
            }
        },
    },
    certificate_transparency: {
        name: "Certificate Transparency",
        description: `Certificate transparency is a concept that was created to monitor the certificates that were signed by a CA.
        This attack will query the logs of a certificate transparency log collector to retrieve certificates with the given name in it.`,
        category: AttackCategory.Domains,
        inputs: {
            endpoint: "queryCertificateTransparency",
            jsonKey: "queryCertificateTransparencyRequest",
            inputs: {
                target: {
                    label: "Domain",
                    multi: false,
                    defaultValue: "",
                    required: true,
                    prefill: "domain",
                    type: StringAttackInput
                },
                includeExpired: {
                    label: "Include expired certificates",
                    multi: false,
                    defaultValue: false,
                    type: BooleanAttackInput
                },
                maxRetries: {
                    label: "Max. no. of retries",
                    multi: false,
                    defaultValue: 3,
                    required: true,
                    type: NumberAttackInput,
                    group: "Advanced",
                },
                retryInterval: {
                    label: "Retry interval",
                    multi: false,
                    defaultValue: 500,
                    required: true,
                    type: DurationAttackInput,
                    group: "Advanced",
                }
            }
        }
    },
    dns_resolution: {
        name: "Dns Resolution",
        description: "Query a DNS server to resolve a given domain",
        category: AttackCategory.Domains,
        inputs: {
            endpoint: "dnsResolution",
            jsonKey: "dnsResolutionRequest",
            inputs: {
                concurrentLimit: {
                    fixed: 1,
                },
                targets: {
                    label: "Domain",
                    multi: true,
                    defaultValue: undefined,
                    type: StringAttackInput,
                    prefill: "domain",
                    required: true,
                }
            }
        }
    },
    dns_txt_scan: {
        name: "DNS TXT Scan",
        description: "Scans the given domain's DNS TXT entries for known patterns",
        category: AttackCategory.Domains,
        inputs: {
            endpoint: "dnsTxtScan",
            jsonKey: "dnsTxtScanRequest",
            inputs: {
                targets: {
                    label: "Domain",
                    multi: true,
                    defaultValue: undefined,
                    type: StringAttackInput,
                    prefill: "domain",
                    required: true,
                }
            }
        }
    },
    host_alive: {
        name: "Host alive",
        description: `Performs multiple scan techniques on an IP or a net to determine if a host is online.`,
        category: AttackCategory.Hosts,
        inputs: {
            endpoint: "hostsAliveCheck",
            jsonKey: "hostsAliveRequest",
            inputs: {
                targets: {
                    label: "Domain / IP / net in CIDR",
                    multi: true,
                    defaultValue: undefined,
                    prefill: ["domain", "ipAddr"],
                    type: StringAttackInput,
                    required: true,
                },
                timeout: {
                    label: "Timeout",
                    multi: false,
                    defaultValue: 1000,
                    type: DurationAttackInput,
                    required: true,
                    group: "Advanced",
                },
                concurrentLimit: {
                    label: "Concurrency Limit",
                    multi: false,
                    defaultValue: 50,
                    type: NumberAttackInput,
                    required: true,
                    group: "Advanced",
                }
            }
        }
    },
    tcp_con: {
        name: "TCP port scan",
        description: `Determine if a port is accepting TCP connections.`,
        category: AttackCategory.Ports,
        inputs: {
            endpoint: "scanTcpPorts",
            jsonKey: "scanTcpPortsRequest",
            inputs: {
                targets: {
                    label: "Domain / IP / net in CIDR",
                    multi: true,
                    defaultValue: undefined,
                    prefill: ["domain", "ipAddr"],
                    type: StringAttackInput,
                    required: true
                },
                ports: {
                    label: "Ports",
                    multi: false,
                    required: true,
                    defaultValue: ["1-65535"],
                    prefill: "port",
                    type: PortListInput,
                },
                skipIcmpCheck: {
                    label: "Skip icmp check",
                    multi: false,
                    defaultValue: false,
                    type: BooleanAttackInput,
                },
                timeout: {
                    label: "Timeout",
                    multi: false,
                    defaultValue: 1000,
                    required: true,
                    type: DurationAttackInput,
                    group: "Advanced",
                },
                concurrentLimit: {
                    label: "Concurrency Limit",
                    multi: false,
                    defaultValue: 500,
                    required: true,
                    type: NumberAttackInput,
                    group: "Advanced",
                },
                maxRetries: {
                    label: "Max. no. of retries",
                    multi: false,
                    defaultValue: 6,
                    required: true,
                    type: NumberAttackInput,
                    group: "Advanced",
                },
                retryInterval: {
                    label: "Retry interval",
                    multi: false,
                    defaultValue: 100,
                    required: true,
                    type: DurationAttackInput,
                    group: "Advanced",
                }
            }
        }
    },
    service_detection: {
        name: "Service Detection",
        description: `Try to determine which service is running on a specific port.`,
        category: AttackCategory.Services,
        inputs: {
            endpoint: "serviceDetection",
            jsonKey: "serviceDetectionRequest",
            inputs: {
                address: {
                    label: "IP",
                    multi: false,
                    defaultValue: "",
                    required: true,
                    type: StringAttackInput,
                    prefill: "ipAddr",
                },
                port: {
                    label: "Port",
                    multi: false,
                    defaultValue: undefined,
                    type: NumberAttackInput,
                    prefill: "port",
                    required: true,
                },
                timeout: {
                    label: "Timeout",
                    multi: false,
                    defaultValue: 500,
                    type: DurationAttackInput,
                    required: true,
                    group: "Advanced"
                }
            }
        }
    },
    udp_service_detection: {
        name: "UDP Service Detection",
        description: `Try to determine which UDP service is running on a host on the given ports.`,
        category: AttackCategory.Services,
        inputs: {
            endpoint: "udpServiceDetection",
            jsonKey: "udpServiceDetectionRequest",
            inputs: {
                address: {
                    multi: false,
                    label: "IP",
                    defaultValue: "",
                    prefill: "ipAddr",
                    required: true,
                    type: StringAttackInput,
                },
                ports: {
                    label: "Ports",
                    multi: false,
                    required: true,
                    defaultValue: ["1-65535"],
                    prefill: "port",
                    type: PortListInput,
                },
                timeout: {
                    label: "Timeout",
                    multi: false,
                    defaultValue: 1000,
                    type: DurationAttackInput,
                    group: "Advanced",
                    required: true,
                },
                maxRetries: {
                    label: "Max. no. of retries",
                    multi: false,
                    defaultValue: 5,
                    type: NumberAttackInput,
                    group: "Advanced",
                    required: true,
                },
                retryInterval: {
                    label: "Retry interval",
                    multi: false,
                    defaultValue: 1000,
                    type: DurationAttackInput,
                    group: "Advanced",
                    required: true,
                },
                concurrentLimit: {
                    label: "Concurrency Limit",
                    multi: false,
                    defaultValue: 1024,
                    type: NumberAttackInput,
                    group: "Advanced",
                    required: true,
                }
            }
        }
    },
    dehashed: {
        name: "Dehashed",
        description: `Dehashed provides an API to retrieve passwords (hashed and clear) and other information when querying a domain or an email.`,
        category: AttackCategory.Other,
        inputs: {
            endpoint: "queryDehashed",
            jsonKey: "queryDehashedRequest",
            inputs: {
                query: {
                    label: "Query",
                    multi: false,
                    defaultValue: undefined,
                    type: DehashedAttackInput,
                    prefill: ["domain", "ipAddr"]
                }
            }
        }
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
        const AttackForm = selectedAttack && ATTACKS[selectedAttack];

        const disabled: Partial<Record<AttackType, boolean>> = {};
        if ("targetType" in this.props) {
            if (this.props.targetType === "domain") {
                disabled.service_detection = true;
                disabled.udp_service_detection = true;
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
                    <h2 className={"sub-heading"}>{AttackForm?.name ?? "Attack settings"}</h2>
                    {AttackForm === null ? (
                        <div className={"workspace-attacks-details-empty"}>
                            <span> - Click on an attack to start - </span>
                        </div>
                    ) : (
                        <GenericAttackForm key={"attack_form_" + selectedAttack} prefilled={this.state.target} attack={AttackForm} targetType={this.props.targetType || null} />
                    )}
                </div>
            </div>
        );
    }
}
