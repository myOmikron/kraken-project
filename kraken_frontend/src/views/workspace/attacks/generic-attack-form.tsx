import React, { useEffect, useRef, useState } from "react";
import { toast } from "react-toastify";
import Popup from "reactjs-popup";
import { Api, handleError } from "../../../api/api";
import "../../../styling/workspace-attacks-generic.css";
import CollapseIcon from "../../../svg/collapse";
import ExpandIcon from "../../../svg/expand";
import { ObjectFns, handleApiError } from "../../../utils/helper";
import StartAttack from "../components/start-attack";
import { WORKSPACE_CONTEXT } from "../workspace";
import { AnyApiValue, AttackPrefill, IAttackDescr } from "../workspace-attacks";

type GenericAttackFormProps = {
    prefilled: AttackPrefill;
    attack: IAttackDescr;
};
type GenericAttackFormState = {
    value: { [apiJsonKey: string]: AnyApiValue };
    resetValue: { [apiJsonKey: string]: AnyApiValue };
};

export default function GenericAttackForm(props: GenericAttackFormProps) {
    const {
        workspace: { uuid: workspace },
    } = React.useContext(WORKSPACE_CONTEXT);

    const inputRefs = useRef<Array<HTMLElement | null>>([]);

    const [resetValue, setResetValue] = React.useState<{ [apiJsonKey: string]: AnyApiValue }>();
    const [value, setValue] = React.useState<{ [apiJsonKey: string]: AnyApiValue }>();

    useEffect(() => {
        const resetValue: GenericAttackFormState["resetValue"] = {};
        for (const key of Object.keys(props.attack.inputs.inputs)) {
            const input = props.attack.inputs.inputs[key];
            if ("fixed" in input) {
                resetValue[key] = input.fixed;
            } else {
                resetValue[key] = input.multi ? [input.defaultValue] : input.defaultValue;
            }
        }

        const value = ObjectFns.deepDuplicate(resetValue);
        // if pre-fill is a single value, just embed the value
        if (Object.keys(props.prefilled).every((k) => props.prefilled[k].length == 1)) {
            for (const k of Object.keys(props.prefilled)) {
                const input = props.attack.inputs.inputs[k];
                value[k] = "multi" in input && input.multi ? [props.prefilled[k][0]] : props.prefilled[k][0];
            }
        }

        setResetValue(resetValue);
        setValue(value);
    }, [props.attack.inputs.inputs]);

    // This is called after attacks are successfully started.
    function afterAttackHandler() {
        const first = inputRefs.current.find((v) => v);
        if (first) {
            first.focus();
            if ("select" in first && typeof first["select"] == "function") first.select();
        }
    }

    const groups: { [name: string]: JSX.Element[] } = {};
    const groupOrder: string[] = [];

    function getGroup(name: string) {
        if (name in groups) return groups[name];
        groupOrder.push(name);
        return (groups[name] = []);
    }

    inputRefs.current = [];

    Object.keys(props.attack.inputs.inputs).map((key, i) => {
        const input = props.attack.inputs.inputs[key];
        if ("fixed" in input) {
            // should we show fixed inputs? could show them here
        } else if (usePrefill(props.prefilled, key)) {
            getGroup(input.group ?? "").push(
                <>
                    <div>{input.label}</div>
                    <Popup
                        trigger={
                            <span className="workspace-data-certainty-icon">
                                <em>{props.prefilled[key].filter((v) => v !== undefined).length} values</em>
                            </span>
                        }
                        position={"bottom center"}
                        on={"hover"}
                        arrow={true}
                    >
                        <div className="pane-thin">
                            <h2 className="sub-heading">Values for {key}</h2>
                            <pre>
                                {props.prefilled[key]
                                    .filter((v) => v !== undefined)
                                    .map((v) => JSON.stringify(v))
                                    .join("\n")}
                            </pre>
                        </div>
                    </Popup>
                </>,
            );
        } else {
            const Type = input.type;
            const row = (
                <Type
                    {...input.renderProps}
                    ref={(e) => (inputRefs.current[i++] = e)}
                    key={key + "_gen"}
                    value={input.multi ? value?.[key][0] : value?.[key]}
                    prefill={props.prefilled[key]}
                    valueKey={key}
                    label={input.label ?? key}
                    required={input.required ?? false}
                    autoFocus={i == 0}
                    onUpdate={(k, v) => {
                        setValue((value) => ({
                            ...value,
                            [k]: input.multi ? [v] : v,
                        }));
                    }}
                />
            );

            getGroup(input.group ?? "").push(row);
        }
    });

    return (
        <form
            className={"workspace-attacks-generic-container"}
            onSubmit={(event) => {
                event.preventDefault();
                if (value) startAttack(workspace, props.attack, props.prefilled, value).then(afterAttackHandler);
            }}
        >
            <div className={"fields"}>
                {groupOrder.map((group) =>
                    group ? (
                        <CollapsibleGroup key={group} label={group} startCollapsed={group == "Advanced"}>
                            {groups[group]}
                        </CollapsibleGroup>
                    ) : (
                        <>{groups[group]}</>
                    ),
                )}
            </div>
            <StartAttack />
        </form>
    );
}

