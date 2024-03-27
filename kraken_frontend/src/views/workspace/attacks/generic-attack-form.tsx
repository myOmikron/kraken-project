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

/**
 * The react component props for the `GenericAttackForm`
 */
type GenericAttackFormProps = {
    /**
     * Pre-generated input values for every attack input. If more then one is
     * set, the input value will not be editable and only the prefill can be
     * submitted.
     *
     * For a single value, the value that started off as the prefill value may
     * be edited still.
     *
     * The prefill is generated in `../workspace-attacks.tsx`.
     */
    prefilled: AttackPrefill;

    /**
     * The form description including all inputs and labels and extras.
     *
     * Comes from {@link ATTACKS} in `../workspace-attacks.tsx`
     */
    attack: IAttackDescr;
};

/**
 * Holds the value of all the input parameters that are later passed into the
 * API after a little bit of pre-processing.
 */
type GenericFormInputs = {
    [apiJsonKey: string]: AnyApiValue;
};

/**
 * An automatically generated form based on an {@link IAttackDescr}, which
 * concretely probably comes from {@link ATTACKS} (of type {@link AllAttackDescr})
 */
export default function GenericAttackForm(props: GenericAttackFormProps) {
    const {
        workspace: { uuid: workspace },
    } = React.useContext(WORKSPACE_CONTEXT);

    const inputRefs = useRef<Array<HTMLElement | null>>([]);

    const [resetValue, setResetValue] = React.useState<GenericFormInputs>();
    const [value, setValue] = React.useState<GenericFormInputs>();

    useEffect(() => {
        const newResetValue: typeof resetValue = {};
        for (const key of Object.keys(props.attack.inputs.inputs)) {
            const input = props.attack.inputs.inputs[key];
            if ("fixed" in input) {
                newResetValue[key] = input.fixed;
            } else {
                newResetValue[key] = input.multi ? [input.defaultValue] : input.defaultValue;
            }
        }

        const value = ObjectFns.deepDuplicate(newResetValue);
        // if pre-fill is a single value, just embed the value
        if (Object.keys(props.prefilled).every((k) => props.prefilled[k].length == 1)) {
            for (const k of Object.keys(props.prefilled)) {
                const input = props.attack.inputs.inputs[k];
                value[k] = "multi" in input && input.multi ? [props.prefilled[k][0]] : props.prefilled[k][0];
            }
        }

        setResetValue(newResetValue);
        setValue(value);
    }, [props.attack.inputs.inputs]);

    /** This is called after attacks are successfully started. */
    function afterAttackHandler() {
        const first = inputRefs.current.find((v) => v);
        if (first) {
            first.focus();
            if ("select" in first && typeof first["select"] == "function") first.select();
        }
    }

    const groups = new RenderGrouping();

    inputRefs.current = [];

    Object.keys(props.attack.inputs.inputs).map((key, i) => {
        const input = props.attack.inputs.inputs[key];
        if ("fixed" in input) {
            // should we show fixed inputs? could show them here
        } else if (usePrefill(props.prefilled, key)) {
            groups.get(input.group ?? "").push(
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
                    valueKey={key}
                    label={input.label ?? key}
                    required={input.required ?? false}
                    autoFocus={i == 0}
                    onUpdate={(v) => {
                        setValue((value) => ({
                            ...value,
                            [key]: input.multi ? [v] : v,
                        }));
                    }}
                />
            );

            groups.get(input.group ?? "").push(row);
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
                {groups.map((name, group) =>
                    group ? (
                        <CollapsibleGroup key={name} label={name} startCollapsed={name == "Advanced"}>
                            {group}
                        </CollapsibleGroup>
                    ) : (
                        <>{group}</>
                    ),
                )}
            </div>
            <StartAttack />
        </form>
    );
}

/**
 * Inputs can be grouped via the attack description `group`. The default
 * group is the empty string.
 *
 * Groups are collapsible sections, using {@link CollapsibleGroup}
 */
class RenderGrouping {
    private groups: { [name: string]: JSX.Element[] } = {};
    private groupOrder: string[] = [];

    /**
     * @param name The name, human readable and used as display - matched exactly
     *
     * @returns The existing group array reference with this name or allocating
     * a new empty one if not yet seen. Keeps the order of the group names.
     */
    get(name: string) {
        if (name in this.groups) return this.groups[name];
        this.groupOrder.push(name);
        return (this.groups[name] = []);
    }

    /**
     * Iterates over all the previously seen groups that called `get`.
     *
     * @param cb Callback that is run exactly once for each group, in insertion
     * order. The first parameter being the name as passed in to `get` and the
     * second parameter being the reference that you would get using `get`.
     *
     * @returns the result of all callback function calls as array.
     */
    map<T>(cb: (name: string, group: JSX.Element[]) => T): T[] {
        return this.groupOrder.map((name) => cb(name, this.groups[name]));
    }
}

/**
 * Given the prefilled data and the JSON key to access it, check if we should
 * use the prefill as-is or render and use editable input fields.
 *
 * @param prefilled the raw prefilled object passed into `GenericAttackForm`
 * `props.prefilled`
 * @param key The input key, same as the `IAttackDescr` inputs key as well as
 * the API JSON key.
 *
 * @returns `true` if the uneditable prefill should be used or `false` if the
 * regular editable value should be used.
 */
function usePrefill(prefilled: AttackPrefill, key: string): boolean {
    return prefilled[key]?.length > 1;
}

/**
 * Splits the raw input values from `value` into multiple API requests, as
 * needed and defined by the `IAttackDescr`. Then invokes all the API requests
 * and resolves the promise once all have been started successfully.
 *
 * Shows UI toasts on success/error.
 *
 * @param workspace the workspace to operate on for the API
 * @param attack the attack description that is used to determine what values
 * can be passed in as array and which ones need to be split into multiple
 * requests.
 * @param prefilled the prefill data, used when {@link usePrefill} returns
 * `true` for a key instead of the passed in value.
 * @param value the raw API JSON data that should be split up and sent.
 *
 * @returns a Promise that can be awaited to wait until all API requests have
 * been started.
 */
function startAttack(
    workspace: string,
    attack: IAttackDescr,
    prefilled: AttackPrefill,
    value: GenericFormInputs,
): Promise<void> {
    return new Promise((resolve) => {
        const params: typeof value = {
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

        /**
         * Wrapper around the `Api.attacks.*` API call for a single already
         * split argument JSON
         *
         * @param attack the attack of which to wrap the function for
         * @param params the raw JSON value to send there, which is always
         * wrapped into another key (which this function takes care of)
         * @returns a promise that resolves once the attack has started.
         */
        const send = (attack: IAttackDescr, params: AnyApiValue) => {
            const wrappedParams: AnyApiValue = {};
            wrappedParams[attack.inputs.jsonKey] = params;
            // We log the attack input for debugging purposes. Could extend this
            // to log to the server (telemetry) as well, including the selection
            // shape so we can figure out which inputs are mixed together usually.
            // eslint-disable-next-line no-console
            console.log("API call", attack.inputs.endpoint, JSON.stringify(wrappedParams));
            return handleError(
                // @ts-ignore: The 'this' context of type '...' is not assignable to method's 'this' of type '...'
                Api.attacks.impl[attack.inputs.endpoint].call(Api.attacks.impl, wrappedParams),
            ).then(handleApiError((_) => _));
        };

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

            /**
             * helper function, resolves the promise once all attacks have been
             * started.
             */
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

/** Wrapper for fieldset/legend - just for the genreic attack input form */
function CollapsibleGroup(props: {
    /** The DOM to render within the collapsible group */
    children: React.ReactNode;
    /** Human readable display name (the group name) */
    label: string;
    /** if true, start in collapsed state instead of revealed state */
    startCollapsed?: boolean;
}) {
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
