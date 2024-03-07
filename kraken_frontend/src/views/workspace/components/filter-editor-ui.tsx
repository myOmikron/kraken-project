import { FC, useState } from "react";
import { FullDomain, FullHost, FullPort, FullService, OsType, PortProtocol, SimpleTag } from "../../../api/generated";
import Input from "../../../components/input";
import "../../../styling/filter-editor-ui.css";
import CollapseIcon from "../../../svg/collapse";
import ExpandIcon from "../../../svg/expand";
import { ASTField, ASTFieldTypes, ASTFields, UsedASTTypes } from "../../../utils/filter/ast";
import { SpanlessToken, tokenize, tokensToString, valueToString } from "../../../utils/filter/lexer";
import { getExprs, replaceRaw } from "../../../utils/filter/mutate";
import { parseUserPort } from "../../../utils/ports";
import EditableDataList, { EditableDataListProps } from "./editable-data-list";
import EditableTags from "./editable-tags";

export type FilterEditorProps = {
    workspace: string;
    ast: keyof typeof ASTFields;
    filter: string;
    onChange: (newValue: string) => void;
    onApply: (newApplied: string) => void;
};

export function FilterEditorUi(props: FilterEditorProps) {
    const filterComponents: { [K in UsedASTTypes]?: FC<FilterComponentProps> } = {
        tags: FilterTagsSelector,
        domain: FilterDomainSelector,
        host: FilterHostSelector,
        service: FilterServiceSelector,
        "mayberange.date": FilterDateSelector,
        "mayberange.port": FilterRawPortSelector,
        ostype: FilterOsTypeSelector,
        protocol: FilterPortProtocolSelector,
    };

    const [showAdvanced, setShowAdvanced] = useState(false);
    const [resetCount, setResetCount] = useState(0);

    const anyAdvanced = Object.entries(ASTFieldTypes[props.ast]).some(([key, type]) => {
        const ast = ASTFields[props.ast] as ASTField;
        const astField = ast[key];
        return astField.advanced;
    });

    return (
        <form
            className="filter-editor"
            onReset={() => {
                setResetCount((c) => c + 1);
                props.onChange("");
                props.onApply("");
            }}
            onSubmit={() => {}}
        >
            <main>
                {Object.entries(ASTFieldTypes[props.ast]).map(([key, type]) => {
                    const Component = filterComponents[type];
                    if (!Component) return undefined;
                    const ast = ASTFields[props.ast] as ASTField;
                    const astField = ast[key];
                    if (astField.advanced) return;
                    return <Component key={key} field={key} {...props} />;
                })}
            </main>
            {anyAdvanced && (
                <fieldset key={resetCount} className={`${showAdvanced ? "" : "collapsed"}`}>
                    <legend
                        onMouseDown={(e) => {
                            setShowAdvanced(!showAdvanced);
                            e.preventDefault();
                        }}
                    >
                        {showAdvanced ? <CollapseIcon /> : <ExpandIcon />} Advanced
                    </legend>
                    {showAdvanced &&
                        Object.entries(ASTFieldTypes[props.ast]).map(([key, type]) => {
                            const Component = filterComponents[type];
                            if (!Component) return undefined;
                            const ast = ASTFields[props.ast] as ASTField;
                            const astField = ast[key];
                            if (!astField.advanced) return;
                            return <Component key={key} field={key} {...props} />;
                        })}
                </fieldset>
            )}
            <footer>
                <Input
                    as="textarea"
                    rows={2}
                    className={"input"}
                    placeholder={"Filter..."}
                    value={props.filter}
                    onChange={(v) => props.onChange(v.replace("\n", " "))}
                />
                <button className="button" type="reset">
                    Clear
                </button>
                <button className="button" type="submit">
                    Apply
                </button>
            </footer>
        </form>
    );
}

export type FilterComponentProps = {
    workspace: string;
    ast: keyof typeof ASTFields;
    field: string; // keyof typeof ASTFields[ast]
    filter: string;
    onChange: (newValue: string) => void;
};