function usePrefill(prefilled: AttackPrefill, key: string): boolean {
    return prefilled[key]?.length > 1;
}

function startAttack(
    workspace: string,
    attack: IAttackDescr,
    prefilled: AttackPrefill,
    value: { [apiJsonKey: string]: AnyApiValue },
): Promise<void> {
    return new Promise((resolve) => {
        const params: GenericAttackFormState["value"] = {
            ...value,
            workspaceUuid: workspace,
        };
        let needMultiCallArgs = [];
        for (const key of Object.keys(attack.inputs.inputs)) {
            const input = attack.inputs.inputs[key];
            if (usePrefill(prefilled, key)) {
                needMultiCallArgs.push(key);
            }

            if (!("fixed" in input)) {
                if (
                    input.required &&
                    !usePrefill(prefilled, key) &&
                    (params[key] === undefined || params[key] === "")
                ) {
                    toast.error(input.label + " must not be empty");
                    return;
                }
            }
        }

        let len = undefined;
        for (const k of needMultiCallArgs) {
            len ??= prefilled[k].length;
            if (prefilled[k].length != len)
                return toast.error(
                    "Invalid selection: prefills have different prefill value argument dimensions, can't generate API requests",
                );
        }

        if (
            needMultiCallArgs.every((k) => {
                const input = attack.inputs.inputs[k];
                return "multi" in input && input.multi;
            })
        ) {
            const keys: string[] = [];
            const values: AnyApiValue[][] = [];

            for (const key of needMultiCallArgs) {
                keys.push(key);
                values.push(prefilled[key]);
            }

            for (let i = 0; i < keys.length; i++) params[keys[i]] = values[i];
            needMultiCallArgs = [];
        }

        function send(attack: GenericAttackFormProps["attack"], params: AnyApiValue) {
            const wrappedParams: AnyApiValue = {};
            wrappedParams[attack.inputs.jsonKey] = params;
            console.log("API call", attack.inputs.endpoint, JSON.stringify(wrappedParams));
            return handleError(
                // @ts-ignore: The 'this' context of type '...' is not assignable to method's 'this' of type '...'
                Api.attacks.impl[attack.inputs.endpoint].call(Api.attacks.impl, wrappedParams),
            ).then(handleApiError((_) => _));
        }

        if (needMultiCallArgs.length == 0) {
            send(attack, params).then((_) => {
                toast.success("Attack started");
                resolve();
            });
        } else {
            const copies: (typeof params)[] = [];
            if (len === undefined) throw new Error("impossible state");
            for (let i = 0; i < len; i++) {
                const copy = { ...params };
                for (const k of needMultiCallArgs) {
                    const input = attack.inputs.inputs[k];
                    copy[k] = prefilled[k][i];
                    if (!("fixed" in input)) {
                        if (input.required && copy[k] === undefined)
                            return toast.error("selection has undefined item for required key '" + k + "'");
                        if (input.multi) copy[k] = [copy[k]];
                    }
                }
                copies.push(copy);
            }

            let finished = 0;
            let failed = 0;

            const checkAllStarted = () => {
                if (finished + failed == copies.length) {
                    if (failed == 0) {
                        toast.success("Started " + finished + " attacks");
                    } else if (finished == 0) {
                        toast.error("All " + failed + " attacks failed!");
                    } else {
                        toast.warn(finished + " attacks started, " + failed + " failed!");
                    }

                    resolve();
                }
            };

            checkAllStarted();
            for (const copy of copies) {
                send(attack, copy).then(
                    () => {
                        finished++;
                        checkAllStarted();
                    },
                    () => {
                        failed++;
                        checkAllStarted();
                    },
                );
            }
        }
    });
}

function CollapsibleGroup(props: { children: React.ReactNode; label: string; startCollapsed?: boolean }) {
    const [collapsed, setCollapsed] = useState(props.startCollapsed ?? false);

    return (
        <fieldset className={collapsed ? "collapsed" : ""}>
            <legend
                onMouseDown={(e) => {
                    setCollapsed(!collapsed);
                    e.preventDefault();
                }}
            >
                {collapsed ? <ExpandIcon /> : <CollapseIcon />} {props.label}
            </legend>
            {props.children}
        </fieldset>
    );
}
