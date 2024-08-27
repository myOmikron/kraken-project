import React from "react";
import { Api } from "../../api/api";
import {
    AttacksApi,
    BruteforceSubdomainsRequest,
    DnsResolutionRequest,
    DnsTxtScanRequest,
    FullDomain,
    FullHost,
    FullHttpService,
    FullPort,
    FullService,
    HostsAliveRequest,
    OsDetectionRequest,
    PortProtocol,
    QueryCertificateTransparencyRequest,
    QueryDehashedRequest,
    ServiceDetectionRequest,
    TestSSLRequest,
    UdpServiceDetectionRequest,
} from "../../api/generated";
import { ROUTES } from "../../routes";
import "../../styling/workspace-attacks.css";
import AttacksIcon from "../../svg/attacks";
import CloseIcon from "../../svg/close";
import { ObjectFns, handleApiError } from "../../utils/helper";
import { buildHttpServiceURL } from "../../utils/http-services";
import {
    AttackInputProps,
    BooleanAttackInput,
    DehashedAttackInput,
    DurationAttackInput,
    NullNumberAttackInput,
    NumberAttackInput,
    PortListInput,
    StartTLSAttackInput,
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
    TestSSL = "testssl",
}

/**
 * Given an AttackType (used everywhere as key), maps to the API request type.
 */
/* eslint-disable jsdoc/require-jsdoc */
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
    [AttackType.TestSSL]: TestSSLRequest;
};

/* eslint-enable jsdoc/require-jsdoc */

export interface IAttackInput {
    /**
     * Human readable name for this input field.
     */
    label: string;
    /**
     * If true, the value is wrapped as a single-element array. When prefilled,
     * may contain more than one value.
     * If false and prefilled with more than one value, multiple requests will
     * be sent out, one for each value.
     */
    multi?: boolean;
    /**
     * If true, the value must be set before submitting is possible. Depending
     * on the type this can mean different things, but usually should mean that
     * empty values are not valid for required fields.
     *
     * `undefined` is always treated as missing in required fields and won't
     * include the field in the API.
     */
    required?: boolean;
    // any instead of T, defined below in AttackInput<T> when known
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    defaultValue: any;
    /**
     * Optional prefill types to automatically fill this field with data based
     * on the selection.
     */
    prefill?: PrefillType[];
    // The ref can be on anything - the GenericAttackForm checks for functions,
    // such as "focus", by name
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    type: React.FC<AttackInputProps<any> & React.RefAttributes<any>>;
    /**
     * A human readable group name. Any string is valid and all equal group
     * names will be combined into one drop-down. The group named "Advanced" is
     * special in that it will be collapsed by default. All other groups are
     * expanded by default.
     */
    group?: undefined | string;
    /**
     * Extra props that will be passed to the instantiated `type` input
     * component as-is.
     */
    renderProps?: React.HTMLProps<HTMLElement>;
    /**
     * Called for prefilled inputs, to adjust prefilled value (e.g. primitive
     * string or number) to expected input type (e.g. port range)
     */
    // TODO: the preprocess type is dependent on the value(s) of "this.prefill"
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    preprocess?: (v: any) => any;
}

export type AttackPrefill = { [key: string]: AnyPrefill[] };

export interface AttackInput<T, IsMulti extends boolean> extends IAttackInput {
    multi: IsMulti;
    defaultValue: T | undefined;
    // The ref can be on anything - the GenericAttackForm checks for functions,
    // such as "focus", by name
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    type: React.FC<AttackInputProps<T> & React.RefAttributes<any>>;
    // TODO: the preprocess type is dependent on the value(s) of "this.prefill"
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    preprocess?: (v: any) => T | undefined;
}

export type AnyAttackInput = {
    [index: string]: IAttackInput;
};

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
            [index: string]:
                | IAttackInput
                | {
                      /**
                       * always a fixed value that is sent to the API without being able to be edited by the user.
                       */
                      fixed: AnyApiValue;
                  };
        };
    };
}

