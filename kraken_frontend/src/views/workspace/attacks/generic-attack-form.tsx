import React, { useState } from "react";
import { toast } from "react-toastify";
import Popup from "reactjs-popup";
import { Api, handleError } from "../../../api/api";
import "../../../styling/workspace-attacks-generic.css";
import CollapseIcon from "../../../svg/collapse";
import ExpandIcon from "../../../svg/expand";
import { ObjectFns, handleApiError } from "../../../utils/helper";
import StartAttack from "../components/start-attack";
import { WORKSPACE_CONTEXT } from "../workspace";
import { IAttackDescr, PrefilledAttackParams } from "../workspace-attacks";

type GenericAttackFormProps = {
    prefilled: PrefilledAttackParams[];
    attack: IAttackDescr;
};
type GenericAttackFormState = {
    value: { [apiJsonKey: string]: any };
    resetValue: { [apiJsonKey: string]: any };
    prefilled: { [apiJsonKey: string]: any[] };
};

/**
 * If a value inside `GenericAttackFormState.value` is `=== PREFILL_MULTI_MAGIC`,
 * use prefilled values when sending data.
 */
const PREFILL_MULTI_MAGIC = function () {};

export default class GenericAttackForm extends React.Component<GenericAttackFormProps, GenericAttackFormState> {
    static contextType = WORKSPACE_CONTEXT;
    declare context: React.ContextType<typeof WORKSPACE_CONTEXT>;

    constructor(props: GenericAttackFormProps) {
        super(props);

        let resetValue: GenericAttackFormState["resetValue"] = {};
        let prefilled: GenericAttackFormState["prefilled"] = {};
        for (const key of Object.keys(props.attack.inputs.inputs)) {
            let input = props.attack.inputs.inputs[key];
            if ("fixed" in input) {
                resetValue[key] = input.fixed;
            } else {
                resetValue[key] = input.multi ? [input.defaultValue] : input.defaultValue;

                this.updatePrefill(resetValue, prefilled, key);
            }
        }

        this.state = {
            resetValue: resetValue,
            value: ObjectFns.deepDuplicate(resetValue),
            prefilled,
        };
    }

    componentDidUpdate(prevProps: Readonly<GenericAttackFormProps>, prevState: Readonly<GenericAttackFormState>) {
        if (this.props.attack.inputs.inputs != prevProps.attack.inputs.inputs) {
            let resetValue: GenericAttackFormState["resetValue"] = {};
            let prefilled: GenericAttackFormState["prefilled"] = {};
            for (const key of Object.keys(this.props.attack.inputs.inputs)) {
                let input = this.props.attack.inputs.inputs[key];
                if ("fixed" in input) {
                    resetValue[key] = input.fixed;
                } else {
                    resetValue[key] = input.multi ? [input.defaultValue] : input.defaultValue;

                    this.updatePrefill(resetValue, prefilled, key);
                }
            }

            this.setState({
                resetValue,
                value: ObjectFns.deepDuplicate(resetValue),
                prefilled,
            });
        }
    }

    updatePrefill(into: GenericAttackFormState["value"], prefilled: GenericAttackFormState["prefilled"], key: string) {
        let input = this.props.attack.inputs.inputs[key];
        if (input && !("fixed" in input) && input.prefill) {
            for (const prefill of input.prefill) {
                let preprocess = input.preprocess ? input.preprocess.bind(input) : undefined;
                let v = this.props.prefilled.map((v) => (preprocess ? preprocess(v[prefill]) : v[prefill]));
                let first = v.find((v) => v !== undefined);
                if (first !== undefined) {
                    prefilled[key] = v;
                    if (v.every((v) => v === first)) {
                        if (input.preprocess) first = input.preprocess(first);
                        into[key] = input.multi ? [first] : first;
                        break;
                    } else if (v.some((v) => v !== undefined)) {
                        into[key] = PREFILL_MULTI_MAGIC;
                        break;
                    }
                }
            }
        }
    }