export function FilterTagsSelector(props: FilterComponentProps) {
    const ast = ASTFields[props.ast] as ASTField;
    const astField = ast[props.field];
    if (!astField) return undefined;

    const [allTags, setAllTags] = useState<SimpleTag[]>([]);

    function findTag(label: string): SimpleTag | undefined {
        return allTags.find((v) => v.name == label);
    }

    let values = astField.columns
        .map<[string, SpanlessToken[]]>((c) => [c, getExprs(props.filter, c)!])
        .find((c) => c[1] !== undefined);
    const usedColumn = values?.[0] ?? astField.columns[0];
    const tags =
        (values?.[1]
            .map((v) => (v.type == "value" ? findTag(v.value) : undefined))
            .filter((v) => v !== undefined) as SimpleTag[]) ?? [];

    return (
        <label>
            <span>{astField.label}</span>
            <EditableTags
                workspace={props.workspace}
                onTagsLoaded={setAllTags}
                tags={tags}
                allowCreate={false}
                onChange={(newTags: SimpleTag[]) => {
                    props.onChange(
                        replaceRaw(props.filter, usedColumn, newTags.map((t) => valueToString(t.name)).join(" & ")),
                    );
                }}
            />
        </label>
    );
}

export function FilterDomainSelector(props: FilterComponentProps) {
    return FilterDataSelector<FullDomain>({
        ...props,
        type: "domains",
        mapper: (v) => v.domain,
    });
}

export function FilterHostSelector(props: FilterComponentProps) {
    return FilterDataSelector<FullHost>({
        ...props,
        type: "hosts",
        mapper: (v) => v.ipAddr,
    });
}

export function FilterServiceSelector(props: FilterComponentProps) {
    return FilterDataSelector<FullService>({
        ...props,
        type: "services",
        mapper: (v) => v.name,
    });
}

function FilterDataSelector<T extends FullHost | FullPort | FullDomain | FullService>(
    props: FilterComponentProps & { type: EditableDataListProps<T>["type"]; mapper: (item: T) => string },
) {
    const ast = ASTFields[props.ast] as ASTField;
    const astField = ast[props.field];
    if (!astField) return undefined;

    const [allDatas, setAllDatas] = useState<T[]>([]);

    function findData(data: string): T | undefined {
        return allDatas.find((v) => props.mapper(v) == data);
    }

    let values = astField.columns
        .map<[string, SpanlessToken[]]>((c) => [c, getExprs(props.filter, c)!])
        .find((c) => c[1] !== undefined);
    const usedColumn = values?.[0] ?? astField.columns[0];
    const datas =
        (values?.[1]
            .map((v) => (v.type == "value" ? findData(v.value) : undefined))
            .filter((v) => v !== undefined) as T[]) ?? [];

    return (
        <label>
            <span>{astField.label}</span>
            <EditableDataList<T>
                workspace={props.workspace}
                onItemsLoaded={setAllDatas}
                type={props.type}
                items={datas}
                onChange={(newTags: T[]) => {
                    props.onChange(
                        replaceRaw(
                            props.filter,
                            usedColumn,
                            newTags.map((t) => valueToString(props.mapper(t))).join(", "),
                        ),
                    );
                }}
            />
        </label>
    );
}

function FilterDateSelector(props: FilterComponentProps) {
    const ast = ASTFields[props.ast] as ASTField;
    const astField = ast[props.field];
    if (!astField) return undefined;

    let values = astField.columns
        .map<[string, SpanlessToken[]]>((c) => [c, getExprs(props.filter, c)!])
        .find((c) => c[1] !== undefined);
    const usedColumn = values?.[0] ?? astField.columns[0];
    const tokens = values?.[1];
    const rangeOp = tokens?.findIndex((t) => t.type == "rangeOperator") ?? -1;
    const beforeTok = tokens?.[rangeOp - 1];
    const afterTok = tokens?.[rangeOp + 1];

    const minDateInput = beforeTok?.type == "value" ? new Date(beforeTok.value) : undefined;
    const maxDateInput = afterTok?.type == "value" ? new Date(afterTok.value) : undefined;

    let [minOverride, setMinOverride] = useState<Date | undefined>(undefined);
    let [maxOverride, setMaxOverride] = useState<Date | undefined>(undefined);

    const minDate = minOverride ?? minDateInput;
    const maxDate = maxOverride ?? maxDateInput;

    // returns a string suitable for input type="datetime-local"
    function dateToString(d: Date): string {
        const pad2 = (n: number) => (n < 10 ? "0" + n.toFixed(0) : n.toFixed(0));
        return `${d.getFullYear()}-${pad2(d.getMonth() + 1)}-${pad2(d.getDate())}T${pad2(d.getHours())}:${pad2(d.getMinutes())}`;
    }

    function setDates(min: Date | undefined, max: Date | undefined) {
        props.onChange(
            replaceRaw(
                props.filter,
                usedColumn,
                min || max
                    ? (min ? valueToString(min.toISOString()) : "") +
                          "-" +
                          (max ? valueToString(max.toISOString()) : "")
                    : "",
            ),
        );
        setMaxOverride(undefined);
        setMinOverride(undefined);
    }

    return (
        <label className="label3">
            <span>{astField.label}</span>
            <input
                type="datetime-local"
                className="input"
                value={minDate ? dateToString(minDate) : undefined}
                max={maxDate ? dateToString(maxDate) : undefined}
                onChange={(e) => {
                    if (e.target.value) {
                        let date = new Date(e.target.value);
                        if (maxDate && date > maxDate) setMinOverride(date);
                        else setDates(date, maxDate);
                    } else {
                        setDates(undefined, maxDate);
                    }
                }}
            />
            <input
                type="datetime-local"
                className="input"
                value={maxDate ? dateToString(maxDate) : undefined}
                min={minDate ? dateToString(minDate) : undefined}
                onChange={(e) => {
                    if (e.target.value) {
                        let date = new Date(e.target.value);
                        if (minDate && date < minDate) setMaxOverride(date);
                        else setDates(minDate, date);
                    } else {
                        setDates(minDate, undefined);
                    }
                }}
            />
        </label>
    );
}

