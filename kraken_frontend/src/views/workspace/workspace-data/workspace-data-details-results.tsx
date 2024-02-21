import React from "react";
import { SourceAttack } from "../../../api/generated";
import "../../../styling/workspace-data-details.css";
import ArrowLeftIcon from "../../../svg/arrow-left";
import ArrowRightIcon from "../../../svg/arrow-right";
import CopyIcon from "../../../svg/copy";
import { copyToClipboard } from "../../../utils/helper";

type WorkspaceDataDetailsResultsProps = {
    attack: SourceAttack;
    uuid: string;
};
type WorkspaceDataDetailsResultsState = {
    page: number;
};

export default class WorkspaceDataDetailsResults extends React.Component<
    WorkspaceDataDetailsResultsProps,
    WorkspaceDataDetailsResultsState
> {
    constructor(props: WorkspaceDataDetailsResultsProps) {
        super(props);

        this.state = {
            page: 0,
        };
    }

    componentDidUpdate(
        prevProps: Readonly<WorkspaceDataDetailsResultsProps>,
        prevState: Readonly<WorkspaceDataDetailsResultsState>,
        snapshot?: any
    ) {
        if (prevProps.attack.uuid !== this.props.attack.uuid || prevProps.uuid !== this.props.uuid) {
            this.setState({ page: 0 });
        }
    }

    formateDate(date: Date | null | undefined) {
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

    render() {
        if (this.props.attack === undefined || this.props.attack === null) {
            return null;
        }

        const a = this.props.attack;
        const attackElement = (() => {
            switch (this.props.attack.attackType) {
                case "DnsResolution":
                    if (this.state.page < this.props.attack.results.length) {
                        let dnsResult = this.props.attack.results[this.state.page];
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
                                            <span>{a.startedBy.displayName}</span>
                                            <span>{this.formateDate(a.createdAt)}</span>
                                            <span>{this.formateDate(a.finishedAt)}</span>
                                        </div>
                                    </div>
                                </div>
                                <div className={"workspace-data-details-pane-layout"}>
                                    <div className={"workspace-data-details-pane"}>
                                        <h3 className={"sub-heading"}>{dnsResult.dnsRecordType}</h3>
                                        <span className={"workspace-data-details-text-wrap"}>
                                            {dnsResult.destination}
                                        </span>
                                    </div>
                                    <div className={"workspace-data-details-pane"}>
                                        <h3 className={"sub-heading"}>Date</h3>
                                        <span>{this.formateDate(dnsResult.createdAt)}</span>
                                    </div>
                                </div>
                            </div>
                        );
                    } else {
                        return null;
                    }
                case "DnsTxtScan":
                    if (this.state.page < this.props.attack.results.length) {
                        let txtResult = this.props.attack.results[this.state.page];
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
                                            <span>{a.startedBy.displayName}</span>
                                            <span>{this.formateDate(a.createdAt)}</span>
                                            <span>{this.formateDate(a.finishedAt)}</span>
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
                                                                        {entry.spf.spfDomain &&
                                                                            entry.spf.spfIp &&
                                                                            " (domain) "}
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
                                                                        {entry.spf.spfDomain &&
                                                                            entry.spf.spfIp &&
                                                                            " (ip) "}
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
                    } else {
                        return null;
                    }
                case "QueryCertificateTransparency":
                    if (this.state.page < this.props.attack.results.length) {
                        let qctResult = this.props.attack.results[this.state.page];
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
                                            <span>{a.startedBy.displayName}</span>
                                            <span>{this.formateDate(a.createdAt)}</span>
                                            <span>{this.formateDate(a.finishedAt)}</span>
                                        </div>
                                    </div>
                                </div>
                                <div className={"workspace-data-details-pane-layout"}>
                                    <div className={"workspace-data-details-pane"}>
                                        <h3 className={"sub-heading"}>Serial Number</h3>
                                        <span className={"workspace-data-details-text-wrap"}>
                                            {qctResult.serialNumber}
                                        </span>
                                    </div>
                                    <div className={"workspace-data-details-pane"}>
                                        <h3 className={"sub-heading"}>Date</h3>
                                        <span>{this.formateDate(qctResult.createdAt)}</span>
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
                                            <span>{this.formateDate(qctResult.notBefore)}</span>
                                            <span>{this.formateDate(qctResult.notAfter)}</span>
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
                    } else {
                        return null;
                    }
                case "BruteforceSubdomains":
                    if (this.state.page < this.props.attack.results.length) {
                        let bsResult = this.props.attack.results[this.state.page];
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
                                            <span>{a.startedBy.displayName}</span>
                                            <span>{this.formateDate(a.createdAt)}</span>
                                            <span>{this.formateDate(a.finishedAt)}</span>
                                        </div>
                                    </div>
                                </div>
                                <div className={"workspace-data-details-pane"}>
                                    <h3 className={"sub-heading"}>Date</h3>
                                    <span>{this.formateDate(bsResult.createdAt)}</span>
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
                    } else {
                        return null;
                    }

                case "HostAlive":
                    if (this.state.page < this.props.attack.results.length) {
                        let haResult = this.props.attack.results[this.state.page];
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
                                                <span>{a.startedBy.displayName}</span>
                                                <span>{this.formateDate(a.createdAt)}</span>
                                                <span>{this.formateDate(a.finishedAt)}</span>
                                            </div>
                                        </div>
                                    </div>
                                    <div className={"workspace-data-details-pane-layout"}>
                                        <div className={"workspace-data-details-pane"}>
                                            <h3 className={"sub-heading"}>Date</h3>
                                            <span>{this.formateDate(haResult.createdAt)}</span>
                                        </div>
                                    </div>
                                </div>
                            )
                        );
                    } else {
                        return null;
                    }

                case "TcpPortScan":
                    if (this.state.page < this.props.attack.results.length) {
                        let tcpResult = this.props.attack.results[this.state.page];
                        return (
                            <div className="workspace-data-details-container">
                                <div className={"workspace-data-details-pane"}>
                                    <h3 className={"sub-heading"}>TCP Port Scan</h3>
                                    <div className={"workspace-data-details-list"}>
                                        <div className="workspace-data-details-list-elements">
                                            <span>Address:</span>
                                            <span>Started by:</span>
                                            <span>Created:</span>
                                            <span>Finished:</span>
                                        </div>
                                        <div className="workspace-data-details-list-elements">
                                            <span>{tcpResult.address}</span>
                                            <span>{a.startedBy.displayName}</span>
                                            <span>{this.formateDate(a.createdAt)}</span>
                                            <span>{this.formateDate(a.finishedAt)}</span>
                                        </div>
                                    </div>
                                </div>
                                <div className={"workspace-data-details-pane-layout"}>
                                    <div className={"workspace-data-details-pane"}>
                                        <h3 className={"sub-heading"}>Port</h3>
                                        <span>{tcpResult.port}</span>
                                    </div>
                                    <div className={"workspace-data-details-pane"}>
                                        <h3 className={"sub-heading"}>Date</h3>
                                        <span>{this.formateDate(a.createdAt)}</span>
                                    </div>
                                </div>
                            </div>
                        );
                    } else {
                        return null;
                    }
                case "ServiceDetection":
                case "UdpServiceDetection":
                    if (this.state.page < this.props.attack.results.length) {
                        let sdResult = this.props.attack.results[this.state.page];
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
                                            <span>{a.startedBy.displayName}</span>
                                            <span>{this.formateDate(a.createdAt)}</span>
                                            <span>{this.formateDate(a.finishedAt)}</span>
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
                                        <span>{this.formateDate(sdResult.createdAt)}</span>
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
                    } else {
                        return null;
                    }
                case "QueryDehashed":
                    if (this.state.page < this.props.attack.results.length) {
                        let qdResult = this.props.attack.results[this.state.page];
                        return (
                            <div className="workspace-data-details-container">
                                <div className={"workspace-data-details-pane"}>
                                    <h3 className={"sub-heading"}>Query Unhashed</h3>
                                    <div className={"workspace-data-details-list"}>
                                        <div className="workspace-data-details-list-elements">
                                            <span>Started by:</span>
                                            <span>Created:</span>
                                            <span>Finished:</span>
                                        </div>
                                        <div className="workspace-data-details-list-elements">
                                            <span>{a.startedBy.displayName}</span>
                                            <span>{this.formateDate(a.createdAt)}</span>
                                            <span>{this.formateDate(a.finishedAt)}</span>
                                        </div>
                                    </div>
                                </div>
                                <div className={"workspace-data-details-pane-layout"}>
                                    <div className={"workspace-data-details-pane"}>
                                        <h3 className={"sub-heading"}>Date</h3>
                                        <span>{this.formateDate(qdResult.createdAt)}</span>
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
                    } else {
                        return null;
                    }
                case undefined:
                    return "undefined";
                default:
                    return "Unimplemented";
            }
        })();
        console.log(this.state.page);
        if (this.state.page < this.props.attack.results.length) {
            let uuid = this.props.attack.results[this.state.page].uuid;
            return (
                <div className="workspace-data-details-result-container">
                    <div className="workspace-data-details-result-control">
                        <button
                            className={"workspace-table-button"}
                            disabled={this.state.page === 0}
                            onClick={() => {
                                let x = this.state.page - 1;
                                this.setState({ page: x });
                            }}
                        >
                            <ArrowLeftIcon />
                        </button>
                        <span>
                            {this.state.page + 1} of {this.props.attack.results.length}
                        </span>
                        <button
                            className={"workspace-table-button"}
                            disabled={this.state.page === this.props.attack.results.length - 1}
                            onClick={() => {
                                let x = this.state.page + 1;
                                this.setState({ page: x });
                            }}
                        >
                            <ArrowRightIcon />
                        </button>
                    </div>
                    {attackElement}

                    <div className="workspace-data-details-uuid">
                        {uuid}
                        <button
                            className="icon-button"
                            onClick={async () => {
                                {
                                    await copyToClipboard(uuid);
                                }
                            }}
                        >
                            <CopyIcon />
                        </button>
                    </div>
                </div>
            );
        } else {
            return null;
        }
    }
}
