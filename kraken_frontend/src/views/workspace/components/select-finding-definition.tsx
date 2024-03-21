import React from "react";
import Select, { components } from "react-select";
import { Api } from "../../../api/api";
import { SimpleFindingDefinition } from "../../../api/generated";
import { selectStyles } from "../../../components/select-menu";
import { handleApiError } from "../../../utils/helper";

export type SelectFindingDefinitionProps = {
    selected: string | undefined;
    onSelect: (newSelected: SimpleFindingDefinition) => void;
    hovered: string | undefined;
    onHover: (newHovered: SimpleFindingDefinition | undefined) => void;
    required?: boolean;
};

export default function SelectFindingDefinition(props: SelectFindingDefinitionProps) {
    const { selected, onSelect, hovered, onHover, required } = props;

    const [definitions, setDefinitions] = React.useState([] as Array<SimpleFindingDefinition>); // all definitions
    const { current: lookups } = React.useRef<Record<"byName" | "byUuid", Record<string, SimpleFindingDefinition>>>({
        byName: {},
        byUuid: {},
    });
    React.useEffect(() => {
        Api.knowledgeBase.findingDefinitions.all().then(
            handleApiError(({ findingDefinitions }) => {
                findingDefinitions.sort(
                    (a, b) => a.name.localeCompare(b.name) || (a.cve && b.cve ? a.cve.localeCompare(b.cve) : 0),
                );
                setDefinitions(findingDefinitions);
                lookups.byName = Object.fromEntries(findingDefinitions.map((x) => [x.name, x]));
                lookups.byUuid = Object.fromEntries(findingDefinitions.map((x) => [x.uuid, x]));
            }),
        );
    }, []);

    const label = ({ name, cve }: { name: string; cve?: string | null }) => name + (cve ? ` [${cve}]` : "");

    return (
        <Select<{ label: string; name: string; cve?: string | null; value: string }>
            className={"dropdown"}
            styles={selectStyles("default")}
            options={definitions.map(({ uuid, name, cve }) => ({
                label: label({ name, cve }),
                name,
                cve,
                value: uuid,
            }))}
            value={
                typeof selected === "string"
                    ? {
                          label: lookups.byUuid[selected] ? label(lookups.byUuid[selected]) : "",
                          name: lookups.byUuid[selected].name ?? "",
                          cve: lookups.byUuid[selected].cve,
                          value: selected,
                      }
                    : null
            }
            onChange={(value) => {
                const definition = value && lookups.byUuid[value.value];
                if (definition) onSelect(definition);
            }}
            isClearable={false}
            autoFocus={false}
            required={required}
            components={{
                Option: ({ children, ...props }) => (
                    <div
                        onMouseOver={() => onHover(lookups.byUuid[props.data.value])}
                        onMouseOut={() => onHover(undefined)}
                    >
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
                    </div>
                ),
            }}
        />
    );
}
