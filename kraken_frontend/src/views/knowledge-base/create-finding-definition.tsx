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
import { SectionSelectionTabs, useSectionsState } from "./finding-definition/sections";

export type CreateFindingDefinitionProps = {};
export function CreateFindingDefinition(props: CreateFindingDefinitionProps) {
    const [name, setName] = React.useState("");
    const [severity, setSeverity] = React.useState("Medium");
    const [cve, setCve] = React.useState("");

    const sections = useSectionsState();

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
                            {sections.Summary.value.length === 0
                                ? null
                                : sections.Summary.value.split("\n\n").map((line) => <p>{line}</p>)}
                        </div>
                    </div>

                    <div>
                        <h2 className={"sub-heading"}>
                            <BookIcon />
                            Description
                        </h2>
                        <div className={`nested-pane`}>
                            <GithubMarkdown>{sections.Description.value}</GithubMarkdown>
                        </div>
                    </div>

                    <div>
                        <h2 className={"sub-heading"}>
                            <FlameIcon />
                            Impact
                        </h2>
                        <div className={`nested-pane`}>
                            <GithubMarkdown>{sections.Impact.value}</GithubMarkdown>
                        </div>
                    </div>

                    <div>
                        <h2 className={"sub-heading"}>
                            <BandageIcon />
                            Remediation
                        </h2>
                        <div className={`nested-pane`}>
                            <GithubMarkdown>{sections.Remediation.value}</GithubMarkdown>
                        </div>
                    </div>

                    <div>
                        <h2 className={"sub-heading"}>
                            <LibraryIcon />
                            References
                        </h2>
                        <div className={`nested-pane`}>
                            <GithubMarkdown>{sections.References.value}</GithubMarkdown>
                        </div>
                    </div>

                    <button className={"button"}>Create</button>
                </div>
                <div className={"create-finding-definition-editor"}>
                    <SectionSelectionTabs sections={sections} />
                    <Editor
                        className={"knowledge-base-editor"}
                        theme={"custom"}
                        beforeMount={setupMonaco}
                        {...sections[sections.selected].editor}
                        onChange={(value, event) => {
                            if (value !== undefined) sections[sections.selected].set(value);
                        }}
                    />
                </div>
            </div>
        </div>
    );
}
