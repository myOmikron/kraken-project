import React from "react";
import { SourceAttack } from "../../../api/generated";
import "../../../styling/workspace-data-details.css";
import ArrowLeftIcon from "../../../svg/arrow-left";
import ArrowRightIcon from "../../../svg/arrow-right";
import CopyIcon from "../../../svg/copy";
import { copyToClipboard } from "../../../utils/helper";
import OsIcon from "../../../components/os-icon";

type WorkspaceDataDetailsResultsProps = {
    attacks: Array<SourceAttack>;
};

export default function WorkspaceDataDetailsResults(props: WorkspaceDataDetailsResultsProps) {
    const { attacks } = props;
    const [attackPage, setAttackPage] = React.useState(0);
    const [resultPage, setResultPage] = React.useState(0);

    if (attacks.length === 0) {
        return null;
    }
    const clampedAttackPage = attackPage < 0 ? 0 : attackPage >= attacks.length ? attacks.length - 1 : attackPage;
    const attack = attacks[clampedAttackPage];

    const clampedResultPage =
        resultPage < 0 ? 0 : resultPage >= attack.results.length ? attack.results.length - 1 : resultPage;
    const resultUuid = attack.results[clampedResultPage].uuid;

    const attackElement = (() => {
        switch (attack.attackType) {
            case "DnsResolution":
                const dnsResult = attack.results.at(clampedResultPage);
                if (dnsResult === undefined) return null;
                return (
                    <div className="workspace-data-details-container">
                        <div className={"workspace-data-details-pane"}>
                            <h3 className={"sub-heading"}>DNS Resolution</h3>
                            <div className={"workspace-data-details-list"}>
                                <div className="workspace-data-details-list-elements">
                                    <span>Input:</span>
                                    <span>Started by:</span>
                                    <span>Created:</span>
                                    <span>Finished:</span>
                                </div>
                                <div className="workspace-data-details-list-elements">
                                    <span>{dnsResult.source}</span>
                                    <span>{attack.startedBy.displayName}</span>
                                    <span>{formatDate(attack.createdAt)}</span>
                                    <span>{formatDate(attack.finishedAt)}</span>
                                </div>
                            </div>
                        </div>
                        <div className={"workspace-data-details-pane-layout"}>
                            <div className={"workspace-data-details-pane"}>
                                <h3 className={"sub-heading"}>{dnsResult.dnsRecordType}</h3>
                                <span className={"workspace-data-details-text-wrap"}>{dnsResult.destination}</span>
                            </div>
                            <div className={"workspace-data-details-pane"}>
                                <h3 className={"sub-heading"}>Date</h3>
                                <span>{formatDate(dnsResult.createdAt)}</span>
                            </div>
                        </div>
                    </div>
                );
            case "DnsTxtScan":
                const txtResult = attack.results.at(clampedResultPage);
                if (txtResult === undefined) return null;
                return (
                    <div className="workspace-data-details-container">
                        <div className={"workspace-data-details-pane"}>
                            <h3 className={"sub-heading"}>DNS_TXT::{txtResult.collectionType}</h3>
                            <div className={"workspace-data-details-list"}>
                                <div className="workspace-data-details-list-elements">
                                    <span>Domain:</span>
                                    <span>Started by:</span>
                                    <span>Created:</span>
                                    <span>Finished:</span>
                                </div>
                                <div className="workspace-data-details-list-elements">
                                    <span>{txtResult.domain}</span>
                                    <span>{attack.startedBy.displayName}</span>
                                    <span>{formatDate(attack.createdAt)}</span>
                                    <span>{formatDate(attack.finishedAt)}</span>
                                </div>
                            </div>
                        </div>
                        <div className={"workspace-data-details-pane-layout"}>
                            {txtResult.entries.map((entry) => (
                                <>
                                    <div className={"workspace-data-details-pane wide"}>
                                        {"serviceHint" in entry && entry["serviceHint"] ? (
                                            <>
                                                <h3 className={"sub-heading"}>{entry.serviceHint.txtType}</h3>
                                                <span className={"workspace-data-details-text-wrap"}>
                                                    {entry.serviceHint.rule}
                                                </span>
                                            </>
                                        ) : "spf" in entry && entry["spf"] ? (
                                            <>
                                                <div className={"workspace-data-details-list"}>
                                                    <h3 className={"sub-heading"}>{entry.spf.spfType}</h3>
                                                    <span className={"workspace-data-details-text-wrap"}>
                                                        {entry.spf.rule}
                                                    </span>
                                                </div>
                                                {entry.spf.spfDomain && (
                                                    <>
                                                        <hr />
                                                        <div className={"workspace-data-details-list"}>
                                                            <h3 className={"sub-heading"}>
                                                                parsed
                                                                {entry.spf.spfDomain && entry.spf.spfIp && " (domain) "}
                                                            </h3>
                                                            <div>
                                                                <span>{entry.spf.spfDomain}</span>
                                                                <span>
                                                                    {entry.spf.spfDomainIpv4Cidr &&
                                                                        "/" + entry.spf.spfDomainIpv4Cidr}
                                                                </span>
                                                                <span>
                                                                    {entry.spf.spfDomainIpv6Cidr &&
                                                                        "//" + entry.spf.spfDomainIpv6Cidr}
                                                                </span>
                                                            </div>
                                                        </div>
                                                    </>
                                                )}
                                                {entry.spf.spfIp && (
                                                    <>
                                                        <hr />
                                                        <div className={"workspace-data-details-list"}>
                                                            <h3 className={"sub-heading"}>
                                                                parsed
                                                                {entry.spf.spfDomain && entry.spf.spfIp && " (ip) "}
                                                            </h3>
                                                            <span>{entry.spf.spfIp}</span>
                                                        </div>
                                                    </>
                                                )}
                                            </>
                                        ) : undefined}
                                    </div>
                                </>
                            ))}
                        </div>
                    </div>
                );
            case "QueryCertificateTransparency":
                const qctResult = attack.results.at(clampedResultPage);
                if (qctResult === undefined) return null;
                return (
                    <div className="workspace-data-details-container">
                        <div className={"workspace-data-details-pane"}>
                            <h3 className={"sub-heading"}>Query Certificate Transparency</h3>
                            <div className={"workspace-data-details-list"}>
                                <div className="workspace-data-details-list-elements">
                                    <span>Started by:</span>
                                    <span>Created:</span>
                                    <span>Finished:</span>
                                </div>
                                <div className="workspace-data-details-list-elements">
                                    <span>{attack.startedBy.displayName}</span>
                                    <span>{formatDate(attack.createdAt)}</span>
                                    <span>{formatDate(attack.finishedAt)}</span>
                                </div>
                            </div>
                        </div>
                        <div className={"workspace-data-details-pane-layout"}>
                            <div className={"workspace-data-details-pane"}>
                                <h3 className={"sub-heading"}>Serial Number</h3>
                                <span className={"workspace-data-details-text-wrap"}>{qctResult.serialNumber}</span>
                            </div>
                            <div className={"workspace-data-details-pane"}>
                                <h3 className={"sub-heading"}>Date</h3>
                                <span>{formatDate(qctResult.createdAt)}</span>
                            </div>
                        </div>
                        <div className={"workspace-data-details-pane"}>
                            <h3 className={"sub-heading"}>Issuer</h3>
                            <div className={"workspace-data-details-list"}>
                                <div className="workspace-data-details-list-elements">
                                    <span>Issuer Name:</span>
                                    <span>Common Name:</span>
                                </div>
                                <div className="workspace-data-details-list-elements">
                                    <span>{qctResult.issuerName}</span>
                                    <span>{qctResult.commonName}</span>
                                </div>
                            </div>
                        </div>
                        <div className={"workspace-data-details-pane"}>
                            <h3 className={"sub-heading"}>Validity</h3>
                            <div className={"workspace-data-details-list"}>
                                <div className="workspace-data-details-list-elements">
                                    <span>Issued On:</span>
                                    <span>Expires On:</span>
                                </div>
                                <div className="workspace-data-details-list-elements">
                                    <span>{formatDate(qctResult.notBefore)}</span>
                                    <span>{formatDate(qctResult.notAfter)}</span>
                                </div>
                            </div>
                        </div>
                        <div className={"workspace-data-details-pane"}>
                            <h3 className={"sub-heading"}>Values</h3>
                            <div className="workspace-data-details-list-elements">
                                {qctResult.valueNames.map((v) => {
                                    return <span>{v}</span>;
                                })}
                            </div>
                        </div>
                    </div>
                );
            case "BruteforceSubdomains":
                const bsResult = attack.results.at(clampedResultPage);
                if (bsResult === undefined) return null;
                return (
                    <div className="workspace-data-details-container">
                        <div className={"workspace-data-details-pane"}>
                            <h3 className={"sub-heading"}>Bruteforce Subdomain</h3>
                            <div className={"workspace-data-details-list"}>
                                <div className="workspace-data-details-list-elements">
                                    <span>Start domain:</span>
                                    <span>Started by:</span>
                                    <span>Created:</span>
                                    <span>Finished:</span>
                                </div>
                                <div className="workspace-data-details-list-elements">
                                    <span>{bsResult.source}</span>
                                    <span>{attack.startedBy.displayName}</span>
                                    <span>{formatDate(attack.createdAt)}</span>
                                    <span>{formatDate(attack.finishedAt)}</span>
                                </div>
                            </div>
                        </div>
                        <div className={"workspace-data-details-pane"}>
                            <h3 className={"sub-heading"}>Date</h3>
                            <span>{formatDate(bsResult.createdAt)}</span>
                        </div>
                        <div className={"workspace-data-details-pane"}>
                            <h3 className={"sub-heading"}>DNS</h3>
                            <div className={"workspace-data-details-list"}>
                                <div className="workspace-data-details-list-elements">
                                    <span>{bsResult.dnsRecordType.toUpperCase()}</span>
                                </div>
                                <div className="workspace-data-details-list-elements">
                                    <span>{bsResult.destination}</span>
                                </div>
                            </div>
                        </div>
                    </div>
                );
            case "HostAlive":
                const haResult = attack.results.at(clampedResultPage);
                if (haResult === undefined) return null;
                return (
                    haResult && (
                        <div className="workspace-data-details-container">
                            <div className={"workspace-data-details-pane"}>
                                <h3 className={"sub-heading"}>Host alive </h3>
                                <div className={"workspace-data-details-list"}>
                                    <div className="workspace-data-details-list-elements">
                                        <span>Started IP:</span>
                                        <span>Started by:</span>
                                        <span>Created:</span>
                                        <span>Finished:</span>
                                    </div>
                                    <div className="workspace-data-details-list-elements">
                                        <span>{haResult.host}</span>
                                        <span>{attack.startedBy.displayName}</span>
                                        <span>{formatDate(attack.createdAt)}</span>
                                        <span>{formatDate(attack.finishedAt)}</span>
                                    </div>
                                </div>
                            </div>
                            <div className={"workspace-data-details-pane-layout"}>
                                <div className={"workspace-data-details-pane"}>
                                    <h3 className={"sub-heading"}>Date</h3>
                                    <span>{formatDate(haResult.createdAt)}</span>
                                </div>
                            </div>
                        </div>
                    )
                );
            case "ServiceDetection":
            case "UdpServiceDetection":
                const sdResult = attack.results.at(clampedResultPage);
                if (sdResult === undefined) return null;
                return (
                    <div className="workspace-data-details-container">
                        <div className={"workspace-data-details-pane"}>
                            <h3 className={"sub-heading"}>Service Detection</h3>
                            <div className={"workspace-data-details-list"}>
                                <div className="workspace-data-details-list-elements">
                                    <span>Started by:</span>
                                    <span>Created:</span>
                                    <span>Finished:</span>
                                </div>
                                <div className="workspace-data-details-list-elements">
                                    <span>{attack.startedBy.displayName}</span>
                                    <span>{formatDate(attack.createdAt)}</span>
                                    <span>{formatDate(attack.finishedAt)}</span>
                                </div>
                            </div>
                        </div>
                        <div className={"workspace-data-details-pane"}>
                            <h3 className={"sub-heading"}>Result</h3>
                            <div className="workspace-data-details-list-elements">
                                {sdResult.serviceNames.map((s) => {
                                    return <span>{s}</span>;
                                })}
                            </div>
                        </div>
                        <div className={"workspace-data-details-pane-layout"}>
                            <div className={"workspace-data-details-pane"}>
                                <h3 className={"sub-heading"}>Date</h3>
                                <span>{formatDate(sdResult.createdAt)}</span>
                            </div>
                            <div className={"workspace-data-details-pane"}>
                                <h3 className={"sub-heading"}>Certainty</h3>
                                <span>{sdResult.certainty}</span>
                            </div>
                        </div>
                        <div className={"workspace-data-details-pane-layout"}>
                            <div className={"workspace-data-details-pane"}>
                                <h3 className={"sub-heading"}>IP</h3>
                                <span>{sdResult.host}</span>
                            </div>
                            <div className={"workspace-data-details-pane"}>
                                <h3 className={"sub-heading"}>Port</h3>
                                <span>{sdResult.port}</span>
                            </div>
                        </div>
                    </div>
                );
            case "QueryDehashed":
                const qdResult = attack.results.at(clampedResultPage);
                if (qdResult === undefined) return null;
                return (
                    <div className="workspace-data-details-container">
                        <div className={"workspace-data-details-pane"}>
                            <h3 className={"sub-heading"}>Query Dehashed</h3>
                            <div className={"workspace-data-details-list"}>
                                <div className="workspace-data-details-list-elements">
                                    <span>Started by:</span>
                                    <span>Created:</span>
                                    <span>Finished:</span>
                                </div>
                                <div className="workspace-data-details-list-elements">
                                    <span>{attack.startedBy.displayName}</span>
                                    <span>{formatDate(attack.createdAt)}</span>
                                    <span>{formatDate(attack.finishedAt)}</span>
                                </div>
                            </div>
                        </div>
                        <div className={"workspace-data-details-pane-layout"}>
                            <div className={"workspace-data-details-pane"}>
                                <h3 className={"sub-heading"}>Date</h3>
                                <span>{formatDate(qdResult.createdAt)}</span>
                            </div>
                            <div className={"workspace-data-details-pane"}>
                                <h3 className={"sub-heading"}>ID</h3>
                                <span>{qdResult.dehashedId}</span>
                            </div>
                        </div>
                        <div className={"workspace-data-details-pane-layout"}>
                            <div className={"workspace-data-details-pane"}>
                                <h3 className={"sub-heading"}>E-mail</h3>
                                {qdResult.email !== null && qdResult.email !== undefined ? (
                                    <span>{qdResult.email}</span>
                                ) : (
                                    <span>/</span>
                                )}
                            </div>
                            <div className={"workspace-data-details-pane"}>
                                <h3 className={"sub-heading"}>Username</h3>
                                {qdResult.username !== null && qdResult.username !== undefined ? (
                                    <span>{qdResult.username}</span>
                                ) : (
                                    <span>/</span>
                                )}
                            </div>
                        </div>
                        <div className={"workspace-data-details-pane-layout"}>
                            <div className={"workspace-data-details-pane"}>
                                <h3 className={"sub-heading"}>Name</h3>
                                {qdResult.name !== null && qdResult.name !== undefined ? (
                                    <span>{qdResult.name}</span>
                                ) : (
                                    <span>/</span>
                                )}
                            </div>
                            <div className={"workspace-data-details-pane"}>
                                <h3 className={"sub-heading"}>Address</h3>
                                {qdResult.address !== null && qdResult.address !== undefined ? (
                                    <span>{qdResult.address}</span>
                                ) : (
                                    <span>/</span>
                                )}
                            </div>
                        </div>
                        <div className={"workspace-data-details-pane-layout"}>
                            <div className={"workspace-data-details-pane"}>
                                <h3 className={"sub-heading"}>Password</h3>
                                {qdResult.password !== null && qdResult.password !== undefined ? (
                                    <span>{qdResult.password}</span>
                                ) : (
                                    <span>/</span>
                                )}
                            </div>
                            <div className={"workspace-data-details-pane"}>
                                <h3 className={"sub-heading"}>Hashed pw</h3>
                                {qdResult.hashedPassword !== null && qdResult.hashedPassword !== undefined ? (
                                    <span>{qdResult.hashedPassword}</span>
                                ) : (
                                    <span>/</span>
                                )}
                            </div>
                        </div>
                        <div className={"workspace-data-details-pane-layout"}>
                            <div className={"workspace-data-details-pane"}>
                                <h3 className={"sub-heading"}>IP</h3>
                                <span>{qdResult.ipAddress}</span>
                            </div>
                            <div className={"workspace-data-details-pane"}>
                                <h3 className={"sub-heading"}>Phone</h3>
                                {qdResult.phone !== null && qdResult.phone !== undefined ? (
                                    <span>{qdResult.phone}</span>
                                ) : (
                                    <span>/</span>
                                )}
                            </div>
                        </div>
                        <div className={"workspace-data-details-pane-layout"}>
                            <div className={"workspace-data-details-pane"}>
                                <h3 className={"sub-heading"}>Database</h3>
                                {qdResult.databaseName !== null && qdResult.databaseName !== undefined ? (
                                    <span>{qdResult.databaseName}</span>
                                ) : (
                                    <span>/</span>
                                )}
                            </div>
                            <div className={"workspace-data-details-pane"}>
                                <h3 className={"sub-heading"}>Vin</h3>
                                {qdResult.vin !== null && qdResult.vin !== undefined ? (
                                    <span>{qdResult.vin}</span>
                                ) : (
                                    <span>/</span>
                                )}
                            </div>
                        </div>
                    </div>
                );
            case "OsDetection":
                const osResult = attack.results.at(clampedResultPage);
                if (osResult === undefined) return null;
                return (
                    <div className="workspace-data-details-container">
                        <div className={"workspace-data-details-pane"}>
                            <h3 className={"sub-heading"}>OS Detection</h3>
                            <div className={"workspace-data-details-list"}>
                                <div className="workspace-data-details-list-elements">
                                    <span>Started by:</span>
                                    <span>Created:</span>
                                    <span>Finished:</span>
                                </div>
                                <div className="workspace-data-details-list-elements">
                                    <span>{attack.startedBy.displayName}</span>
                                    <span>{formatDate(attack.createdAt)}</span>
                                    <span>{formatDate(attack.finishedAt)}</span>
                                </div>
                            </div>
                        </div>
                        <div className={"workspace-data-details-pane-layout"}>
                            <div className={"workspace-data-details-pane wide"}>
                                <h3 className={"sub-heading"}>Date</h3>
                                <span>{formatDate(osResult.createdAt)}</span>
                            </div>
                            <div className={"workspace-data-details-pane"}>
                                <h3 className={"sub-heading"}>OS</h3>
                                <div style={{ display: "flex", alignItems: "center" }}>
                                    <OsIcon os={osResult.os} size="24px" style={{ marginRight: "4px" }} />
                                    {osResult.os}
                                </div>
                            </div>
                            <div className={"workspace-data-details-pane"}>
                                <h3 className={"sub-heading"}>Version</h3>
                                <pre>{osResult.version || "unknown"}</pre>
                            </div>
                            <div className={"workspace-data-details-pane wide"}>
                                <h3 className={"sub-heading"}>Hints</h3>
                                <pre>{osResult.hints || "n/a"}</pre>
                            </div>
                        </div>
                    </div>
                );
            case undefined:
                return "undefined";
            default:
                return "Unimplemented";
        }
    })();

    return (
        <>
            <div className="workspace-data-details-result-container">
                <div className="workspace-data-details-result-control">
                    <button
                        className={"workspace-table-button"}
                        disabled={clampedResultPage === 0}
                        onClick={() => setResultPage(clampedResultPage - 1)}
                    >
                        <ArrowLeftIcon />
                    </button>
                    <span>
                        {clampedResultPage + 1} of {attack.results.length}
                    </span>
                    <button
                        className={"workspace-table-button"}
                        disabled={clampedResultPage === attack.results.length - 1}
                        onClick={() => setResultPage(clampedResultPage + 1)}
                    >
                        <ArrowRightIcon />
                    </button>
                </div>
                {attackElement}

                <div className="workspace-data-details-uuid">
                    {resultUuid}
                    <button className="icon-button" onClick={() => copyToClipboard(resultUuid)}>
                        <CopyIcon />
                    </button>
                </div>
            </div>
            <div className="workspace-data-details-table-controls">
                <div className="workspace-data-details-controls-container">
                    <button
                        className={"workspace-table-button"}
                        disabled={clampedAttackPage === 0}
                        onClick={() => {
                            setResultPage(0);
                            setAttackPage(clampedAttackPage - 1);
                        }}
                    >
                        <ArrowLeftIcon />
                    </button>
                    <div className="workspace-table-controls-page-container">
                        <span>
                            {clampedAttackPage + 1} of {attacks.length}
                        </span>
                    </div>
                    <button
                        className={"workspace-table-button"}
                        disabled={clampedAttackPage === attacks.length - 1}
                        onClick={() => {
                            setResultPage(0);
                            setAttackPage(clampedAttackPage + 1);
                        }}
                    >
                        <ArrowRightIcon />
                    </button>
                </div>
            </div>
        </>
    );
}

function formatDate(date: Date | null | undefined) {
    if (date !== null && date !== undefined) {
        return (
            date.getDate() +
            "/" +
            (date.getMonth() + 1) +
            "/" +
            date.getUTCFullYear() +
            " " +
            date.getHours() +
            ":" +
            date.getMinutes() +
            ":" +
            date.getSeconds()
        );
    }
}
