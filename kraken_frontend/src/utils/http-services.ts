import { FullHttpService } from "../api/generated";

export function buildHttpServiceURL(httpService: FullHttpService) {
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
        basePath
    );
}
