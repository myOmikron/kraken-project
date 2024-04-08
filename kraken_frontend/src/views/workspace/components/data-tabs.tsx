import React from "react";
import { AggregationType } from "../../../api/generated";
import { ObjectFns, useStableObj } from "../../../utils/helper";

const KEY = "dataTab";
const TABS: Record<AggregationType, string> = {
    [AggregationType.Domain]: "Domains",
    [AggregationType.Host]: "Hosts",
    [AggregationType.Port]: "Ports",
    [AggregationType.Service]: "Services",
    [AggregationType.HttpService]: "HTTP Services",
} as const;

export type DataTabsSelectorProps = {
    value: AggregationType;
    onChange: (newValue: AggregationType) => void;
};

export function useDataTabs(): [AggregationType, React.Dispatch<React.SetStateAction<AggregationType>>] {
    const [value, setValue] = React.useState<AggregationType>(() => {
        const stored = window.localStorage.getItem(KEY);
        if (stored && stored in TABS) {
            return stored as AggregationType;
        } else {
            return AggregationType.Domain;
        }
    });
    const stableValue = useStableObj({ value });
    React.useEffect(
        () => () => {
            window.localStorage.setItem(KEY, stableValue.value);
        },
        [],
    );
    return [value, setValue];
}

export function DataTabsSelector(props: DataTabsSelectorProps) {
    const { value, onChange } = props;

    return (
        <div className="tabs-selector-container">
            {ObjectFns.entries(TABS).map(([key, displayName]) => (
                <div className={"tabs " + (value !== key ? "" : "selected-tab")} onClick={() => onChange(key)}>
                    <h3 className={"heading"}>{displayName}</h3>
                </div>
            ))}
        </div>
    );
}
