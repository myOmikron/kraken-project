import React, { CSSProperties } from "react";
import { Api } from "../../api/api";
import { SimpleFinding, SimpleFindingDefinition } from "../../api/generated";
import Input from "../../components/input";
import { ROUTES } from "../../routes";
import "../../styling/tabs.css";
import "../../styling/workspace-findings.css";
import GraphIcon from "../../svg/graph";
import PlusIcon from "../../svg/plus";
import SeverityIcon from "../../svg/severity";
import TableIcon from "../../svg/table";
import { handleApiError } from "../../utils/helper";
import { WORKSPACE_CONTEXT } from "./workspace";
import { TreeGraph, TreeNode } from "./workspace-finding/workspace-finding-tree";

const TABS = { table: "Table", graph: "Graph" };

type WorkspaceFindingsProps = {};

export default function WorkspaceFindings(props: WorkspaceFindingsProps) {
    const {
        workspace: { uuid: workspace },
    } = React.useContext(WORKSPACE_CONTEXT);
    const [tab, setTab] = React.useState<keyof typeof TABS>("table");
    const [findings, setFindings] = React.useState<Array<SimpleFinding>>([]);
    const [defs, setDefs] = React.useState([] as Array<SimpleFindingDefinition>);
    const [search, setSearch] = React.useState("");

    const [roots, setRoots] = React.useState<TreeNode[]>([]);

    React.useEffect(() => {
        Api.knowledgeBase.findingDefinitions
            .all()
            .then(handleApiError(({ findingDefinitions }) => setDefs(findingDefinitions)));
    }, []);
    React.useEffect(() => {
        Api.workspaces.findings.all(workspace).then(handleApiError(({ findings }) => setFindings(findings)));
    }, [workspace]);

    // @ts-ignore
    const style: CSSProperties = { "--columns": "0.1fr 1fr 1fr" };

    const body = (() => {
        switch (tab) {
            case "table":
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
                        <div className="workspace-findings-table" style={style}>
                            <div className={"workspace-table-header"}>
                                <span>Severity</span>
                                <span>Name</span>
                                <span>CVE</span>
                            </div>
                            <div className="workspace-table-body">
                                {findings
                                    .filter((f) => {
                                        if (search === "") {
                                            return true;
                                        }
                                        return f.name.toLowerCase().includes(search.toLowerCase());
                                    })
                                    .map((f) => (
                                        <div
                                            className="workspace-table-row"
                                            {...ROUTES.WORKSPACE_FINDINGS_EDIT.clickHandler({
                                                wUuid: workspace,
                                                fUuid: f.uuid,
                                            })}
                                        >
                                            <span className="workspace-data-certainty-icon">
                                                <SeverityIcon severity={f.severity} />
                                            </span>
                                            <span>{f.name}</span>
                                            <span>{f.cve}</span>
                                        </div>
                                    ))}
                            </div>
                        </div>
                    </>
                );
            case "graph":
                return <TreeGraph roots={roots} />;
            default:
                return "Unimplemented";
        }
    })();

    return (
        <div className="workspace-findings-layout">
            <div className="tabs-selector-container">
                {Object.entries(TABS).map(([key, name]) => (
                    <div
                        className={`icon-tabs ${tab !== key ? "" : "selected-icon-tab"}`}
                        onClick={() => setTab(key as keyof typeof TABS)}
                    >
                        {name === "Table" ? <TableIcon /> : <GraphIcon />}
                    </div>
                ))}
            </div>
            <div className="pane workspace-findings-body">{body}</div>
        </div>
    );
}
