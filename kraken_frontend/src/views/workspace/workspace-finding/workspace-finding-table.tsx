import React from "react";
import { Api } from "../../../api/api";
import { SimpleFinding } from "../../../api/generated";
import Input from "../../../components/input";
import { ROUTES } from "../../../routes";
import PlusIcon from "../../../svg/plus";
import { handleApiError } from "../../../utils/helper";
import CategoryList from "../components/category-list";
import SeverityIcon from "../components/severity-icon";
import { WORKSPACE_CONTEXT } from "../workspace";

export type WorkspaceFindingTableProps = {
    filter?: (finding: SimpleFinding) => boolean;
    onClickRow?: (finding: SimpleFinding, e: { ctrlKey: boolean; altKey: boolean; shiftKey: boolean }) => void;
    onAuxClickRow?: (finding: SimpleFinding, e: { ctrlKey: boolean; altKey: boolean; shiftKey: boolean }) => void;
};

export default function WorkspaceFindingTable({ onClickRow, onAuxClickRow, filter }: WorkspaceFindingTableProps) {
    const {
        workspace: { uuid: workspace },
    } = React.useContext(WORKSPACE_CONTEXT);
    const [findings, setFindings] = React.useState<Array<SimpleFinding>>([]);
    const [search, setSearch] = React.useState("");

    React.useEffect(() => {
        Api.workspaces.findings.all(workspace).then(
            handleApiError(({ findings }): void => {
                setFindings(findings);
            }),
        );
    }, [workspace]);

    return (
        <>
            <div className={"workspace-table-pre-header"}>
                <Input placeholder={"Search findings..."} value={search} onChange={setSearch} />
                <button
                    className={"button"}
                    title={"Create finding"}
                    {...ROUTES.WORKSPACE_FINDINGS_CREATE.clickHandler({ uuid: workspace })}
                >
                    <PlusIcon />
                </button>
            </div>
            <div
                className="workspace-findings-table"
                style={{ "--columns": "4em 6em 1fr 1fr 12em 0.5fr" } as Record<string, string>}
            >
                <div className={"workspace-table-header"}>
                    <span className={"workspace-data-certainty-icon"}>Severity</span>
                    <span className={"workspace-data-certainty-icon"}>Affected</span>
                    <span>Name</span>
                    <span>Categories</span>
                    <span>CVE</span>
                    <span>Created At</span>
                </div>
                <div className="workspace-table-body">
                    {findings
                        .filter((f) => {
                            const q = search.toLowerCase();
                            return f.name.toLowerCase().includes(q) || f.cve?.toLowerCase().includes(q);
                        })
                        .filter((f) => (filter ? filter(f) : true))
                        .map((f) => (
                            <div
                                key={f.uuid}
                                className="workspace-table-row"
                                onClick={(e) => onClickRow?.(f, e)}
                                onAuxClick={(e) => onAuxClickRow?.(f, e)}
                            >
                                <span className="workspace-data-certainty-icon">
                                    <SeverityIcon severity={f.severity} />
                                </span>
                                <span className="workspace-data-certainty-icon">{f.affectedCount}</span>
                                <span>{f.name}</span>
                                <CategoryList categories={f.categories} />
                                <span>{f.cve}</span>
                                <span>{f.createdAt.toLocaleString()}</span>
                            </div>
                        ))}
                </div>
            </div>
        </>
    );
}