function FilterRawPortSelector(props: FilterComponentProps) {
    const ast = ASTFields[props.ast] as ASTField;
    const astField = ast[props.field];
    if (!astField) return undefined;

    let values = astField.columns
        .map<[string, SpanlessToken[]]>((c) => [c, getExprs(props.filter, c)!])
        .find((c) => c[1] !== undefined);
    const usedColumn = values?.[0] ?? astField.columns[0];
    const tokens = values?.[1];
    const inputValue = tokens ? tokensToString(tokens) : undefined;

    let [override, setOverride] = useState<string | undefined>(undefined);

    const value = override ?? inputValue;

    function validatePort(v: string): void {
        if (parseUserPort(v) === false) throw new Error("invalid port: " + v);
    }

    function validatePorts(tokens: SpanlessToken[]): void {
        tokens.forEach((t) => t.type == "value" && validatePort(t.value));
    }

    return (
        <label>
            <span>{astField.label}</span>
            <input
                type="text"
                className="input"
                value={value}
                onChange={(e) => {
                    if (e.target.value) {
                        try {
                            const tokens = tokenize(e.target.value);
                            validatePorts(tokens);
                            props.onChange(replaceRaw(props.filter, usedColumn, tokensToString(tokens)));
                        } catch (err) {
                            setOverride(e.target.value);
                            e.target.setCustomValidity("Invalid input: " + err);
                            return;
                        }
                    } else {
                        props.onChange(replaceRaw(props.filter, usedColumn, ""));
                    }
                    setOverride(undefined);
                    e.target.setCustomValidity("");
                }}
            />
        </label>
    );
}

function FilterOsTypeSelector(props: FilterComponentProps) {
    return FilterCheckboxEnumSelector({
        ...props,
        enum: Object.values(OsType),
    });
}

function FilterPortProtocolSelector(props: FilterComponentProps) {
    return FilterCheckboxEnumSelector({
        ...props,
        enum: Object.values(PortProtocol),
    });
}

function FilterCheckboxEnumSelector(props: FilterComponentProps & { enum: string[] }) {
    const ast = ASTFields[props.ast] as ASTField;
    const astField = ast[props.field];
    if (!astField) return undefined;

    let values = astField.columns
        .map<[string, SpanlessToken[]]>((c) => [c, getExprs(props.filter, c)!])
        .find((c) => c[1] !== undefined);
    const usedColumn = values?.[0] ?? astField.columns[0];
    const tokens = values?.[1];

    const checked = tokens?.filter((v) => v.type == "value").map((v) => ("value" in v ? v.value : "")) ?? [];

    return (
        <div className="row checkbox-list">
            <span>{astField.label}</span>
            <div>
                {props.enum.map((enumValue) => (
                    <label key={enumValue}>
                        <input
                            type="checkbox"
                            className="input"
                            checked={checked.includes(enumValue)}
                            onChange={(e) => {
                                let newChecked = checked;
                                let existing = newChecked.indexOf(enumValue);
                                if (e.target.checked) {
                                    if (existing != -1) return;
                                    newChecked.push(enumValue);
                                } else {
                                    if (existing == -1) return;
                                    newChecked.splice(existing, 1);
                                }
                                props.onChange(
                                    replaceRaw(props.filter, usedColumn, newChecked.map(valueToString).join(", ")),
                                );
                            }}
                        />
                        {enumValue}
                    </label>
                ))}
            </div>
        </div>
    );
}