/**
 * This is all JSON values that can be sent via the API through the various
 * inputs. Used as common type for the combining runtime code.
 */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type AnyApiValue = any;

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
                    prefill: ["port[Tcp]"],
                    type: PortListInput,
                    // eslint-disable-next-line jsdoc/require-jsdoc
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
                    defaultValue: ["1-6000"],
                    prefill: ["port[Udp]"],
                    type: PortListInput,
                    // eslint-disable-next-line jsdoc/require-jsdoc
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
                    prefill: ["raw"],
                    // eslint-disable-next-line jsdoc/require-jsdoc
                    preprocess: (v: RawSelectionData | undefined) => {
                        if (!v) return undefined;
                        if (v.domain) return { domain: { simple: v.domain.domain } };
                        if (v.httpService?.domain) return { domain: { simple: v.httpService.domain.domain } };
                        if (v.httpService) return { ipAddress: { simple: v.httpService.host.ipAddr } };
                        if (v.host) return { ipAddress: { simple: v.host.ipAddr } };
                        if (v.port) return { ipAddress: { simple: v.port.host.ipAddr } };
                        if (v.service) return { ipAddress: { simple: v.service.host.ipAddr } };
                        return undefined;
                    },
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
                targets: {
                    label: "Domain / IP / net in CIDR",
                    multi: true,
                    defaultValue: undefined,
                    prefill: ["domain", "ipAddr"],
                    type: StringAttackInput,
                    required: true,
                },
                sshPort: {
                    defaultValue: 22,
                    label: "SSH Port",
                    prefill: ["service[ssh].port"],
                    multi: false,
                    type: NullNumberAttackInput,
                    required: false,
                },
                fingerprintPort: {
                    defaultValue: undefined,
                    prefill: ["port[Tcp]"],
                    label: "TCP Fingerprint Port",
                    type: NullNumberAttackInput,
                    multi: false,
                    required: false,
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
                concurrentLimit: {
                    label: "Concurrency Limit",
                    multi: false,
                    defaultValue: 32,
                    required: true,
                    type: NumberAttackInput,
                    group: "Advanced",
                },
            },
        },
    },
    testssl: {
        name: "Testssl",
        description: "Run testssl.sh to check a services tls configuration",
        category: AttackCategory.Ports,
        inputs: {
            endpoint: "testssl",
            jsonKey: "testSSLRequest",
            inputs: {
                uri: {
                    label: "Domain",
                    multi: false,
                    defaultValue: "",
                    required: true,
                    prefill: ["domain"],
                    type: StringAttackInput,
                },
                host: {
                    label: "IP",
                    multi: false,
                    defaultValue: "",
                    prefill: ["ipAddr"],
                    type: StringAttackInput,
                    required: true,
                },
                port: {
                    label: "Port",
                    multi: false,
                    defaultValue: 443,
                    prefill: ["port[Tcp]"],
                    type: NumberAttackInput,
                    required: true,
                },
                connectTimeout: {
                    label: "Connect Timeout (in s)",
                    multi: false,
                    defaultValue: 5,
                    type: NullNumberAttackInput,
                    group: "Advanced",
                    required: true,
                },
                opensslTimeout: {
                    label: "OpenSSL Timeout (in s)",
                    multi: false,
                    defaultValue: 5,
                    type: NullNumberAttackInput,
                    group: "Advanced",
                    required: true,
                },
                basicAuth: { fixed: undefined },
                starttls: {
                    label: "StartTLS Protocol",
                    multi: false,
                    defaultValue: undefined,
                    type: StartTLSAttackInput,
                    group: "Advanced",
                    required: false,
                },
            },
        },
    },
};

const TARGET_TYPE = ["domain", "host", "port", "service", "httpService"] as const;
/**
 * An attack target's type
 *
 * Used in combination with an uuid to identify an attack's target
 */
export type TargetType = (typeof TARGET_TYPE)[number];

