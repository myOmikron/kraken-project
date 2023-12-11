import { AttackType } from "../api/generated";

export type AttackResolver = {
    [Key in AttackType]: {
        abbreviation: string;
        long: string;
    };
};

export const ATTACKS: AttackResolver = {
    BruteforceSubdomains: { abbreviation: "BSd", long: "Bruteforce Subdomains" },
    TcpPortScan: { abbreviation: "PsT", long: "TCP port scan" },
    QueryCertificateTransparency: { abbreviation: "CT", long: "Certificate Transparency" },
    QueryUnhashed: { abbreviation: "Dh", long: "Dehashed" },
    HostAlive: { abbreviation: "HA", long: "Host alive" },
    ServiceDetection: { abbreviation: "SvD", long: "Service Detection" },
    DnsResolution: { abbreviation: "DR", long: "DNS Resolution" },
    ForcedBrowsing: { abbreviation: "FB", long: "Forced Browsing" },
    OSDetection: { abbreviation: "OS", long: "OS Detection" },
    AntiPortScanningDetection: { abbreviation: "APs", long: "Anti port-scanning detection" },
    UdpPortScan: { abbreviation: "PsU", long: "UDP port scan" },
    VersionDetection: { abbreviation: "VsD", long: "Version detection" },
    Undefined: { abbreviation: "?", long: "Undefined" },
};
