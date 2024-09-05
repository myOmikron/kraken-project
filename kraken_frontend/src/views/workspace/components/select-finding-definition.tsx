import React from "react";
import { components } from "react-select";
import Creatable from "react-select/creatable";
import Popup from "reactjs-popup";
import { Api } from "../../../api/api";
import { SimpleFindingDefinition } from "../../../api/generated";
import { selectStyles } from "../../../components/select-menu";
import "../../../styling/select-finding-definition.css";
import { handleApiError } from "../../../utils/helper";
import { CreateFindingDefinition } from "../../knowledge-base/create-finding-definition";

export type SelectFindingDefinitionProps = {
    selected: string | null | undefined;
    onHover: (newHovered: SimpleFindingDefinition | undefined) => void;
    required?: boolean;
} & (
    | {
          isClearable: true;
          onSelect: (newSelected: SimpleFindingDefinition | null) => void;
      }
    | {
          isClearable?: false;
          onSelect: (newSelected: SimpleFindingDefinition) => void;
      }
);

export default function SelectFindingDefinition(props: SelectFindingDefinitionProps) {
    const { selected, onHover, required } = props;

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
                isClearable={props.isClearable}
                autoFocus={false}
                required={required}
                options={definitions}
                getOptionLabel={({ name, cve }) => name + (cve ? ` [${cve}]` : "")}
                getOptionValue={({ uuid }) => uuid}
                value={definitions.find(({ uuid }) => uuid === selected) ?? null}
                onChange={(value) => {
                    onHover(undefined);
                    if (props.isClearable) props.onSelect(value);
                    else if (value !== null) props.onSelect(value);
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
                            props.onSelect(def);
                        }}
                        inPane
                    />
                </div>
            </Popup>
        </>
    );
}