/**
 * Verifies the runtime string is an available TargetType, throwing on error.
 *
 * @param value the string that is expected to be one of TARGET_TYPE.
 * @returns `value` itself if it is valid, otherwise throws an error
 */
export function TargetType(value: string): TargetType {
    // @ts-ignore: TargetType is by definition anything inside TARGET_TYPE which is just a list of "special strings"
    if (TARGET_TYPE.indexOf(value) >= 0) return value;
    else throw Error(`Got invalid target type: ${value}`);
}

/** Set of attacks' parameters prefilled based on the target and passed to the attacks' forms */
export type RawSelectionData = {
    domain?: FullDomain;
    host?: FullHost;
    port?: FullPort;
    service?: FullService;
    httpService?: FullHttpService;
};

/**
 * All available prefill types that the attack metadata can specify for auto-filling
 * with selection attacks.
 */
export type PrefillType =
    | "raw" // receive full PrefilledAttackParams
    | "domain" // domain (from any aggregation kind relating to domains)
    | "ipAddr" // ip address (e.g. from host, port or service)
    | "port" // port (e.g. from port or service)
    | `port[${PortProtocol}]` // TCP/UDP/... only port (e.g. from port or service)
    | "service.name" // service name (from any aggregation kind relating to services)
    | `service[${string}].port` // port (only from a service with the name within the square brackets)
    | `service[!${string}].port` // port (only from a service not with the name within the square brackets)
    | `service[${string}][${PortProtocol}].port` // TCP/UDP/... only port (only from a service with the name within the square brackets)
    | "httpService.name" // http service name (from any aggregation kind relating to services)
    | "httpService.path" // http service base path
    | "httpService.port" // http service port
    | "httpService.domain" // http service domain (if set)
    | "httpService.ipAddr"; // http service ip address

/**
 * Props for the <WorkspaceAttacks> component.
 */
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
          httpServices: string[];
      };

/**
 * The full workspace attacks page, includes attack selector, description and
 * attack form.
 */
