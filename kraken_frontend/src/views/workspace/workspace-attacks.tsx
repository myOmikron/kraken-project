import React from "react";
import { Api } from "../../api/api";
import { ApiError } from "../../api/error";
import {
    AttacksApi,
    BruteforceSubdomainsRequest,
    DnsResolutionRequest,
    DnsTxtScanRequest,
    HostsAliveRequest,
    OsDetectionRequest,
    QueryCertificateTransparencyRequest,
    QueryDehashedRequest,
    ServiceDetectionRequest,
    UdpServiceDetectionRequest,
} from "../../api/generated";
import { ROUTES } from "../../routes";
import "../../styling/workspace-attacks.css";
import AttacksIcon from "../../svg/attacks";
import CloseIcon from "../../svg/close";
import { ObjectFns, handleApiError } from "../../utils/helper";
import { Result } from "../../utils/result";
import {
    AttackInputProps,
    BooleanAttackInput,
    DehashedAttackInput,
    DurationAttackInput,
    IAttackInputProps,
    NumberAttackInput,
    PortListInput,
    StringAttackInput,
    WordlistAttackInput,
} from "./attacks/attack-input";
import GenericAttackForm from "./attacks/generic-attack-form";
import { WORKSPACE_CONTEXT } from "./workspace";

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
    DnsResolution = "dns_resolution",
    DnsTxtScan = "dns_txt_scan",
    OsDetection = "os_detection",
}

type AttackRequestTypes = {
    [AttackType.Dehashed]: QueryDehashedRequest;
    [AttackType.CertificateTransparency]: QueryCertificateTransparencyRequest;
    [AttackType.HostAlive]: HostsAliveRequest;
    [AttackType.ServiceDetection]: ServiceDetectionRequest;
    [AttackType.UdpServiceDetection]: UdpServiceDetectionRequest;
    [AttackType.BruteforceSubdomains]: BruteforceSubdomainsRequest;
    [AttackType.DnsResolution]: DnsResolutionRequest;
    [AttackType.DnsTxtScan]: DnsTxtScanRequest;
    [AttackType.OsDetection]: OsDetectionRequest;
};

export interface IAttackInput {
    label: string;
    /// If true, the value is wrapped as a single-element array. When prefilled,
    /// may contain more than one value.
    /// If false and prefilled with more than one value, multiple requests will
    /// be sent out, one for each value.
    multi?: boolean;
    required?: boolean;
    defaultValue: any;
    prefill?: (keyof PrefilledAttackParams)[];
    type: React.FC<IAttackInputProps>;
    group?: undefined | string;
    renderProps?: React.HTMLProps<HTMLElement>;
    /// Called for prefilled inputs, to adjust prefilled value (e.g. primitive
    /// string or number) to expected input type (e.g. port range)
    preprocess?: (v: any) => any;
}

export interface AttackInput<T, IsMulti extends boolean> extends IAttackInput {
    multi: IsMulti;
    defaultValue: T | undefined;
    type: React.FC<AttackInputProps<T>>;
    preprocess?: (v: any) => T | undefined;
}

export type AttackInputs<ReqType extends AttackType> = {
    // see IAttackDescr for docs

    endpoint: keyof AttacksApi;
    jsonKey: string;
    inputs: {
        [T in Exclude<keyof AttackRequestTypes[ReqType], "workspaceUuid" | "leechUuid">]:
            | (AttackRequestTypes[ReqType][T] extends readonly (infer ElementType)[]
                  ? AttackInput<ElementType, boolean>
                  : AttackInput<AttackRequestTypes[ReqType][T], false>)
            | { fixed: AttackRequestTypes[ReqType][T] };
    };
};

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
         */
        endpoint: keyof AttacksApi;
        /**
         * What the key inside the `[AttackName]OperationRequest` is called
         * (first parameter to `AttacksApi.endpoint`)
         *
         * See `src/api/generated/apis/AttacksApi.ts`
         */
        jsonKey: string;
        /**
         * Describes all the available inputs on the request object how to
         * process and send them.
         */
        inputs: {
            [index: string]: IAttackInput | { fixed: any };
        };
    };
}

