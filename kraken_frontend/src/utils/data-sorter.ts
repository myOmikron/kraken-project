/**
 * Compare functions to use as argument in {@link Array.sort}
 *
 * @module
 */

import {
    AggregationType,
    FullDomain,
    FullHost,
    FullPort,
    FullService,
    SimpleDomain,
    SimpleHost,
    SimplePort,
    SimpleService,
} from "../api/generated";
import { FullHttpService } from "../api/generated/models/FullHttpService";
import { SimpleHttpService } from "../api/generated/models/SimpleHttpService";

/**
 * The ordering of aggregation types, lower number means it comes first.
 */
export const aggregationTypeOrdering: { [K in AggregationType]: number } = {
    Domain: 0,
    Host: 1,
    Port: 2,
    Service: 3,
    HttpService: 4,
};

/**
 * Sorting predicate for domains.
 *
 * Domains are sorted exclusively by domain name.
 *
 * @param a A domain
 * @param b Another domain
 * @returns negative number if should be sorted before, positive number if should be sorted after.
 */
export function compareDomain(a: FullDomain | SimpleDomain, b: FullDomain | SimpleDomain): number {
    return a.domain.localeCompare(b.domain);
}

const IPv4 = /^(\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})$/;

/**
 * Sorting predicate for hosts.
 *
 * Hosts are sorted exclusively by ip address. IPv4 is sorted before IPv6. IPv6
 * sorting currently uses literal comparison instead of sorting by bytes.
 *
 * @param a A host
 * @param b Another host
 * @returns negative number if should be sorted before, positive number if should be sorted after.
 */
export function compareHost(a: FullHost | SimpleHost, b: FullHost | SimpleHost): number {
    const av4 = IPv4.exec(a.ipAddr);
    const bv4 = IPv4.exec(b.ipAddr);
    if (av4 && !bv4) return -1;
    if (!av4 && bv4) return 1;
    if (av4 && bv4) {
        // eslint-disable-next-line jsdoc/require-param, jsdoc/require-returns
        /** Converts the matched ipv4 address into an integer */
        const num = (ip: RegExpExecArray): number =>
            ((parseInt(ip[1]) << 24) | (parseInt(ip[2]) << 16) | (parseInt(ip[3]) << 8) | parseInt(ip[4])) >>> 0;
        return num(av4) - num(bv4);
    } else {
        // TODO: IPv6 sorting
        return a.ipAddr.localeCompare(b.ipAddr);
    }
}

/**
 * Sorting predicate for services.
 *
 * Services are sorted by name first, then host, then port.
 *
 * @param a A service
 * @param b Another service
 * @returns negative number if should be sorted before, positive number if should be sorted after.
 */
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

/**
 * Sorting predicate for ports.
 *
 * Ports are sorted by port number, then host.
 *
 * @param a A port
 * @param b Another port
 * @returns negative number if should be sorted before, positive number if should be sorted after.
 */
export function comparePort(a: FullPort | SimplePort, b: FullPort | SimplePort): number {
    if (a.port != b.port) return a.port - b.port;
    if (typeof a.host != "string" && typeof b.host != "string") return compareHost(a.host, b.host);
    else return 0;
}

/**
 * Sorting predicate for http services.
 *
 * HTTP services are sorted by name, then domain, then host, then port.
 *
 * @param a A http service
 * @param b Another http service
 * @returns negative number if should be sorted before, positive number if should be sorted after.
 */
export function compareHttpService(
    a: FullHttpService | SimpleHttpService,
    b: FullHttpService | SimpleHttpService,
): number {
    if (a.name != b.name) return a.name.localeCompare(b.name);
    if (typeof a.domain != "string" && typeof b.domain != "string" && a.domain?.domain != b.domain?.domain)
        return (a.domain?.domain ?? "").localeCompare(b.domain?.domain ?? "");
    if (
        typeof a.host != "string" &&
        typeof b.host != "string" &&
        typeof a.port != "string" &&
        typeof b.port != "string"
    )
        return compareHost(a.host, b.host) || (a.port?.port ?? 0) - (b.port?.port ?? 0);
    else return 0;
}
