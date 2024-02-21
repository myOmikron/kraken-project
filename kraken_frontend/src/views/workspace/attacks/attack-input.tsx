import React, { useEffect, useRef, useState } from "react";
import Input from "../../../components/input";
import Checkbox from "../../../components/checkbox";
import { PortOrRange, Query } from "../../../api/generated";
import { parseUserPorts } from "../../../utils/ports";
import { Api } from "../../../api/api";
import { handleApiError } from "../../../utils/helper";
import SelectMenu from "../../../components/select-menu";

export interface IAttackInputProps extends React.HTMLProps<HTMLElement> {
    valueKey: string;
    label: string;
    prefill: string | undefined;
    value: any;
    required: boolean;
    onUpdate: (key: string, v: any) => any;
}

export interface AttackInputProps<T> extends IAttackInputProps {
    value: T | undefined;
    onUpdate: (key: string, v: T | undefined) => any;
}

export function StringAttackInput(props: AttackInputProps<string>) {
    let htmlProps: any = { ...props };
    delete htmlProps["value"];
    delete htmlProps["label"];
    delete htmlProps["valueKey"];
    delete htmlProps["validate"];
    delete htmlProps["onUpdate"];

    return (
        <>
            <label htmlFor={props.valueKey + "_input"} key={props.valueKey + "_label"}>
                {props.label ?? props.valueKey}
            </label>
            <Input
                key={props.valueKey + "_value"}
                id={props.valueKey + "_input"}
                value={props.value || ""}
                onChange={(v) => {
                    props.onUpdate(props.valueKey, v);
                }}
                {...htmlProps}
            />
        </>
    );
}

export function ConvertingAttackInput<T>(
    props: AttackInputProps<T> & {
        deserialize: (v: string) => T;
        serialize: (v: T | undefined) => string;
    },
) {
    let [errorInput, setErrorInput] = useState<string | undefined>(undefined);

    let htmlProps: any = { ...props };
    delete htmlProps["value"];
    delete htmlProps["label"];
    delete htmlProps["valueKey"];
    delete htmlProps["validate"];
    delete htmlProps["onUpdate"];
    delete htmlProps["serialize"];
    delete htmlProps["deserialize"];

    let ref = useRef<HTMLInputElement>();

    return (
        <>
            <label htmlFor={props.valueKey + "_input"} key={props.valueKey + "_label"}>
                {props.label ?? props.valueKey}
            </label>
            <Input
                key={props.valueKey + "_value"}
                id={props.valueKey + "_input"}
                ref={ref}
                value={errorInput ?? props.serialize(props.value)}
                onChange={(v) => {
                    let newValue;
                    try {
                        newValue = props.deserialize(v);
                    } catch (e) {
                        console.log("invalid input:", v, e);
                        setErrorInput(v);
                        ref.current?.setCustomValidity((e as any)?.message || "Invalid input");
                        return;
                    }
                    setErrorInput(undefined);
                    ref.current?.setCustomValidity("");
                    props.onUpdate(props.valueKey, newValue);
                }}
                {...htmlProps}
            />
        </>
    );
}

export function PortListInput(props: AttackInputProps<PortOrRange[] | undefined>) {
    return (
        <ConvertingAttackInput
            deserialize={(v) => parseUserPorts(v).unwrap()}
            serialize={(v) => (v ? (typeof v === "number" ? "" + v : v.join(", ")) : "")}
            {...props}
        />
    );
}

export function NumberAttackInput(
    props: AttackInputProps<number> & {
        minimum?: number;
    },
) {
    let minimum = props.minimum ?? 1;

    return (
        <ConvertingAttackInput
            deserialize={(v) => {
                const n = Number(v);
                if (n === null || !Number.isSafeInteger(n) || n < minimum) {
                    throw new Error("can't accept for NumberAttackInput: " + v);
                }
                return n;
            }}
            serialize={(v) => (v ? v.toString() : "")}
            {...props}
        />
    );
}

export function DurationAttackInput(
    props: AttackInputProps<number> & {
        minimum?: number;
    },
) {
    return <NumberAttackInput placeholder="time in ms" {...props} label={props.label + " (ms)"} />;
}

