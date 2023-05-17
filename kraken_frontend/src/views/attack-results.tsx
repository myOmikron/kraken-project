import React from "react";
import { Api } from "../api/api";
import { handleApiError } from "../utils/helper";
import { SimpleAttack, SimpleTcpPortScanResult } from "../api/generated";
import Loading from "../components/loading";

type AttackResultsProps = { attackId: number };

export default function AttackResults(props: AttackResultsProps) {
    const { attackId } = props;
    const [attack, setAttack] = React.useState<SimpleAttack | null>(null);
    React.useEffect(() => {
        Api.attacks.get(attackId).then(handleApiError(setAttack));
    }, [setAttack, attackId]);

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
                            return <TcpPortScanResults attackId={attackId} />;
                        default:
                            return null;
                    }
                })()}
            </div>
        );
}

const PAGE_SIZE = 50;
type TcpPortScanResultsProps = { attackId: number };
function TcpPortScanResults(props: TcpPortScanResultsProps) {
    const { attackId } = props;
    const [page, setPage] = React.useState(0);
    const [results, setResults] = React.useState<Array<SimpleTcpPortScanResult> | null>(null);
    React.useEffect(() => {
        Api.attacks.getTcpPortScanResults(attackId, page * PAGE_SIZE, PAGE_SIZE).then(
            handleApiError((results) => {
                setResults(results.items);
            })
        );
    }, [setResults, attackId, page]);
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
