import { SimpleAggregationSource } from "../../../api/generated";
import React from "react";
import { ObjectFns } from "../../../utils/helper";
import Bubble from "../../../components/bubble";

type SourcesListProps = {
    sources: SimpleAggregationSource;
};
export default function SourcesList(props: SourcesListProps) {
    return (
        <div className={"bubble-list"}>
            {ObjectFns.entries(ATTACKS).map(([key, [abrv, name]]) =>
                props.sources[key] > 0 ? (
                    <Bubble color={"primary"} name={`${abrv} ${props.sources[key]}`} title={name} />
                ) : null,
            )}
            {props.sources.manual ? <Bubble name={"MI"} title={"Manually inserted"} /> : null}
        </div>
    );
}

const ATTACKS = {
    bruteforceSubdomains: ["BSd", "Bruteforce Subdomains"],
    tcpPortScan: ["PsT", "TCP port scan"],
    queryCertificateTransparency: ["CT", "Certificate Transparency"],
    queryDehashed: ["Dh", "Dehashed"],
    hostAlive: ["HA", "Host alive"],
    serviceDetection: ["SvD", "Service Detection"],
    dnsResolution: ["DR", "DNS Resolution"],
};
