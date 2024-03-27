import React from "react";
import { toast } from "react-toastify";
import { Api } from "../../api/api";
import { FindingSeverity, SimpleFindingDefinition } from "../../api/generated";
import { GithubMarkdown } from "../../components/github-markdown";
import Input from "../../components/input";
import ModelEditor from "../../components/model-editor";
import { SelectPrimitive } from "../../components/select-menu";
import { ROUTES } from "../../routes";
import "../../styling/create-finding-definition.css";
import BandageIcon from "../../svg/bandage";
import BookIcon from "../../svg/book";
import FlameIcon from "../../svg/flame";
import InformationIcon from "../../svg/information";
import LibraryIcon from "../../svg/library";
import { handleApiError } from "../../utils/helper";
import { SectionSelectionTabs, useSectionsState } from "./finding-definition/sections";

export type CreateFindingDefinitionProps = {
    /** Prefill the name <input /> with an initial value*/
    initialName?: string;

    /**
     * Use a custom callback upon successful creation
     *
     * The default will redirect to the list of finding definitions.
     */
    onCreate?: (definition: SimpleFindingDefinition) => void;

    /**
     * Is this component already rendered inside a pane?
     *
     * If yes, use nested-pane instead of pane again.
     */
    inPane?: boolean;
};

export function CreateFindingDefinition(props: CreateFindingDefinitionProps) {
    const [name, setName] = React.useState(props.initialName ?? "");
    const [severity, setSeverity] = React.useState<FindingSeverity>(FindingSeverity.Medium);
    const [cve, setCve] = React.useState("");

    const sections = useSectionsState();

    return (
        <div className={"create-finding-definition-container"}>
            <div className={props.inPane ? "nested-pane" : "pane"}>
                <h1 className={"heading"}>New Finding Definition</h1>
            </div>
            <div className={props.inPane ? "nested-pane" : "pane"}>
                <div className={"create-finding-definition-form"}>
                    <div className={"create-finding-definition-header"}>
                        <h2 className={"sub-heading"}>Name</h2>
                        <h2 className={"sub-heading"}>Severity</h2>
                        <h2 className={"sub-heading"}>CVE</h2>
                        <Input maxLength={255} value={name} required onChange={setName} />
                        <SelectPrimitive
                            value={severity}
                            options={Object.values(FindingSeverity)}
                            onChange={(value) => setSeverity(value || severity)}
                        />
                        <Input maxLength={255} value={cve} onChange={setCve} />
                    </div>

                    <div>
                        <h2 className={"sub-heading"}>
                            <InformationIcon />
                            Summary
                        </h2>
                        <div>
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
                        <GithubMarkdown>{sections.Description.value}</GithubMarkdown>
                    </div>

                    <div>
                        <h2 className={"sub-heading"}>
                            <FlameIcon />
                            Impact
                        </h2>
                        <GithubMarkdown>{sections.Impact.value}</GithubMarkdown>
                    </div>

                    <div>
                        <h2 className={"sub-heading"}>
                            <BandageIcon />
                            Remediation
                        </h2>
                        <GithubMarkdown>{sections.Remediation.value}</GithubMarkdown>
                    </div>

                    <div>
                        <h2 className={"sub-heading"}>
                            <LibraryIcon />
                            References
                        </h2>
                        <GithubMarkdown>{sections.References.value}</GithubMarkdown>
                    </div>

                    <button
                        className={"button"}
                        onClick={() =>
                            Api.knowledgeBase.findingDefinitions
                                .create({
                                    name,
                                    severity,
                                    cve: cve.length > 0 ? cve : null,
                                    summary: sections.Summary.value,
                                    description: sections.Description.value,
                                    impact: sections.Impact.value,
                                    remediation: sections.Remediation.value,
                                    references: sections.References.value,
                                })
                                .then(
                                    handleApiError(({ uuid }) => {
                                        toast.success("Created finding definition");
                                        if (!props.onCreate) {
                                            ROUTES.FINDING_DEFINITION_LIST.visit({});
                                        } else {
                                            props.onCreate({
                                                uuid,
                                                name,
                                                severity,
                                                cve: cve.length > 0 ? cve : null,
                                                summary: sections.Summary.value,
                                                createdAt: new Date(),
                                            });
                                        }
                                    }),
                                )
                        }
                    >
                        Create
                    </button>
                </div>
                <div className={"create-finding-definition-editor"}>
                    <SectionSelectionTabs sections={sections} />
                    <ModelEditor model={sections[sections.selected].model} />
                </div>
            </div>
        </div>
    );
}