export interface AttackDescr<ReqType extends AttackType> extends IAttackDescr {
    /** The React component which renders the form */
    inputs: AttackInputs<ReqType>;
}

export type AllAttackDescr = {
    [T in AttackType]: AttackDescr<T>;
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
                    prefill: ["domain"],
                    type: StringAttackInput,
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
                },
            },
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
                    prefill: ["domain"],
                    type: StringAttackInput,
                },
                includeExpired: {
                    label: "Include expired certificates",
                    multi: false,
                    defaultValue: false,
                    type: BooleanAttackInput,
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
                },
            },
        },
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
                    prefill: ["domain"],
                    required: true,
                },
            },
        },
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
                    prefill: ["domain"],
                    required: true,
                },
            },
        },
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
                    label: "Timeout (in ms)",
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
                },
            },
        },
    },
    service_detection: {
        name: "Service Detection",
        description:
            "Scan a port range on a collection of hosts for open TCP ports and detect the services running on them",
        category: AttackCategory.Services,
        inputs: {
            endpoint: "serviceDetection",
            jsonKey: "serviceDetectionRequest",
            inputs: {
                targets: {
                    label: "Domain / IP / net in CIDR",
                    multi: true,
                    defaultValue: undefined,
                    prefill: ["domain", "ipAddr"],
                    type: StringAttackInput,
                    required: true,
                },
                ports: {
                    label: "Ports",
                    multi: false,
                    required: true,
                    defaultValue: ["1-65535"],
                    prefill: ["port"],
                    type: PortListInput,
                    preprocess: (v) => (typeof v == "number" ? [v] : v),
                },
                connectTimeout: {
                    label: "Connect Timeout (in ms)",
                    multi: false,
                    defaultValue: 1000,
                    required: true,
                    type: DurationAttackInput,
                    group: "Advanced",
                },
                receiveTimeout: {
                    label: "Receive Timeout (in ms)",
                    multi: false,
                    defaultValue: 500,
                    type: DurationAttackInput,
                    required: true,
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
                },
                concurrentLimit: {
                    label: "Concurrency Limit",
                    multi: false,
                    defaultValue: 500,
                    required: true,
                    type: NumberAttackInput,
                    group: "Advanced",
                },
                skipIcmpCheck: {
                    label: "Skip icmp check",
                    multi: false,
                    defaultValue: true,
                    type: BooleanAttackInput,
                },
            },
        },
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
                    prefill: ["ipAddr"],
                    required: true,
                    type: StringAttackInput,
                },
                ports: {
                    label: "Ports",
                    multi: false,
                    required: true,
                    defaultValue: ["1-65535"],
                    prefill: ["port"],
                    type: PortListInput,
                    preprocess: (v) => (typeof v == "number" ? [v] : v),
                },
                timeout: {
                    label: "Timeout (in ms)",
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
                },
            },
        },
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
                    prefill: ["domain", "ipAddr"],
                },
            },
        },
    },
    os_detection: {
        name: "OS detection",
        description: `Attempt to guess the operating system of the remote host.\n\nIf SSH port is non-empty, the SSH banner on that port will be checked.\n\nA fingerprint port can be force set to perform fingerprinting on the given port.`,
        category: AttackCategory.Hosts,
        inputs: {
            endpoint: "osDetection",
            jsonKey: "osDetectionRequest",
            inputs: {
                address: {
                    defaultValue: "",
                    prefill: ["ipAddr"],
                    label: "IP",
                    type: StringAttackInput,
                    multi: false,
                    required: true,
                },
                sshPort: {
                    defaultValue: 22,
                    label: "SSH Port",
                    multi: false,
                    type: NumberAttackInput as any,
                    required: false,
                },
                fingerprintPort: {
                    defaultValue: undefined,
                    prefill: ["port"],
                    label: "TCP Fingerprint Port",
                    type: NumberAttackInput as any,
                    multi: false,
                },
                fingerprintTimeout: {
                    group: "TCP fingerprint task",
                    defaultValue: 5000,
                    label: "Timeout",
                    type: DurationAttackInput,
                    required: true,
                    multi: false,
                },
                sshTimeout: {
                    group: "SSH task",
                    defaultValue: 5000,
                    label: "Timeout",
                    type: DurationAttackInput,
                    required: true,
                    multi: false,
                },
                sshConnectTimeout: {
                    group: "SSH task",
                    defaultValue: 2500,
                    label: "Connection timeout",
                    type: DurationAttackInput,
                    required: true,
                    multi: false,
                },
                portAckTimeout: {
                    group: "TCP SYN port test",
                    defaultValue: 2000,
                    label: "ACK timeout",
                    type: DurationAttackInput,
                    required: true,
                    multi: false,
                },
                portParallelSyns: {
                    group: "TCP SYN port test",
                    defaultValue: 8,
                    label: "Max parallel requests",
                    type: NumberAttackInput,
                    required: true,
                    multi: false,
                },
            },
        },
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
      }
    | {
          targetType: "selection";
          domains: string[];
          hosts: string[];
          ports: string[];
          services: string[];
      };