export default function WorkspaceAttacks(props: WorkspaceAttacksProps) {
    const {
        workspace: { uuid: workspace },
    } = React.useContext(WORKSPACE_CONTEXT);

    const [selectedAttack, setSelectedAttack] = React.useState<AttackType | null>(null);
    const [hoverAttack, setHoverAttack] = React.useState<AttackType | null>(null);
    const [target, setTarget] = React.useState<{ name: string; selection: RawSelectionData[] }>({
        name: "Loading...",
        selection: [],
    });

    function loadTarget() {
        switch (props.targetType) {
            case "domain":
                Api.workspaces.domains.get(workspace, props.targetUuid).then(
                    handleApiError((domain) =>
                        setTarget({
                            name: domain.domain,
                            selection: [{ domain }],
                        }),
                    ),
                );
                break;
            case "host":
                Api.workspaces.hosts.get(workspace, props.targetUuid).then(
                    handleApiError((host) =>
                        setTarget({
                            name: host.ipAddr,
                            selection: [{ host }],
                        }),
                    ),
                );
                break;
            case "port":
                Api.workspaces.ports.get(workspace, props.targetUuid).then(
                    handleApiError((port) =>
                        setTarget({
                            name: `${port.host.ipAddr}'s port ${port.port}`,
                            selection: [{ port }],
                        }),
                    ),
                );
                break;
            case "service":
                Api.workspaces.services.get(workspace, props.targetUuid).then(
                    handleApiError((service) => {
                        const { name, host, port } = service;
                        setTarget({
                            name: port
                                ? `${host.ipAddr}'s service ${name} on port ${port.port}`
                                : `${host.ipAddr}'s service ${name}`,
                            selection: [{ service }],
                        });
                    }),
                );
                break;
            case "httpService":
                Api.workspaces.httpServices.get(workspace, props.targetUuid).then(
                    handleApiError((httpService) => {
                        const { name } = httpService;
                        setTarget({
                            name: `HTTP service ${name} on ${buildHttpServiceURL(httpService)}`,
                            selection: [{ httpService }],
                        });
                    }),
                );
                break;
            case "selection":
                setTarget({
                    name: [
                        `${props.hosts.length} hosts`,
                        `${props.ports.length} ports`,
                        `${props.domains.length} domains`,
                        `${props.services.length} services`,
                        `${props.httpServices.length} http services`,
                    ]
                        .filter((s) => !s.startsWith("0 "))
                        .join(", "),
                    selection: [],
                });

                const selection: RawSelectionData[] = [];
                Promise.all([
                    ...props.hosts.map((itemUuid) =>
                        Api.workspaces.hosts
                            .get(workspace, itemUuid)
                            .then(handleApiError((host) => selection.push({ host }))),
                    ),
                    ...props.ports.map((itemUuid) =>
                        Api.workspaces.ports
                            .get(workspace, itemUuid)
                            .then(handleApiError((port) => selection.push({ port }))),
                    ),
                    ...props.domains.map((itemUuid) =>
                        Api.workspaces.domains
                            .get(workspace, itemUuid)
                            .then(handleApiError((domain) => selection.push({ domain }))),
                    ),
                    ...props.services.map((itemUuid) =>
                        Api.workspaces.services
                            .get(workspace, itemUuid)
                            .then(handleApiError((service) => selection.push({ service }))),
                    ),
                    ...props.httpServices.map((itemUuid) =>
                        Api.workspaces.httpServices
                            .get(workspace, itemUuid)
                            .then(handleApiError((httpService) => selection.push({ httpService }))),
                    ),
                ]).then(() =>
                    setTarget((target) => ({
                        name: target.name,
                        selection: selection,
                    })),
                );

                break;
            default:
                setTarget({ name: "Loading...", selection: [] });
                break;
        }
    }

    React.useEffect(() => {
        loadTarget();
    }, [
        "targetUuid" in props ? props.targetUuid : undefined,
        "domains" in props ? props.domains : undefined,
        "hosts" in props ? props.hosts : undefined,
        "ports" in props ? props.ports : undefined,
        "services" in props ? props.services : undefined,
        "httpServices" in props ? props.httpServices : undefined,
        props.targetType,
    ]);

    function renderSelection() {
        if (!target?.selection?.length) return <></>;
        const attack = (hoverAttack || selectedAttack) as AttackType;
        if (!attack) return <></>;
        const values = generateAttackPrefill(attack, target.selection);
        const keys = Object.keys(values);
        if (!keys) return <></>;
        const columnLabels = keys.map((k) => (ATTACKS[attack].inputs.inputs as AnyAttackInput)[k].label);
        const rows = values[keys[0]].map((_, i) => (
            <tr key={attack + "_row" + i}>
                {keys.map((k) => (
                    <td>{values[k][i] === undefined ? <em>n/a</em> : <pre>{JSON.stringify(values[k][i])}</pre>}</td>
                ))}
            </tr>
        ));
        return (
            <div className="pane selection">
                <h2 className={"sub-heading"}>Selection</h2>
                <table>
                    <thead>
                        <tr>
                            {columnLabels.map((label) => (
                                <th>{label}</th>
                            ))}
                        </tr>
                    </thead>
                    <tbody>{rows}</tbody>
                </table>
            </div>
        );
    }

    const attackInfo = (hoverAttack && ATTACKS[hoverAttack]) || (selectedAttack && ATTACKS[selectedAttack]);
    const AttackForm = selectedAttack && ATTACKS[selectedAttack];

    const disabled: Record<AttackType, boolean> = generateDisabled(target.selection);

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
                {renderSelection()}
            </div>
            <div className={"workspace-attacks-center-column"}>
                {"targetType" in props ? (
                    <div className={"pane workspace-attacks-target"}>
                        <h2 className={"sub-heading"}>Attacking {target.name}</h2>
                        <button
                            className={"icon-button"}
                            type={"button"}
                            onClick={() => ROUTES.WORKSPACE_ATTACKS.visit({ uuid: workspace })}
                        >
                            <CloseIcon />
                        </button>
                    </div>
                ) : null}
                <div className={"pane workspace-attacks"}>
                    <AttacksIcon
                        onAttackHover={setHoverAttack}
                        activeAttack={selectedAttack}
                        onAttackSelect={setSelectedAttack}
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
                        prefilled={generateAttackPrefill(selectedAttack!, target.selection)}
                        attack={AttackForm}
                    />
                )}
            </div>
        </div>
    );
}

