import React, { forwardRef, useEffect, useRef, useState } from "react";
import { Api } from "../../../api/api";
import { PortOrRange, Query } from "../../../api/generated";
import Checkbox from "../../../components/checkbox";
import Input from "../../../components/input";
import SelectMenu from "../../../components/select-menu";
import { handleApiError } from "../../../utils/helper";
import { parseUserPorts } from "../../../utils/ports";

export interface IAttackInputProps extends Omit<React.HTMLProps<HTMLElement>, "ref"> {
    valueKey: string;
    label: string;
    prefill: any[] | undefined;
    value: any;
    required: boolean;
    onUpdate: (key: string, v: any) => any;
    ref: React.Ref<any>;
}

export interface AttackInputProps<T> extends IAttackInputProps {
    value: T | undefined;
    onUpdate: (key: string, v: T | undefined) => any;
}

export const StringAttackInput = forwardRef<HTMLInputElement, AttackInputProps<string>>((props, ref) => {
    let { as, value, label, valueKey, onUpdate, ref: _, onChange, ...htmlProps } = props;

    return (
        <>
            <label htmlFor={props.valueKey + "_input"} key={props.valueKey + "_label"}>
                {props.label ?? props.valueKey}
            </label>
            <Input
                ref={ref}
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
});

export function ConvertingAttackInput<T>(
    props: AttackInputProps<T> & {
        deserialize: (v: string) => T;
        serialize: (v: T | undefined) => string;
        inputRef?: React.ForwardedRef<HTMLInputElement>;
    },
) {
    let [errorInput, setErrorInput] = useState<string | undefined>(undefined);

    let { as, value, label, valueKey, onChange, onUpdate, serialize, deserialize, ref: _, ...htmlProps } = props;

    let ref = useRef<HTMLInputElement | null>();

    return (
        <>
            <label htmlFor={props.valueKey + "_input"} key={props.valueKey + "_label"}>
                {props.label ?? props.valueKey}
            </label>
            <Input
                key={props.valueKey + "_value"}
                id={props.valueKey + "_input"}
                ref={(e) => {
                    ref.current = e;
                    if (typeof props.inputRef == "function") props.inputRef(e);
                    else if (props.inputRef) props.inputRef.current = e;
                }}
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

export const PortListInput = forwardRef(
    (props: Omit<AttackInputProps<PortOrRange[] | undefined>, "ref">, ref: React.ForwardedRef<HTMLInputElement>) => {
        return (
            <ConvertingAttackInput
                ref={ref}
                deserialize={(v) => parseUserPorts(v).unwrap()}
                serialize={(v) => (v ? (typeof v === "number" ? "" + v : v.join(", ")) : "")}
                {...props}
            />
        );
    },
);

export const NumberAttackInput = forwardRef<
    HTMLInputElement,
    AttackInputProps<number> & {
        minimum?: number;
    }
>((props, ref) => {
    let minimum = props.minimum ?? 1;

    return (
        <ConvertingAttackInput
            inputRef={ref}
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
});

export const DurationAttackInput = forwardRef<
    HTMLInputElement,
    // Omit because of this error: https://stackoverflow.com/questions/70198671/react-nested-forwardref
    Omit<
        AttackInputProps<number> & {
            minimum?: number;
        },
        "ref"
    >
>((props, ref) => {
    return <NumberAttackInput ref={ref} placeholder="time in ms" {...props} label={props.label + " (ms)"} />;
});

export const BooleanAttackInput = forwardRef((props: AttackInputProps<boolean>, ref) => {
    let { value, label, valueKey, onUpdate, ref: _, onChange, ...htmlProps } = props;

    return (
        <div className="checkbox">
            <Checkbox
                ref={ref}
                id={props.valueKey + "_input"}
                value={props.value ?? false}
                onChange={(v) => props.onUpdate(props.valueKey, v)}
                {...htmlProps}
            />
            <label key={props.valueKey + "_label"} htmlFor={props.valueKey + "_input"}>
                {props.label}
            </label>
        </div>
    );
});

export const WordlistAttackInput = (props: AttackInputProps<string>) => {
    let [wordlists, setWordlists] = useState<{ label: string; value: string }[] | null>(null);

    useEffect(() => {
        if (wordlists === null) {
            Api.wordlists
                .all()
                .then(
                    handleApiError((wordlists) =>
                        setWordlists(wordlists.wordlists.map((x) => ({ label: x.name, value: x.uuid }))),
                    ),
                );
        }
    });

    let { value, label, valueKey, onUpdate, ref: _, onChange, ...htmlProps } = props;

    return (
        <>
            <label key={props.valueKey + "_label"} htmlFor={props.valueKey + "_input"}>
                {props.label ?? props.valueKey}
            </label>
            {wordlists === null ? (
                <Input value="Loading..." onChange={() => {}} readOnly />
            ) : (
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
            )}
        </>
    );
};

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

export const DehashedAttackInput = forwardRef<HTMLInputElement, AttackInputProps<Query>>((props, ref) => {
    let [search, setSearch] = useState<string>(props.value !== undefined ? Object.keys(props.value)[0] || "" : "");
    let [type, setType] = useState<null | SelectValue>(
        props.value && search ? (props.value as any)[search]?.simple || "" : "",
    );

    let { as, value, label, valueKey, onUpdate, ref: _, onChange, ...htmlProps } = props;

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
                ref={ref}
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
});
