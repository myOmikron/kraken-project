import React from "react";
import Select from "react-select";
import { Api } from "../../../api/api";
import { SimpleFinding, SimpleFindingCategory } from "../../../api/generated";
import FindingCategory from "../../../components/finding-category";
import FindingCategoryList from "../../../components/finding-category-list";
import Input from "../../../components/input";
import { selectStyles } from "../../../components/select-menu";
import { ROUTES } from "../../../routes";
import PlusIcon from "../../../svg/plus";
import { handleApiError } from "../../../utils/helper";
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
    // Finding categories which are used by the `findings`
    const [usedCategories, setUsedCategories] = React.useState<Array<SimpleFindingCategory>>([]);
    // Categories currently selected to be filtered by
    const [filteredCategories, setFilteredCategories] = React.useState<ReadonlyArray<SimpleFindingCategory>>([]);

    React.useEffect(() => {
        Api.workspaces.findings.all(workspace).then(
            handleApiError(({ findings }): void => {
                setFindings(findings);
                // Collect list of all USED categories without duplicates
                const categories: Record<string, SimpleFindingCategory> = {};
                for (const finding of findings) {
                    for (const category of finding.categories) {
                        categories[category.uuid] = category;
                    }
                }
                setUsedCategories(Object.values(categories));
            }),
        );
    }, [workspace]);

    return (
        <>
            <div className={"workspace-findings-table-pre-header workspace-table-pre-header"}>
                <Input placeholder={"Search findings..."} value={search} onChange={setSearch} />
                <Select<SimpleFindingCategory, true>
                    styles={selectStyles("default")}
                    placeholder={"Filter by category..."}
                    options={usedCategories}
                    isMulti
                    value={filteredCategories}
                    onChange={setFilteredCategories}
                    formatOptionLabel={(c) => <FindingCategory {...c} />}
                    getOptionLabel={({ name }) => name}
                    getOptionValue={({ uuid }) => uuid}
                />
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
                style={{ "--columns": "4em 6em 1fr 1fr 12em 0.5fr", "overflow-x": "auto" } as Record<string, string>}
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
                        .filter((finding) => {
                            const lowerCaseSearch = search.toLowerCase();
                            return (
                                (lowerCaseSearch.length > 0
                                    ? finding.name.toLowerCase().includes(lowerCaseSearch) ||
                                      finding.cve?.toLowerCase().includes(lowerCaseSearch)
                                    : true) &&
                                (filter ? filter(finding) : true) &&
                                (filteredCategories.length > 0
                                    ? finding.categories.some(({ uuid: a }) =>
                                          filteredCategories.some(({ uuid: b }) => a === b),
                                      )
                                    : true)
                            );
                        })
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
                                <FindingCategoryList categories={f.categories} />
                                <span>{f.cve}</span>
                                <span>{f.createdAt.toLocaleString()}</span>
                            </div>
                        ))}
                </div>
            </div>
        </>
    );
}