export function BooleanAttackInput(props: AttackInputProps<boolean>) {
    let htmlProps: any = { ...props };
    delete htmlProps["value"];
    delete htmlProps["label"];
    delete htmlProps["valueKey"];
    delete htmlProps["validate"];
    delete htmlProps["onUpdate"];

    return (
        <div className="checkbox">
            <Checkbox
                id={props.valueKey + "_input"}
                value={props.value}
                onChange={(v) => props.onUpdate(props.valueKey, v)}
                {...htmlProps}
            />
            <label key={props.valueKey + "_label"} htmlFor={props.valueKey + "_input"}>
                {props.label}
            </label>
        </div>
    );
}

export function WordlistAttackInput(props: AttackInputProps<string>) {
    let [wordlists, setWordlists] = useState<{ label: string; value: string }[]>([]);

    useEffect(() => {
        Api.wordlists
            .all()
            .then(
                handleApiError((wordlists) =>
                    setWordlists(wordlists.wordlists.map((x) => ({ label: x.name, value: x.uuid }))),
                ),
            );
    });

    let htmlProps: any = { ...props };
    delete htmlProps["value"];
    delete htmlProps["label"];
    delete htmlProps["valueKey"];
    delete htmlProps["validate"];
    delete htmlProps["onUpdate"];

    return (
        <>
            <label key={props.valueKey + "_label"} htmlFor={props.valueKey + "_input"}>
                {props.label ?? props.valueKey}
            </label>
            <SelectMenu
                id={"wordlist"}
                required
                options={wordlists}
                theme={"default"}
                value={wordlists.find((v) => v.value == props.value) ?? null}
                onChange={(wordlist) => {
                    props.onUpdate(props.valueKey, wordlist?.value);
                }}
            />
        </>
    );
}

export type DehashedQueryType =
    | "email"
    | "domain"
    | "vin"
    | "username"
    | "password"
    | "hashed_password"
    | "address"
    | "phone"
    | "name"
    | "ip_address";

type SelectValue = {
    label: string;
    value: DehashedQueryType;
};

const DEHASHED_SEARCH_TYPES: Array<SelectValue> = [
    { label: "Domain", value: "domain" },
    { label: "Email", value: "email" },
    { label: "Name", value: "name" },
    { label: "Username", value: "username" },
    { label: "Password", value: "password" },
    { label: "Hashed password", value: "hashed_password" },
    { label: "Address", value: "address" },
    { label: "Phone", value: "phone" },
    { label: "IP Address", value: "ip_address" },
    { label: "Vin", value: "vin" },
];

export function DehashedAttackInput(props: AttackInputProps<Query>) {
    let [search, setSearch] = useState<string>(props.prefill || "");
    let [type, setType] = useState<null | SelectValue>(null);

    let htmlProps: any = { ...props };
    delete htmlProps["value"];
    delete htmlProps["label"];
    delete htmlProps["valueKey"];
    delete htmlProps["validate"];
    delete htmlProps["onUpdate"];

    function update(type: null | SelectValue, search: string) {
        let query;
        switch (type?.value) {
            case "email":
                query = { email: { simple: search } };
                break;
            case "domain":
                query = { domain: { simple: search } };
                break;
            case "vin":
                query = { vin: { simple: search } };
                break;
            case "username":
                query = { username: { simple: search } };
                break;
            case "password":
                query = { password: { simple: search } };
                break;
            case "hashed_password":
                query = { hashedPassword: { simple: search } };
                break;
            case "address":
                query = { address: { simple: search } };
                break;
            case "phone":
                query = { phone: { simple: search } };
                break;
            case "name":
                query = { name: { simple: search } };
                break;
            case "ip_address":
                query = { ipAddress: { simple: search } };
                break;
            default:
                console.error("Encountered unknown type");
                break;
        }

        props.onUpdate(props.valueKey, query);
        console.log("update:", query);
    }

    return (
        <>
            <SelectMenu
                key={props.valueKey + "_select"}
                required
                options={DEHASHED_SEARCH_TYPES}
                theme={"default"}
                value={type}
                onChange={(type) => {
                    setType(type);
                    update(type, search);
                }}
            />
            <Input
                key={props.valueKey + "_value"}
                placeholder={"dehashed query"}
                {...htmlProps}
                value={search}
                onChange={(search) => {
                    setSearch(search);
                    update(type, search);
                }}
            />
        </>
    );
}
