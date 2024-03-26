import React from "react";
import { components } from "react-select";
import { Api } from "../../../api/api";
import { SimpleFindingDefinition } from "../../../api/generated";
import { selectStyles } from "../../../components/select-menu";
import { handleApiError } from "../../../utils/helper";
import Creatable from "react-select/creatable";
import Popup from "reactjs-popup";
import { CreateFindingDefinition } from "../../knowledge-base/create-finding-definition";
import "../../../styling/select-finding-definition.css";

export type SelectFindingDefinitionProps = {
    selected: string | undefined;
    onSelect: (newSelected: SimpleFindingDefinition) => void;
    onHover: (newHovered: SimpleFindingDefinition | undefined) => void;
    required?: boolean;
};

export default function SelectFindingDefinition(props: SelectFindingDefinitionProps) {
    const { selected, onSelect, onHover, required } = props;

    const [newDefinition, setNewDefinition] = React.useState<string>();
    const [definitions, setDefinitions] = React.useState([] as Array<SimpleFindingDefinition>); // all definitions
    React.useEffect(() => {
        Api.knowledgeBase.findingDefinitions
            .all()
            .then(
                handleApiError(({ findingDefinitions }) =>
                    setDefinitions(
                        findingDefinitions.sort(
                            (a, b) => a.name.localeCompare(b.name) || (a.cve && b.cve ? a.cve.localeCompare(b.cve) : 0),
                        ),
                    ),
                ),
            );
    }, []);

    return (
        <>
            <Creatable<SimpleFindingDefinition>
                className={"dropdown"}
                styles={selectStyles("default")}
                isClearable={false}
                autoFocus={false}
                required={required}
                options={definitions}
                getOptionLabel={({ name, cve }) => name + (cve ? ` [${cve}]` : "")}
                getOptionValue={({ uuid }) => uuid}
                value={definitions.find(({ uuid }) => uuid === selected) ?? null}
                onChange={(value) => {
                    onHover(undefined);
                    if (value) onSelect(value);
                }}
                onCreateOption={(name) => setNewDefinition(name)}
                components={{
                    SelectContainer: (props) => {
                        props.innerProps.onMouseOut = () => onHover(undefined);
                        return components.SelectContainer(props);
                    },
                    Option: ({ children: _, ...props }) => {
                        if ("value" in props.data) {
                            return (
                                <components.Option {...props}>Create {props.data.value as string}</components.Option>
                            );
                        } else {
                            props.innerProps.onMouseOver = () => onHover(props.data);
                            props.innerProps.onMouseOut = () => onHover(undefined);
                            return (
                                <components.Option {...props}>
                                    {props.data.name}
                                    {props.data.cve && (
                                        <span
                                            style={{
                                                fontSize: "0.8em",
                                                float: "right",
                                                opacity: 0.8,
                                            }}
                                        >
                                            [{props.data.cve}]
                                        </span>
                                    )}
                                </components.Option>
                            );
                        }
                    },
                }}
            />
            <Popup modal nested open={newDefinition !== undefined} onClose={() => setNewDefinition(undefined)}>
                <div className={"select-finding-definition-popup pane"}>
                    <CreateFindingDefinition
                        initialName={newDefinition || ""}
                        onCreate={(def) => {
                            setDefinitions((defs) => [def, ...defs]);
                            setNewDefinition(undefined);
                            onSelect(def);
                        }}
                        inPane
                    />
                </div>
            </Popup>
        </>
    );
}