    startAttack() {
        let params: GenericAttackFormState["value"] = {
            ...this.state.value,
            workspaceUuid: this.context.workspace.uuid,
        };
        let needMultiCallArgs = [];
        for (const key of Object.keys(this.props.attack.inputs.inputs)) {
            let input = this.props.attack.inputs.inputs[key];
            if (params[key] === PREFILL_MULTI_MAGIC) {
                needMultiCallArgs.push(key);
            }

            if ("fixed" in input) {
            } else {
                if (input.required && (params[key] === undefined || params[key] === "")) {
                    toast.error(input.label + " must not be empty");
                    return;
                }
            }
        }

        let len = undefined;
        for (const k of needMultiCallArgs) {
            len ??= this.state.prefilled[k].length;
            if (this.state.prefilled[k].length != len)
                return toast.error(
                    "Invalid selection: prefills have different prefill value argument dimensions, can't generate API requests"
                );
        }

        if (
            needMultiCallArgs.every((k) => {
                let input = this.props.attack.inputs.inputs[k];
                return "multi" in input && input.multi;
            })
        ) {
            let keys: any[] = [];
            let values: any[][] = [];

            for (const key of needMultiCallArgs) {
                keys.push(key);
                values.push(this.state.prefilled[key]);
            }

            // deduplicate attack input parameters for multi-functions
            // for non-multi functions, see below where `copies` is created
            values = ObjectFns.transpose2D(values);
            values = ObjectFns.uniqueObjects(values);
            values = ObjectFns.transpose2D(values);
            if (keys.length != values.length) throw new Error("logic error");
            for (let i = 0; i < keys.length; i++) params[keys[i]] = values[i];
            needMultiCallArgs = [];
        }

        function send(attack: GenericAttackFormProps["attack"], params: any) {
            let wrappedParams: any = {};
            wrappedParams[attack.inputs.jsonKey] = params;
            console.log("API call", attack.inputs.endpoint, JSON.stringify(wrappedParams));
            return handleError(
                // @ts-ignore: The 'this' context of type '...' is not assignable to method's 'this' of type '...'
                Api.attacks.impl[attack.inputs.endpoint].call(Api.attacks.impl, wrappedParams) as any
            ).then(handleApiError((_) => _));
        }

        if (needMultiCallArgs.length == 0) {
            send(this.props.attack, params).then((_) => toast.success("Attack started"));
        } else {
            let copies: (typeof params)[] = [];
            if (len === undefined) throw new Error("impossible state");
            for (let i = 0; i < len; i++) {
                let copy = { ...params };
                for (const k of needMultiCallArgs) {
                    let input = this.props.attack.inputs.inputs[k];
                    copy[k] = this.state.prefilled[k][i];
                    if (!("fixed" in input)) {
                        if (input.required && copy[k] === undefined)
                            return toast.error("selection has undefined item for required key '" + k + "'");
                        if (input.multi) copy[k] = [copy[k]];
                    }
                }
                copies.push(copy);
            }

            copies = ObjectFns.uniqueObjects(copies);

            let finished = 0;
            let failed = 0;

            function checkDone() {
                if (finished + failed == copies.length) {
                    if (failed == 0) {
                        toast.success("Started " + finished + " attacks");
                    } else if (finished == 0) {
                        toast.error("All " + failed + " attacks failed!");
                    } else {
                        toast.warn(finished + " attacks started, " + failed + " failed!");
                    }
                }
            }

            checkDone();
            for (const copy of copies) {
                send(this.props.attack, copy).then(
                    (f) => {
                        finished++;
                        checkDone();
                    },
                    (e) => {
                        failed++;
                        checkDone();
                    }
                );
            }
        }
    }

    render() {
        let groups: { [name: string]: JSX.Element[] } = {};
        let groupOrder: string[] = [];

        function getGroup(name: string) {
            if (name in groups) return groups[name];
            groupOrder.push(name);
            return (groups[name] = []);
        }

        Object.keys(this.props.attack.inputs.inputs).map((key, i) => {
            let input = this.props.attack.inputs.inputs[key];
            if ("fixed" in input) {
                // should we show fixed inputs? could show them here
            } else if (this.state.value[key] === PREFILL_MULTI_MAGIC) {
                getGroup(input.group ?? "").push(
                    <>
                        <div>{input.label}</div>
                        <Popup
                            trigger={
                                <span className="workspace-data-certainty-icon">
                                    <em>
                                        {this.state.prefilled[key].filter((v) => v !== undefined).length} different
                                        values
                                    </em>
                                </span>
                            }
                            position={"bottom center"}
                            on={"hover"}
                            arrow={true}
                        >
                            <div className="pane-thin">
                                <h2 className="sub-heading">Values for {key}</h2>
                                <pre>
                                    {this.state.prefilled[key]
                                        .filter((v) => v !== undefined)
                                        .map((v) => JSON.stringify(v))
                                        .join("\n")}
                                </pre>
                            </div>
                        </Popup>
                    </>
                );
            } else {
                let Type = input.type;
                let row = (
                    <Type
                        {...input.renderProps}
                        key={key + "_gen"}
                        value={input.multi ? this.state.value[key][0] : this.state.value[key]}
                        prefill={this.state.prefilled[key] ? this.state.prefilled[key][0] : undefined}
                        valueKey={key}
                        label={input.label ?? key}
                        required={input.required ?? false}
                        autoFocus={i == 0}
                        onUpdate={(k, v) => {
                            let value = this.state.value;
                            value[k] = (input as any).multi ? [v] : v;
                            this.setState({ value });
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
                    this.startAttack();
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
                        )
                    )}
                </div>
                <StartAttack />
            </form>
        );
    }
}

function CollapsibleGroup(props: { children: any; label: string; startCollapsed?: boolean }) {
    let [collapsed, setCollapsed] = useState(props.startCollapsed ?? false);

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