function generateDisabled(prefill: RawSelectionData[]): Record<AttackType, boolean> {
    return Object.fromEntries(
        Object.keys(ATTACKS).map((v) => [
            v,
            prefill.length &&
                Object.values(generateAttackPrefill(v as AttackType, prefill)).every((v) => v.length == 0),
        ]),
    ) as { [T in keyof typeof ATTACKS]: boolean };
}

function generateAttackPrefill(attack: AttackType, prefill: RawSelectionData[]): AttackPrefill {
    const ret: AttackPrefill = {};
    for (const key of Object.keys(ATTACKS[attack].inputs.inputs)) {
        const input: IAttackInput = (ATTACKS[attack].inputs.inputs as AnyAttackInput)[key];
        if (typeof input === "object" && !Array.isArray(input) && input.prefill) {
            ret[key] = [];
        }
    }

    // first generate all the raw data
    for (const row of prefill) {
        for (const key of Object.keys(ret)) {
            const input: IAttackInput = (ATTACKS[attack].inputs.inputs as AnyAttackInput)[key];
            let data = getFirstPrefill(row, input.prefill!);
            if (input.preprocess) data = input.preprocess(data);
            ret[key].push(data);
        }
    }

    // then eliminate columns that are completely undefined
    const toDeleteKeys = Object.keys(ret).filter((key) => ret[key].every((value) => value === undefined));
    for (const d of toDeleteKeys) {
        delete ret[d];
    }

    // now deduplicate rows
    const entries = Object.entries(ret);
    const keys = entries.map((e) => e[0]);
    let values = entries.map((e) => e[1]);
    values = ObjectFns.transpose2D(values);
    values = ObjectFns.uniqueObjects(values);
    // and remove rows that have missing required values
    values = values.filter(
        (row) =>
            !row.some((v, i) => v === undefined && (ATTACKS[attack].inputs.inputs as AnyAttackInput)[keys[i]].required),
    );
    values = ObjectFns.transpose2D(values);
    if (keys.length != values.length) throw new Error("logic error");

    return Object.fromEntries(keys.map((k, i) => [k, values[i]]));
}

/**
 * All available prefill types that can be generated by the getPrefill function.
 */
type AnyPrefill = RawSelectionData | string | number | undefined;

/**
 * Calls `getPrefill` on each available type and returns the first non-undefined result.
 *
 * @param raw The data row to generate the prefill object for.
 * @param types The types to check in order.
 * @returns prefill, if available, or undefined
 */
export function getFirstPrefill(raw: RawSelectionData, types: PrefillType[]): AnyPrefill {
    for (const p of types) {
        const v = getPrefill(raw, p);
        if (v) return v;
    }
    return undefined;
}

