import { FullHttpService } from "../api/generated";

/**
 * Returns a human-readable URL representation for the given HTTP service.
 *
 * @param httpService the HTTP service to format to a string.
 * @param includeIp if true, include the IP address in the string
 *
 * @returns a URL representation for the given HTTP service.
 */
export function buildHttpServiceURL(httpService: FullHttpService, includeIp = true): string {
    const {
        host: { ipAddr },
        port: { port },
        domain,
        basePath,
        tls,
    } = httpService;
    const defaultPort = tls ? 443 : 80;

    const isIPv6 = ipAddr.includes(":");

    return (
        (tls ? "https://" : "http://") +
        (domain?.domain ?? (isIPv6 ? `[${ipAddr}]` : ipAddr)) +
        (port == defaultPort ? "" : ":" + port) +
        basePath +
        (includeIp && domain?.domain ? ` (on ${ipAddr})` : "")
    );
}
