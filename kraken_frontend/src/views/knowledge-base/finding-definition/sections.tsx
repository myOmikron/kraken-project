import React from "react";
import InformationIcon from "../../../svg/information";
import BookIcon from "../../../svg/book";
import FlameIcon from "../../../svg/flame";
import BandageIcon from "../../../svg/bandage";
import LibraryIcon from "../../../svg/library";
import PersonCircleIcon from "../../../svg/person-circle";

// TODO: replace with generated schema
export enum FindingSection {
    Summary = "Summary",
    Description = "Description",
    Impact = "Impact",
    Remediation = "Remediation",
    References = "References",
}

/** State object provided by {@link useSectionsState `useSectionsState`} */
export type Sections = Record<
    FindingSection,
    {
        /** This section's content */
        value: string;
        /** Setter for this section's content */
        set: React.Dispatch<React.SetStateAction<string>>;

        /** The language this section should be edited as */
        language: "markdown" | "text";

        /** Selects this section */
        select(): void;
        /** Is this section selected? */
        selected: boolean;
    }
> & {
    /** The currently selected section */
    selected: FindingSection;
};

/**
 * {@link React.useState `React.useState`} specialized for storing a finding definition's sections
 *
 * Besides just storing the sections' raw data, it also stores a `selected` state.
 * It also attaches the static information of what language the editor should use for each section.
 */
export function useSectionsState(): Sections {
    const [summary, setSummary] = React.useState("");
    const [description, setDescription] = React.useState("");
    const [impact, setImpact] = React.useState("");
    const [remediation, setRemediation] = React.useState("");
    const [references, setReferences] = React.useState("");

    const [selectedSection, setSelectedSection] = React.useState<FindingSection>(FindingSection.Summary);

    return {
        Summary: {
            value: summary,
            set: setSummary,
            language: "text",
            select: () => setSelectedSection(FindingSection.Summary),
            selected: selectedSection === FindingSection.Summary,
        },
        Description: {
            value: description,
            set: setDescription,
            language: "markdown",
            select: () => setSelectedSection(FindingSection.Description),
            selected: selectedSection === FindingSection.Description,
        },
        Impact: {
            value: impact,
            set: setImpact,
            language: "markdown",
            select: () => setSelectedSection(FindingSection.Impact),
            selected: selectedSection === FindingSection.Impact,
        },
        Remediation: {
            value: remediation,
            set: setRemediation,
            language: "markdown",
            select: () => setSelectedSection(FindingSection.Remediation),
            selected: selectedSection === FindingSection.Remediation,
        },
        References: {
            value: references,
            set: setReferences,
            language: "markdown",
            select: () => setSelectedSection(FindingSection.References),
            selected: selectedSection === FindingSection.References,
        },
        selected: selectedSection,
    };
}

/** Properties for {@link SectionSelectionTabs `<SectionSelectionTabs />`} */
export type SectionSelectionTabsProps = {
    /** The sections' selection state and their setters */
    sections: Record<FindingSection, { selected: boolean; select(): void }>;

    /** Optional booleans indicating whether another user is currently in a section */
    others?: Record<FindingSection, boolean>;
};

/** Little tab bar next to an `<Editor />` to switch between sections */
export function SectionSelectionTabs(props: SectionSelectionTabsProps) {
    const { sections, others } = props;
    return (
        <div className={"knowledge-base-editor-tabs"}>
            <button
                title={"Summary"}
                className={`knowledge-base-editor-tab ${sections.Summary.selected ? "selected" : ""}`}
                onClick={sections.Summary.select}
            >
                <InformationIcon />
                {others && others.Summary ? <PersonCircleIcon /> : null}
            </button>
            <button
                title={"Description"}
                className={`knowledge-base-editor-tab ${sections.Description.selected ? "selected" : ""}`}
                onClick={sections.Description.select}
            >
                <BookIcon />
                {others && others.Description ? <PersonCircleIcon /> : null}
            </button>
            <button
                title={"Impact"}
                className={`knowledge-base-editor-tab ${sections.Impact.selected ? "selected" : ""}`}
                onClick={sections.Impact.select}
            >
                <FlameIcon />
                {others && others.Impact ? <PersonCircleIcon /> : null}
            </button>
            <button
                title={"Remediation"}
                className={`knowledge-base-editor-tab ${sections.Remediation.selected ? "selected" : ""}`}
                onClick={sections.Remediation.select}
            >
                <BandageIcon />
                {others && others.Remediation ? <PersonCircleIcon /> : null}
            </button>
            <button
                title={"References"}
                className={`knowledge-base-editor-tab ${sections.References.selected ? "selected" : ""}`}
                onClick={sections.References.select}
            >
                <LibraryIcon />
                {others && others.References ? <PersonCircleIcon /> : null}
            </button>
        </div>
    );
}