export function getPrefill(raw: RawSelectionData, type: "raw"): RawSelectionData;
export function getPrefill(raw: RawSelectionData, type: "domain"): string | undefined;
export function getPrefill(raw: RawSelectionData, type: "ipAddr"): string | undefined;
export function getPrefill(raw: RawSelectionData, type: "port"): number | undefined;
export function getPrefill(raw: RawSelectionData, type: "port[Unknown]"): number | undefined;
export function getPrefill(raw: RawSelectionData, type: "port[Udp]"): number | undefined;
export function getPrefill(raw: RawSelectionData, type: "port[Tcp]"): number | undefined;
export function getPrefill(raw: RawSelectionData, type: "port[Sctp]"): number | undefined;
export function getPrefill(raw: RawSelectionData, type: "service.name"): string | undefined;
export function getPrefill(raw: RawSelectionData, type: `service[${string}].port`): number | undefined;
export function getPrefill(raw: RawSelectionData, type: `service[${string}][Unknown].port`): number | undefined;
export function getPrefill(raw: RawSelectionData, type: `service[${string}][Udp].port`): number | undefined;
export function getPrefill(raw: RawSelectionData, type: `service[${string}][Tcp].port`): number | undefined;
export function getPrefill(raw: RawSelectionData, type: `service[${string}][Sctp].port`): number | undefined;
export function getPrefill(raw: RawSelectionData, type: "httpService.name"): string | undefined;
export function getPrefill(raw: RawSelectionData, type: "httpService.path"): string | undefined;
export function getPrefill(raw: RawSelectionData, type: "httpService.port"): number | undefined;
export function getPrefill(raw: RawSelectionData, type: "httpService.domain"): string | undefined;
export function getPrefill(raw: RawSelectionData, type: "httpService.ipAddr"): string | undefined;
export function getPrefill(raw: RawSelectionData, type: PrefillType): never | undefined;

/**
 * Generates prefill data for attack form inputs, which will also be sent as-is
 * for API requests.
 *
 * @param raw The data row to generate a prefill object for.
 * @param type The wanted prefill type to attempt to generate prefill data for.
 *
 * @returns undefined if no prefill is available for the requested type or a
 * value where its type is based on the `type` parameter. (see overloads)
 */
export function getPrefill(raw: RawSelectionData, type: PrefillType): unknown | undefined {
    switch (type) {
        case "raw":
            return raw;
        case "domain":
            return raw.domain?.domain ?? raw.httpService?.domain?.domain ?? undefined;
        case "ipAddr":
            return (
                raw.host?.ipAddr ??
                raw.port?.host.ipAddr ??
                raw.service?.host.ipAddr ??
                raw.httpService?.host.ipAddr ??
                undefined
            );
        case "port":
        case "port[Unknown]":
        case "port[Udp]":
        case "port[Tcp]":
        case "port[Sctp]":
            const p = raw.port ?? raw.service?.port ?? raw.httpService?.port ?? undefined;
            if (p) {
                if (type.startsWith("port[") && type.endsWith("]")) {
                    if (p.protocol != type.substring(5, type.length - 1)) return undefined;
                }
                return p.port;
            }
            return undefined;
        case "service.name":
            return raw.service?.name;
        case "httpService.name":
            return raw.httpService?.name;
        case "httpService.path":
            return raw.httpService?.basePath;
        case "httpService.port":
            return raw.httpService?.port.port;
        case "httpService.domain":
            return raw.httpService?.domain?.domain;
        case "httpService.ipAddr":
            return raw.httpService?.host.ipAddr;
        default:
            if (type.startsWith("service[")) {
                if (raw.service) {
                    const [service, remaining] = type.substring(8).split("]", 2);
                    if (
                        service.startsWith("!")
                            ? raw.service.name.toLowerCase() == service.substring(1).toLowerCase()
                            : raw.service.name.toLowerCase() != service.toLowerCase()
                    )
                        return undefined;
                    switch (remaining) {
                        case ".port":
                            return getPrefill({ service: raw.service }, "port");
                        case "[Unknown].port":
                            return getPrefill({ service: raw.service }, "port[Unknown]");
                        case "[Udp].port":
                            return getPrefill({ service: raw.service }, "port[Udp]");
                        case "[Tcp].port":
                            return getPrefill({ service: raw.service }, "port[Tcp]");
                        case "[Sctp].port":
                            return getPrefill({ service: raw.service }, "port[Sctp]");
                    }
                }
            }
            return undefined;
    }
}
