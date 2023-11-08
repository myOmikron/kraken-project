import { SimpleAggregationSource } from "../../../api/generated";
import React from "react";

type SourcesListProps = {
    sources: SimpleAggregationSource;
};
export default function SourcesList(props: SourcesListProps) {
    return (
        <div className={"bubble-list"}>
            {Object.entries(props.sources)
                .filter(([_, count]) => count > 0)
                .map(
                    // @ts-ignore
                    ([attackType, count]: [keyof SimpleAggregationSource, number]) => (
                        <div className={"bubble bubble-primary"} title={ATTACK_NAME[attackType]}>
                            {ATTACK_ABRV[attackType]} {count}
                        </div>
                    ),
                )}
        </div>
    );
}

const ATTACK_ABRV: { [attackType in keyof SimpleAggregationSource]: string } = {
    bruteforceSubdomains: "BSd",
    tcpPortScan: "PsT",
    queryCertificateTransparency: "CT",
    queryDehashed: "Dh",
    hostAlive: "HA",
    serviceDetection: "SvD",
    dnsResolution: "DR",
};
const ATTACK_NAME: { [attackType in keyof SimpleAggregationSource]: string } = {
    bruteforceSubdomains: "Bruteforce Subdomains",
    tcpPortScan: "TCP port scan",
    queryCertificateTransparency: "Certificate Transparency",
    queryDehashed: "Dehashed",
    hostAlive: "Host alive",
    serviceDetection: "Service Detection",
    dnsResolution: "DNS Resolution",
};
