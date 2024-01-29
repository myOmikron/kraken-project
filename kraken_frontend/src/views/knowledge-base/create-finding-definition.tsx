import React from "react";
import Input from "../../components/input";
import { SelectPrimitive } from "../../components/select-menu";
import Editor from "@monaco-editor/react";
import { setupMonaco } from "../knowledge-base";
import { GithubMarkdown } from "../../components/github-markdown";

export type CreateFindingDefinitionProps = {};
export function CreateFindingDefinition(props: CreateFindingDefinitionProps) {
    const [name, setName] = React.useState("");
    const [severity, setSeverity] = React.useState("Medium");
    const [cve, setCve] = React.useState("");

    const [summary, setSummary] = React.useState("");
    const [description, setDescription] = React.useState("");
    const [impact, setImpact] = React.useState("");
    const [remediation, setRemediation] = React.useState("");
    const [references, setReferences] = React.useState("");

    // Lookup used to by the markdown editor to switch between the different markdown sections
    const sections = {
        summary: {
            name: "Summary",
            value: summary,
            set: setSummary,
            language: "text",
        },
        description: {
            name: "Description",
            value: description,
            set: setDescription,
            language: "markdown",
        },
        impact: {
            name: "Impact",
            value: impact,
            set: setImpact,
            language: "markdown",
        },
        remediation: {
            name: "Remediation",
            value: remediation,
            set: setRemediation,
            language: "markdown",
        },
        references: {
            name: "Name",
            value: references,
            set: setReferences,
            language: "markdown",
        },
    };
    const [editor, setEditor] = React.useState<keyof typeof sections>("summary");

    return (
        <div className={"create-finding-definition"}>
            <div className={"pane"}>
                <h1 className={"heading"}>New Finding Definition</h1>

                <div className={"create-finding-definition-header"}>
                    <h2 className={"sub-heading"}>Name</h2>
                    <h2 className={"sub-heading"}>Severity</h2>
                    <h2 className={"sub-heading"}>CVE</h2>
                    <Input maxLength={255} value={name} onChange={setName} />
                    <SelectPrimitive
                        value={severity}
                        options={["Okay", "Low", "Medium", "High", "Critical"]}
                        onChange={(value) => setSeverity(value || severity)}
                    />
                    <Input maxLength={255} value={cve} onChange={setCve} />
                </div>

                <h2 className={"sub-heading"}>Summary</h2>
                <div
                    className={`nested-pane create-finding-definition-section ${
                        editor === "summary" ? "selected" : ""
                    }`}
                    onClick={() => setEditor("summary")}
                >
                    {summary.length === 0 ? null : summary.split("\n\n").map((line) => <p>{line}</p>)}
                </div>

                <h2 className={"sub-heading"}>Description</h2>
                <div
                    className={`nested-pane create-finding-definition-section ${
                        editor === "description" ? "selected" : ""
                    }`}
                    onClick={() => setEditor("description")}
                >
                    <GithubMarkdown>{description}</GithubMarkdown>
                </div>

                <h2 className={"sub-heading"}>Impact</h2>
                <div
                    className={`nested-pane create-finding-definition-section ${editor === "impact" ? "selected" : ""}`}
                    onClick={() => setEditor("impact")}
                >
                    <GithubMarkdown>{impact}</GithubMarkdown>
                </div>

                <h2 className={"sub-heading"}>Remediation</h2>
                <div
                    className={`nested-pane create-finding-definition-section ${
                        editor === "remediation" ? "selected" : ""
                    }`}
                    onClick={() => setEditor("remediation")}
                >
                    <GithubMarkdown>{remediation}</GithubMarkdown>
                </div>

                <h2 className={"sub-heading"}>References</h2>
                <div
                    className={`nested-pane create-finding-definition-section ${
                        editor === "references" ? "selected" : ""
                    }`}
                    onClick={() => setEditor("references")}
                >
                    <GithubMarkdown>{references}</GithubMarkdown>
                </div>
            </div>
            {
                <div className={"pane"}>
                    <h2 className={"sub-heading"}>Editing {sections[editor].name}</h2>
                    <Editor
                        className={"knowledge-base-editor"}
                        theme={"custom"}
                        beforeMount={setupMonaco}
                        language={sections[editor].language}
                        value={sections[editor].value}
                        onChange={(value) => {
                            sections[editor].set(value || "");
                        }}
                    />
                </div>
            }
        </div>
    );
}
