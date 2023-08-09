import React from "react";
import { Api, UUID } from "../api/api";
import { handleApiError } from "../utils/helper";
import { SimpleAttack, SimpleTcpPortScanResult } from "../api/generated";
import Loading from "../components/loading";

type AttackResultsProps = { attackUuid: UUID };

export default function AttackResults(props: AttackResultsProps) {
    const { attackUuid } = props;
    const [attack, setAttack] = React.useState<SimpleAttack | null>(null);
    React.useEffect(() => {
        Api.attacks.get(attackUuid).then(handleApiError(setAttack));
    }, [setAttack, attackUuid]);

    if (attack === null) return <Loading />;
    else
        return (
            <div className="pane">
                <h1 className="heading neon">{attack.attackType}</h1>
                <h2 className="neon">By {attack.startedFrom.displayName}</h2>
                <p>Started {attack.createdAt.toLocaleString()}</p>
                <p>
                    {attack.finishedAt === null || attack.finishedAt === undefined
                        ? "Still ongoing"
                        : `Finished at ${attack.finishedAt.toLocaleString()}`}
                </p>
                {(() => {
                    switch (attack.attackType) {
                        case "TcpPortScan":
                            return <TcpPortScanResults attackUuid={attackUuid} />;
                        default:
                            return null;
                    }
                })()}
            </div>
        );
}

const PAGE_SIZE = 50;
type TcpPortScanResultsProps = { attackUuid: string };
function TcpPortScanResults(props: TcpPortScanResultsProps) {
    const { attackUuid } = props;
    const [page, setPage] = React.useState(0);
    const [results, setResults] = React.useState<Array<SimpleTcpPortScanResult> | null>(null);
    React.useEffect(() => {
        Api.attacks.getTcpPortScanResults(attackUuid, page * PAGE_SIZE, PAGE_SIZE).then(
            handleApiError((results) => {
                setResults(results.items);
            })
        );
    }, [setResults, attackUuid, page]);
    if (results === null) return <Loading />;
    else
        return (
            <table>
                <thead>
                    <tr>
                        <th className="neon">Address</th>
                        <th className="neon">Port</th>
                        <th className="neon">Created at</th>
                    </tr>
                </thead>
                <tbody>
                    {results.map((result) => (
                        <tr>
                            <td>{result.address}</td>
                            <td>{result.port}</td>
                            <td>{result.createdAt.toLocaleString()}</td>
                        </tr>
                    ))}
                </tbody>
            </table>
        );
}
