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
import { IAttackDescr } from "../workspace-attacks";

type GenericAttackFormProps = {
    prefilled: { [key: string]: any[] };
    attack: IAttackDescr;
};
type GenericAttackFormState = {
    value: { [apiJsonKey: string]: any };
    resetValue: { [apiJsonKey: string]: any };
};

export default class GenericAttackForm extends React.Component<GenericAttackFormProps, GenericAttackFormState> {
    static contextType = WORKSPACE_CONTEXT;
    declare context: React.ContextType<typeof WORKSPACE_CONTEXT>;
    declare inputRefs: (HTMLElement | null)[];

    constructor(props: GenericAttackFormProps) {
        super(props);

        let resetValue: GenericAttackFormState["resetValue"] = {};
        for (const key of Object.keys(props.attack.inputs.inputs)) {
            let input = props.attack.inputs.inputs[key];
            if ("fixed" in input) {
                resetValue[key] = input.fixed;
            } else {
                resetValue[key] = input.multi ? [input.defaultValue] : input.defaultValue;
            }
        }

        let value = ObjectFns.deepDuplicate(resetValue);
        // if pre-fill is a single value, just embed the value
        if (Object.keys(props.prefilled).every((k) => props.prefilled[k].length == 1)) {
            for (const k of Object.keys(props.prefilled)) {
                let input = props.attack.inputs.inputs[k];
                value[k] = "multi" in input && input.multi ? [props.prefilled[k][0]] : props.prefilled[k][0];
            }
        }

        this.inputRefs = [];

        this.state = {
            resetValue,
            value,
        };
    }

    componentDidUpdate(prevProps: Readonly<GenericAttackFormProps>, prevState: Readonly<GenericAttackFormState>) {
        if (this.props.attack.inputs.inputs != prevProps.attack.inputs.inputs) {
            let resetValue: GenericAttackFormState["resetValue"] = {};
            for (const key of Object.keys(this.props.attack.inputs.inputs)) {
                let input = this.props.attack.inputs.inputs[key];
                if ("fixed" in input) {
                    resetValue[key] = input.fixed;
                } else {
                    resetValue[key] = input.multi ? [input.defaultValue] : input.defaultValue;
                }
            }

            this.setState({
                resetValue,
                value: ObjectFns.deepDuplicate(resetValue),
            });
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
            if (this.usePrefill(key)) {
                needMultiCallArgs.push(key);
            }

            if ("fixed" in input) {
            } else {
                if (input.required && !this.usePrefill(key) && (params[key] === undefined || params[key] === "")) {
                    toast.error(input.label + " must not be empty");
                    return;
                }
            }
        }

        let len = undefined;
        for (const k of needMultiCallArgs) {
            len ??= this.props.prefilled[k].length;
            if (this.props.prefilled[k].length != len)
                return toast.error(
                    "Invalid selection: prefills have different prefill value argument dimensions, can't generate API requests",
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
                values.push(this.props.prefilled[key]);
            }

            for (let i = 0; i < keys.length; i++) params[keys[i]] = values[i];
            needMultiCallArgs = [];
        }

        function send(attack: GenericAttackFormProps["attack"], params: any) {
            let wrappedParams: any = {};
            wrappedParams[attack.inputs.jsonKey] = params;
            console.log("API call", attack.inputs.endpoint, JSON.stringify(wrappedParams));
            return handleError(
                // @ts-ignore: The 'this' context of type '...' is not assignable to method's 'this' of type '...'
                Api.attacks.impl[attack.inputs.endpoint].call(Api.attacks.impl, wrappedParams) as any,
            ).then(handleApiError((_) => _));
        }

        if (needMultiCallArgs.length == 0) {
            send(this.props.attack, params).then((_) => {
                toast.success("Attack started");
                this.afterAttackHandler();
            });
        } else {
            let copies: (typeof params)[] = [];
            if (len === undefined) throw new Error("impossible state");
            for (let i = 0; i < len; i++) {
                let copy = { ...params };
                for (const k of needMultiCallArgs) {
                    let input = this.props.attack.inputs.inputs[k];
                    copy[k] = this.props.prefilled[k][i];
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

                    this.afterAttackHandler();
                }
            };

            checkAllStarted();
            for (const copy of copies) {
                send(this.props.attack, copy).then(
                    (f) => {
                        finished++;
                        checkAllStarted();
                    },
                    (e) => {
                        failed++;
                        checkAllStarted();
                    },
                );
            }
        }
    }

    // This is called after attacks are successfully started.
    afterAttackHandler() {
        let first = this.inputRefs.find((v) => v);
        if (first) {
            first.focus();
            if ("select" in first && typeof first["select"] == "function") first.select();
        }
    }

    usePrefill(key: string): boolean {
        return this.props.prefilled[key]?.length > 1;
    }

    render() {
        let groups: { [name: string]: JSX.Element[] } = {};
        let groupOrder: string[] = [];

        function getGroup(name: string) {
            if (name in groups) return groups[name];
            groupOrder.push(name);
            return (groups[name] = []);
        }

        this.inputRefs = [];
        let i = 0;

        Object.keys(this.props.attack.inputs.inputs).map((key, i) => {
            let input = this.props.attack.inputs.inputs[key];
            if ("fixed" in input) {
                // should we show fixed inputs? could show them here
            } else if (this.usePrefill(key)) {
                getGroup(input.group ?? "").push(
                    <>
                        <div>{input.label}</div>
                        <Popup
                            trigger={
                                <span className="workspace-data-certainty-icon">
                                    <em>{this.props.prefilled[key].filter((v) => v !== undefined).length} values</em>
                                </span>
                            }
                            position={"bottom center"}
                            on={"hover"}
                            arrow={true}
                        >
                            <div className="pane-thin">
                                <h2 className="sub-heading">Values for {key}</h2>
                                <pre>
                                    {this.props.prefilled[key]
                                        .filter((v) => v !== undefined)
                                        .map((v) => JSON.stringify(v))
                                        .join("\n")}
                                </pre>
                            </div>
                        </Popup>
                    </>,
                );
            } else {
                let Type = input.type;
                let row = (
                    <Type
                        {...input.renderProps}
                        ref={(e) => (this.inputRefs[i++] = e)}
                        key={key + "_gen"}
                        value={input.multi ? this.state.value[key][0] : this.state.value[key]}
                        prefill={this.props.prefilled[key]}
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
                        ),
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
