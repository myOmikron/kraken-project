import {
    FullDomain,
    FullHost,
    FullPort,
    FullService,
    SimpleDomain,
    SimpleHost,
    SimplePort,
    SimpleService,
} from "../api/generated";

export function compareDomain(a: FullDomain | SimpleDomain, b: FullDomain | SimpleDomain): number {
    return a.domain.localeCompare(b.domain);
}

const IPv4 = /^(\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})$/;
export function compareHost(a: FullHost | SimpleHost, b: FullHost | SimpleHost): number {
    const av4 = IPv4.exec(a.ipAddr);
    const bv4 = IPv4.exec(b.ipAddr);
    if (av4 && !bv4) return -1;
    if (!av4 && bv4) return 1;
    if (av4 && bv4) {
        const num = (ip: RegExpExecArray): number =>
            ((parseInt(ip[1]) << 24) | (parseInt(ip[2]) << 16) | (parseInt(ip[3]) << 8) | parseInt(ip[4])) >>> 0;
        return num(av4) - num(bv4);
    } else {
        // TODO: IPv6 sorting
        return a.ipAddr.localeCompare(b.ipAddr);
    }
}

export function compareService(a: FullService | SimpleService, b: FullService | SimpleService): number {
    if (a.name != b.name) return a.name.localeCompare(b.name);
    if (
        typeof a.host != "string" &&
        typeof b.host != "string" &&
        typeof a.port != "string" &&
        typeof b.port != "string"
    )
        return compareHost(a.host, b.host) || (a.port?.port ?? 0) - (b.port?.port ?? 0);
    else return 0;
}

export function comparePort(a: FullPort | SimplePort, b: FullPort | SimplePort): number {
    if (a.port != b.port) return a.port - b.port;
    if (typeof a.host != "string" && typeof b.host != "string") return compareHost(a.host, b.host);
    else return 0;
}
