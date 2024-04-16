import { AttackType, SimpleAggregationSource } from "../api/generated";

/**
 * Type of {@link ATTACKS}
 *
 * This type is spelled out instead of being inferred (`typeof ATTACKS`),
 * to enable the typescript compiler to complain about missing values.
 */
export type AttackResolver = {
    [Key in AttackType]: {
        /** Short name consisting only of a few characters */
        abbreviation: string;
        /** Long name a non-programmer would write */
        long: string;
        /** Key into {@link SimpleAggregationSource} */
        key: keyof SimpleAggregationSource | "undefined";
    };
};

/**
 * Lookup object which associates {@link AttackType}s with strings
 * - `abbreviation`: Short name consisting only of a few characters
 * - `long`: Long name a non-programmer would write
 * - `key`: Key into {@link SimpleAggregationSource}
 */
export const ATTACKS: AttackResolver = {
    BruteforceSubdomains: { abbreviation: "BSd", long: "Bruteforce Subdomains", key: "bruteforceSubdomains" },
    QueryCertificateTransparency: {
        abbreviation: "CT",
        long: "Certificate Transparency",
        key: "queryCertificateTransparency",
    },
    QueryUnhashed: { abbreviation: "Dh", long: "Dehashed", key: "queryDehashed" },
    HostAlive: { abbreviation: "HA", long: "Host alive", key: "hostAlive" },
    ServiceDetection: { abbreviation: "SvD", long: "Service Detection", key: "serviceDetection" },
    UdpServiceDetection: { abbreviation: "UDP", long: "UDP Service Detection", key: "udpServiceDetection" },
    DnsResolution: { abbreviation: "DR", long: "DNS Resolution", key: "dnsResolution" },
    DnsTxtScan: { abbreviation: "Txt", long: "DNS TXT Scan", key: "dnsTxtScan" },
    ForcedBrowsing: { abbreviation: "FB", long: "Forced Browsing", key: "forcedBrowsing" },
    OSDetection: { abbreviation: "OS", long: "OS Detection", key: "osDetection" },
    AntiPortScanningDetection: {
        abbreviation: "APs",
        long: "Anti port-scanning detection",
        key: "antiPortScanningDetection",
    },
    UdpPortScan: { abbreviation: "PsU", long: "UDP port scan", key: "udpPortScan" },
    VersionDetection: { abbreviation: "VsD", long: "Version detection", key: "versionDetection" },
    Undefined: { abbreviation: "?", long: "Undefined", key: "undefined" },
};