type WorkspaceAttacksState = {
    selectedAttack: AttackType | null;
    hoverAttack: AttackType | null;
    target: { name: string; prefills: PrefilledAttackParams[] };
};

export default class WorkspaceAttacks extends React.Component<WorkspaceAttacksProps, WorkspaceAttacksState> {
    static contextType = WORKSPACE_CONTEXT;
    declare context: React.ContextType<typeof WORKSPACE_CONTEXT>;

    state: WorkspaceAttacksState = {
        selectedAttack: null,
        hoverAttack: null,
        target: { name: "Loading...", prefills: [] },
    };

    componentDidMount() {
        this.loadTarget();
    }

    componentDidUpdate(prevProps: Readonly<WorkspaceAttacksProps>) {
        if (this.props.targetType !== prevProps.targetType) {
            if ("targetUuid" in this.props && "targetUuid" in prevProps) {
                if (this.props.targetUuid !== prevProps.targetUuid) this.loadTarget();
            } else if ("domains" in this.props && "domains" in prevProps) {
                if (
                    !ObjectFns.deepEquals(
                        [this.props.domains, this.props.hosts, this.props.ports, this.props.services],
                        [prevProps.domains, prevProps.hosts, prevProps.ports, prevProps.services],
                    )
                )
                    this.loadTarget();
            } else {
                this.loadTarget();
            }
        }
    }

    loadTarget() {
        switch (this.props.targetType) {
            case "domain":
                Api.workspaces.domains
                    .get(this.context.workspace.uuid, this.props.targetUuid)
                    .then(
                        handleApiError(({ domain }) =>
                            this.setState({ target: { name: domain, prefills: [{ domain }] } }),
                        ),
                    );
                break;
            case "host":
                Api.workspaces.hosts
                    .get(this.context.workspace.uuid, this.props.targetUuid)
                    .then(
                        handleApiError(({ ipAddr }) =>
                            this.setState({ target: { name: ipAddr, prefills: [{ ipAddr }] } }),
                        ),
                    );
                break;
            case "port":
                Api.workspaces.ports.get(this.context.workspace.uuid, this.props.targetUuid).then(
                    handleApiError(({ host: { ipAddr }, port }) =>
                        this.setState({
                            target: {
                                name: `${ipAddr}'s port ${port}`,
                                prefills: [
                                    {
                                        ipAddr,
                                        port,
                                    },
                                ],
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
                                prefills: [
                                    {
                                        ipAddr,
                                        port: port?.port,
                                    },
                                ],
                            },
                        }),
                    ),
                );
                break;
            case "selection":
                this.updateSelection();
                this.setState({
                    target: {
                        name: [
                            `${this.props.hosts.length} hosts`,
                            `${this.props.ports.length} ports`,
                            `${this.props.domains.length} domains`,
                            `${this.props.services.length} services`,
                        ]
                            .filter((s) => !s.startsWith("0 "))
                            .join(", "),
                        prefills: [],
                    },
                });
                break;
            default:
                this.setState({ target: { name: "Loading...", prefills: [] } });
                break;
        }
    }

