import React, { useState } from "react";
import { toast } from "react-toastify";
import { Api, handleError } from "../../../api/api";
import "../../../styling/workspace-attacks-generic.css";
import CollapseIcon from "../../../svg/collapse";
import ExpandIcon from "../../../svg/expand";
import { handleApiError } from "../../../utils/helper";
import StartAttack from "../components/start-attack";
import { WORKSPACE_CONTEXT } from "../workspace";
import { IAttackDescr, PrefilledAttackParams, TargetType } from "../workspace-attacks";

type GenericAttackFormProps = {
    prefilled: PrefilledAttackParams;
    attack: IAttackDescr;
    targetType: TargetType | null;
};
type GenericAttackFormState = {
    attack: IAttackDescr;
    value: any;
    resetValue: any;
    prefilled: any;
};

function dupJson<T>(v: T): T {
    return JSON.parse(JSON.stringify(v));
}

export default class GenericAttackForm extends React.Component<GenericAttackFormProps, GenericAttackFormState> {
    static contextType = WORKSPACE_CONTEXT;
    declare context: React.ContextType<typeof WORKSPACE_CONTEXT>;

    constructor(props: GenericAttackFormProps) {
        super(props);

        let resetValue: any = {};
        let prefilled: any = {};
        for (const key of Object.keys(props.attack.inputs.inputs)) {
            let input = props.attack.inputs.inputs[key];
            if ("fixed" in input) {
                resetValue[key] = input.fixed;
            } else {
                resetValue[key] = input.multi ? [input.defaultValue] : input.defaultValue;

                if (typeof input.prefill == "string") {
                    if (this.props.prefilled[input.prefill] !== undefined) {
                        let v = this.props.prefilled[input.prefill];
                        prefilled[key] = v;
                        resetValue[key] = input.multi ? [v] : v;
                    }
                } else if (Array.isArray(input.prefill)) {
                    for (const prefill of input.prefill) {
                        if (this.props.prefilled[prefill] !== undefined) {
                            let v = this.props.prefilled[prefill];
                            prefilled[key] = v;
                            resetValue[key] = input.multi ? [v] : v;
                            break;
                        }
                    }
                }
            }
        }

        this.state = {
            attack: props.attack,
            resetValue: resetValue,
            value: dupJson(resetValue),
            prefilled,
        };
    }

    componentDidUpdate(prevProps: Readonly<GenericAttackFormProps>) {
        let value = this.state.value;
        let prefilled = this.state.prefilled;
        let changed = false;
        for (const key of Object.keys(this.state.attack.inputs.inputs)) {
            let input = this.state.attack.inputs.inputs[key];
            if ("fixed" in input) {
            } else {
                if (typeof input.prefill == "string") {
                    let v = this.props.prefilled[input.prefill];
                    if (v !== undefined && v !== prevProps.prefilled[input.prefill]) {
                        prefilled[key] = v;
                        value[key] = input.multi ? [v] : v;
                        changed = true;
                    }
                } else if (Array.isArray(input.prefill)) {
                    for (const prefill of input.prefill) {
                        let v = this.props.prefilled[prefill];
                        if (v !== undefined && v !== prevProps.prefilled[prefill]) {
                            prefilled[key] = v;
                            value[key] = input.multi ? [v] : v;
                            changed = true;
                            break;
                        }
                    }
                }
            }
        }
        if (changed) this.setState({ value, prefilled });
    }

    startAttack() {
        let params = {
            ...this.state.value,
            workspaceUuid: this.context.workspace.uuid,
        };
        for (const key of Object.keys(this.state.attack.inputs.inputs)) {
            let input = this.state.attack.inputs.inputs[key];
            if ("fixed" in input) {
            } else {
                if (input.required && (this.state.value[key] === undefined || this.state.value[key] === "")) {
                    toast.error(input.label + " must not be empty");
                    return;
                }
            }
        }
        let wrappedParams: any = {};
        wrappedParams[this.state.attack.inputs.jsonKey] = params;
        console.log("API call", this.state.attack.inputs.endpoint, JSON.stringify(wrappedParams));
        handleError(
            // @ts-ignore: The 'this' context of type '...' is not assignable to method's 'this' of type '...'
            Api.attacks.impl[this.state.attack.inputs.endpoint].call(Api.attacks.impl, wrappedParams) as any,
        ).then(handleApiError((_) => toast.success("Attack started")));
    }

    render() {
        let groups: { [name: string]: JSX.Element[] } = {};
        let groupOrder: string[] = [];

        function getGroup(name: string) {
            if (name in groups) return groups[name];
            groupOrder.push(name);
            return (groups[name] = []);
        }

        Object.keys(this.state.attack.inputs.inputs).map((key, i) => {
            let input = this.state.attack.inputs.inputs[key];
            if ("fixed" in input) {
                // should we show fixed inputs? could show them here
            } else {
                let Type = input.type;
                let row = (
                    <Type
                        {...input.renderProps}
                        key={key + "_gen"}
                        value={input.multi ? this.state.value[key][0] : this.state.value[key]}
                        prefill={this.state.prefilled[key]}
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
