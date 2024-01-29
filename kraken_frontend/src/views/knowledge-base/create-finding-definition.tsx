import React from "react";
import Input from "../../components/input";
import { SelectPrimitive } from "../../components/select-menu";
import Editor from "@monaco-editor/react";
import { setupMonaco } from "../knowledge-base";
import { GithubMarkdown } from "../../components/github-markdown";
import BandageIcon from "../../svg/bandage";
import LibraryIcon from "../../svg/library";
import FlameIcon from "../../svg/flame";
import InformationIcon from "../../svg/information";
import BookIcon from "../../svg/book";
import PersonCircleIcon from "../../svg/person-circle";

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

    // Boolean flags if others are editing
    const [summaryOthers, setSummaryOthers] = React.useState(false);
    const [descriptionOthers, setDescriptionOthers] = React.useState(false);
    const [impactOthers, setImpactOthers] = React.useState(false);
    const [remediationOthers, setRemediationOthers] = React.useState(false);
    const [referencesOthers, setReferencesOthers] = React.useState(false);

    // Lookup used to by the markdown editor to switch between the different markdown sections
    const sections = {
        summary: {
            value: summary,
            set: setSummary,
            language: "text",
        },
        description: {
            value: description,
            set: setDescription,
            language: "markdown",
        },
        impact: {
            value: impact,
            set: setImpact,
            language: "markdown",
        },
        remediation: {
            value: remediation,
            set: setRemediation,
            language: "markdown",
        },
        references: {
            value: references,
            set: setReferences,
            language: "markdown",
        },
    };
    const [editor, setEditor] = React.useState<keyof typeof sections>("summary");

    return (
        <div className={"create-finding-definition-container"}>
            <div className={"pane"}>
                <h1 className={"heading"}>New Finding Definition</h1>
            </div>
            <div className={"pane"}>
                <div className={"create-finding-definition-form"}>
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

                    <div>
                        <h2 className={"sub-heading"}>
                            <InformationIcon />
                            Summary
                        </h2>
                        <div className={`nested-pane`}>
                            {summary.length === 0 ? null : summary.split("\n\n").map((line) => <p>{line}</p>)}
                        </div>
                    </div>

                    <div>
                        <h2 className={"sub-heading"}>
                            <BookIcon />
                            Description
                        </h2>
                        <div className={`nested-pane`}>
                            <GithubMarkdown>{description}</GithubMarkdown>
                        </div>
                    </div>

                    <div>
                        <h2 className={"sub-heading"}>
                            <FlameIcon />
                            Impact
                        </h2>
                        <div className={`nested-pane`}>
                            <GithubMarkdown>{impact}</GithubMarkdown>
                        </div>
                    </div>

                    <div>
                        <h2 className={"sub-heading"}>
                            <BandageIcon />
                            Remediation
                        </h2>
                        <div className={`nested-pane`}>
                            <GithubMarkdown>{remediation}</GithubMarkdown>
                        </div>
                    </div>

                    <div>
                        <h2 className={"sub-heading"}>
                            <LibraryIcon />
                            References
                        </h2>
                        <div className={`nested-pane`}>
                            <GithubMarkdown>{references}</GithubMarkdown>
                        </div>
                    </div>

                    <button className={"button"}>Create</button>
                </div>
                <div className={"create-finding-definition-editor"}>
                    <div className={"knowledge-base-editor-tabs"}>
                        <button
                            title={"Summary"}
                            className={`knowledge-base-editor-tab ${editor === "summary" ? "selected" : ""}`}
                            onClick={() => setEditor("summary")}
                        >
                            <InformationIcon />
                            {summaryOthers && (
                                <PersonCircleIcon
                                    className={"user-indicator icon"}
                                    title={"Currently edited by someone else"}
                                />
                            )}
                        </button>
                        <button
                            title={"Description"}
                            className={`knowledge-base-editor-tab ${editor === "description" ? "selected" : ""}`}
                            onClick={() => setEditor("description")}
                        >
                            <BookIcon />
                            {descriptionOthers && (
                                <PersonCircleIcon
                                    className={"user-indicator icon"}
                                    title={"Currently edited by someone else"}
                                />
                            )}
                        </button>
                        <button
                            title={"Impact"}
                            className={`knowledge-base-editor-tab ${editor === "impact" ? "selected" : ""}`}
                            onClick={() => setEditor("impact")}
                        >
                            <FlameIcon />
                            {impactOthers && (
                                <PersonCircleIcon
                                    className={"user-indicator icon"}
                                    title={"Currently edited by someone else"}
                                />
                            )}
                        </button>
                        <button
                            title={"Remediation"}
                            className={`knowledge-base-editor-tab ${editor === "remediation" ? "selected" : ""}`}
                            onClick={() => setEditor("remediation")}
                        >
                            <BandageIcon />
                            {remediationOthers && (
                                <PersonCircleIcon
                                    className={"user-indicator icon"}
                                    title={"Currently edited by someone else"}
                                />
                            )}
                        </button>
                        <button
                            title={"References"}
                            className={`knowledge-base-editor-tab ${editor === "references" ? "selected" : ""}`}
                            onClick={() => setEditor("references")}
                        >
                            <LibraryIcon />
                            {referencesOthers && (
                                <PersonCircleIcon
                                    className={"user-indicator icon"}
                                    title={"Currently edited by someone else"}
                                />
                            )}
                        </button>
                    </div>
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
            </div>
        </div>
    );
}
