import React from "react";
import Creatable from "react-select/creatable";
import { toast } from "react-toastify";
import { Api } from "../../../api/api";
import { FullDomain, FullHost, FullPort, FullService } from "../../../api/generated";
import { FullHttpService } from "../../../api/generated/models/FullHttpService";
import { selectStyles } from "../../../components/select-menu";

export type EditableDataListProps<T extends FullHost | FullPort | FullDomain | FullService | FullHttpService> = {
    /**
     * The workspace containing the item whose items to list
     */
    workspace: string;

    /**
     * Which data to query
     */
    type: T extends FullDomain
        ? "domains"
        : T extends FullHost
          ? "hosts"
          : T extends FullPort
            ? "ports"
            : T extends FullService
              ? "services"
              : T extends FullHttpService
                ? "httpServices"
                : never;

    /** List of currently set items */
    items: Array<T>;

    /** Callback when the list changed */
    onChange: (items: Array<T>) => void;

    /** called when all items are loaded */
    onItemsLoaded?: (items: Array<T>) => void;
};

/** A multi `<Select />` for editing a list of items */
export default function EditableDataList<T extends FullHost | FullPort | FullDomain | FullService | FullHttpService>(
    props: EditableDataListProps<T>,
) {
    const { workspace, items, onChange } = props;

    const label = (item: T) => {
        switch (props.type) {
            case "domains":
                return (item as FullDomain).domain;
            case "services":
                return (item as FullService).name;
            case "httpServices":
                return (item as FullHttpService).name;
            case "ports":
                return (item as FullPort).port.toString();
            case "hosts":
                return (item as FullHost).ipAddr;
            default:
                throw new Error("unexpected type");
        }
    };

    // Load items from backend
    const [allData, setAllData] = React.useState<T[]>([]);
    React.useEffect(() => {
        setAllData([]);
        // TODO: partial input search through items, for now we only list the first 1000 items
        Api.workspaces[props.type]
            .all(workspace, 1000, 0)
            .then((v) => {
                const rawItems = v.unwrap().items as T[];
                // deduplicate by label (since filters use label data only)
                const items = Object.values(Object.fromEntries(rawItems.map<[string, T]>((i) => [label(i), i])));
                setAllData(items);
                props.onItemsLoaded?.(items);
            })
            .catch((e) => {
                toast(e.message);
            });
    }, [workspace]);

    return (
        <Creatable<T, true>
            styles={selectStyles("default")}
            isMulti={true}
            value={items}
            onChange={(items) => onChange([...items])}
            options={allData}
            formatOptionLabel={(tag: T) => ("value" in tag ? <>Unknown item {tag.value}</> : label(tag))}
            getOptionLabel={label}
            getOptionValue={({ uuid }) => uuid}
        />
    );
}