    async updateSelection() {
        if (this.props.targetType != "selection") throw new Error("invalid state");

        let workspaceUuid = this.context.workspace.uuid;

        function fetchAll<T>(
            api: { get: (workspaceUuid: string, thingUuid: string) => Promise<Result<T, ApiError>> },
            list: string[],
        ): Promise<T[]> {
            return new Promise((resolve, reject) => {
                let res: T[] = [];

                function checkDone() {
                    if (res.length == list.length) {
                        resolve(res);
                    }
                }

                checkDone();
                list.forEach((item) => {
                    api.get(workspaceUuid, item)
                        .then(
                            handleApiError((v) => {
                                res.push(v);
                                checkDone();
                            }),
                        )
                        .catch((v) => {
                            console.error(v);
                            reject("failed looking up item " + item);
                        });
                });
            });
        }

        let inputs: { [group: string]: PrefilledAttackParams[] } = {
            hosts: (await fetchAll(Api.workspaces.hosts, this.props.hosts)).map((v) => ({ ipAddr: v.ipAddr })),
            ports: (await fetchAll(Api.workspaces.ports, this.props.ports)).map((v) => ({
                ipAddr: v.host.ipAddr,
                port: v.port,
            })),
            domains: (await fetchAll(Api.workspaces.domains, this.props.domains)).map((v) => ({ domain: v.domain })),
            services: (await fetchAll(Api.workspaces.services, this.props.services)).map((v) => ({
                ipAddr: v.host.ipAddr,
                port: v.port?.port,
            })),
        };

        let prefills: PrefilledAttackParams[] = Object.keys(inputs).flatMap((k) => inputs[k]);

        this.setState({
            target: {
                name: this.state.target.name,
                prefills: prefills,
            },
        });
    }

    renderSelection() {
        if (!this.state.target?.prefills?.length) return <></>;
        const columnLabels: { [P in keyof PrefilledAttackParams]: string } = {
            domain: "Domain",
            ipAddr: "Host",
            port: "Port",
        };
        let columns: { [P in keyof PrefilledAttackParams]: boolean } = {
            domain: true,
            ipAddr: true,
            port: true,
        };
        for (const k of Object.keys(columns)) {
            (columns as any)[k] = this.state.target.prefills.some((p) => (p as any)[k] !== undefined);
        }
        return (
            <div className="pane selection">
                <h2 className={"sub-heading"}>Selection</h2>
                <table>
                    <thead>
                        <tr>
                            {Object.keys(columns).map(
                                (c) => (columns as any)[c] && <th>{(columnLabels as any)[c]}</th>,
                            )}
                        </tr>
                    </thead>
                    <tbody>
                        {this.state.target.prefills.map((row: any) => (
                            <tr>
                                {Object.keys(columns).map(
                                    (c) =>
                                        (columns as any)[c] && (
                                            <td>
                                                {row[c] === undefined ? (
                                                    <em>n/a</em>
                                                ) : (
                                                    <pre>
                                                        {typeof row[c] === "string" ? row[c] : JSON.stringify(row[c])}
                                                    </pre>
                                                )}
                                            </td>
                                        ),
                                )}
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>
        );
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
                <div className={"workspace-attacks-info"}>
                    <div className={"pane"}>
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
                    {this.renderSelection()}
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
                        <GenericAttackForm
                            key={"attack_form_" + selectedAttack}
                            prefilled={this.state.target.prefills}
                            attack={AttackForm}
                        />
                    )}
                </div>
            </div>
        );
    }
}
