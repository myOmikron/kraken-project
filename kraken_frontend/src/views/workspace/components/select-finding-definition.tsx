import Select, { components } from "react-select";
import { selectStyles } from "../../../components/select-menu";
import React from "react";
import { Api } from "../../../api/api";
import { handleApiError } from "../../../utils/helper";
import { SimpleFindingDefinition } from "../../../api/generated";
import { Option } from "../../../utils/option";

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
                setDefinitions(findingDefinitions);
                lookups.byName = Object.fromEntries(findingDefinitions.map((x) => [x.name, x]));
                lookups.byUuid = Object.fromEntries(findingDefinitions.map((x) => [x.uuid, x]));
            }),
        );
    }, []);

    return (
        <Select<{ label: string; value: string }>
            className={"dropdown"}
            styles={selectStyles("default")}
            options={definitions.map(({ uuid, name }) => ({
                label: name,
                value: uuid,
            }))}
            value={
                typeof selected === "string"
                    ? {
                          label: lookups.byUuid[selected]?.name ?? "",
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
                        onMouseOver={() => onHover(definitions.find(({ name }) => name === props.label))}
                        onMouseOut={() => onHover(undefined)}
                    >
                        <components.Option {...props}>{children}</components.Option>
                    </div>
                ),
            }}
        />
    );
}
