import React, { forwardRef, useEffect, useRef, useState } from "react";
import { Api } from "../../../api/api";
import { PortOrRange, Query, SearchType } from "../../../api/generated";
import Checkbox from "../../../components/checkbox";
import Input from "../../../components/input";
import { handleApiError } from "../../../utils/helper";
import { parseUserPorts } from "../../../utils/ports";
import Select from "react-select";
import { selectStyles } from "../../../components/select-menu";

export type AttackInputProps<T, PrefillT = T> = {
    valueKey: string;
    label: string;
    prefill: PrefillT[] | undefined;
    required: boolean;
    value: T | undefined;
    onUpdate: (key: string, v: T | undefined) => void;
} & Omit<React.HTMLProps<HTMLElement>, "onChange" | "onUpdate" | "prefill" | "ref" | "required" | "value" | "valueKey">;

export const StringAttackInput = forwardRef<HTMLInputElement, AttackInputProps<string>>((props, ref) => {
    const { value, label, valueKey, onUpdate, ...htmlProps } = props;

    return (
        <>
            <label htmlFor={valueKey + "_input"} key={valueKey + "_label"}>
                {label ?? valueKey}
            </label>
            <Input
                ref={ref}
                key={valueKey + "_value"}
                id={valueKey + "_input"}
                value={value || ""}
                onChange={(v) => {
                    onUpdate(valueKey, v);
                }}
                {...htmlProps}
            />
        </>
    );
});

export function ConvertingAttackInput<T, PrefillT = T>(
    props: AttackInputProps<T, PrefillT> & {
        deserialize: (v: string) => T | undefined;
        serialize: (v: T | undefined) => string;
        inputRef?: React.ForwardedRef<HTMLInputElement>;
    },
) {
    const [errorInput, setErrorInput] = useState<string | undefined>(undefined);

    const { inputRef, value, label, valueKey, onUpdate, serialize, deserialize, ...htmlProps } = props;

    const ref = useRef<HTMLInputElement | null>();

    return (
        <>
            <label htmlFor={valueKey + "_input"} key={valueKey + "_label"}>
                {label ?? valueKey}
            </label>
            <Input
                key={valueKey + "_value"}
                id={valueKey + "_input"}
                ref={(e) => {
                    ref.current = e;
                    if (typeof inputRef == "function") inputRef(e);
                    else if (inputRef) inputRef.current = e;
                }}
                value={errorInput ?? serialize(value)}
                onChange={(v) => {
                    let newValue;
                    try {
                        newValue = deserialize(v);
                    } catch (e) {
                        console.log("invalid input:", v, e);
                        setErrorInput(v);
                        ref.current?.setCustomValidity(
                            e && typeof e == "object" && "message" in e ? "" + e?.message : "Invalid input",
                        );
                        return;
                    }
                    setErrorInput(undefined);
                    ref.current?.setCustomValidity("");
                    onUpdate(valueKey, newValue);
                }}
                {...htmlProps}
            />
        </>
    );
}

export const PortListInput = forwardRef<HTMLInputElement, AttackInputProps<PortOrRange[] | undefined>>((props, ref) => {
    return (
        <ConvertingAttackInput<PortOrRange[] | undefined>
            inputRef={ref}
            deserialize={(v) => parseUserPorts(v).unwrap()}
            serialize={(v) => (v ? (typeof v === "number" ? "" + v : v.join(", ")) : "")}
            {...props}
        />
    );
});

export const NumberAttackInput = forwardRef<
    HTMLInputElement,
    AttackInputProps<number> & {
        minimum?: number;
    }
>((props, ref) => {
    const minimum = props.minimum ?? 1;

    return (
        <ConvertingAttackInput<number>
            inputRef={ref}
            deserialize={(v) => {
                if (v === "") {
                    if (props.required) throw new Error("this field is required");
                    else return undefined;
                }
                const n = Number(v);
                if (n === null || !Number.isSafeInteger(n) || n < minimum) {
                    throw new Error("can't accept for NumberAttackInput: " + v);
                }
                return n;
            }}
            serialize={(v) => (v === null || v === undefined ? "" : v.toString())}
            {...props}
        />
    );
});

/**
 * Wraps an Option<int> input - however this behaves exactly the same as a
 * regular non-required int input. This is only required since the optional
 * annotation is somehow different for the few fields where this is used.
 */
export const NullNumberAttackInput = forwardRef<
    HTMLInputElement,
    AttackInputProps<number | null, number | null | undefined> & {
        minimum?: number;
    }
>((props, ref) => {
    const minimum = props.minimum ?? 1;

    return (
        <ConvertingAttackInput<number | null, number | null | undefined>
            inputRef={ref}
            deserialize={(v) => {
                if (v === "") {
                    if (props.required) throw new Error("this field is required");
                    else return null;
                }
                const n = Number(v);
                if (n === null || !Number.isSafeInteger(n) || n < minimum) {
                    throw new Error("can't accept for NumberAttackInput: " + v);
                }
                return n;
            }}
            serialize={(v) => (v === null || v === undefined ? "" : v.toString())}
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
    const { value, label, valueKey, onUpdate, ...htmlProps } = props;

    return (
        <div className="checkbox">
            <Checkbox
                ref={ref}
                id={valueKey + "_input"}
                value={value ?? false}
                onChange={(v) => onUpdate(valueKey, v)}
                {...htmlProps}
            />
            <label key={valueKey + "_label"} htmlFor={valueKey + "_input"}>
                {label}
            </label>
        </div>
    );
});

// Don't want to rely on implementation details of Select library / what the select ref is.
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const WordlistAttackInput = forwardRef<any, AttackInputProps<string>>((props, ref) => {
    const [wordlists, setWordlists] = useState<{ label: string; value: string }[] | null>(null);

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

    const { value, label, valueKey, onUpdate } = props;

    return (
        <>
            <label key={valueKey + "_label"} htmlFor={valueKey + "_input"}>
                {label ?? valueKey}
            </label>
            {wordlists === null ? (
                <Input value="Loading..." onChange={() => {}} readOnly />
            ) : (
                <Select<{ label: string; value: string }>
                    id={"wordlist"}
                    ref={ref}
                    required
                    options={wordlists}
                    styles={selectStyles("default")}
                    value={wordlists.find((v) => v.value == value) ?? null}
                    onChange={(wordlist) => {
                        onUpdate(valueKey, wordlist?.value);
                    }}
                />
            )}
        </>
    );
});

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

const DEHASHED_SEARCH_TYPES = {
    domain: { label: "Domain", value: "domain" },
    email: { label: "Email", value: "email" },
    name: { label: "Name", value: "name" },
    username: { label: "Username", value: "username" },
    password: { label: "Password", value: "password" },
    hashed_password: { label: "Hashed password", value: "hashed_password" },
    address: { label: "Address", value: "address" },
    phone: { label: "Phone", value: "phone" },
    ip_address: { label: "IP Address", value: "ip_address" },
    vin: { label: "Vin", value: "vin" },
} as const satisfies {
    [index: string]: SelectValue;
};

export const DehashedAttackInput = forwardRef<HTMLInputElement, AttackInputProps<Query>>((props, ref) => {
    const { value, valueKey, onUpdate, ...htmlProps } = props;

    // TODO: allow switching between simple/exact/regex + possibly add OR & AND here
    type WantedSearchType = "simple" | "exact" | "regex";

    function getValue(v: SearchType): [string, WantedSearchType] {
        if ("simple" in v) {
            return [v.simple, "simple"];
        } else if ("exact" in v) {
            return [v.exact, "exact"];
        } else if ("regex" in v) {
            return [v.regex, "regex"];
        } else {
            return ["", "simple"];
        }
    }

    function getDefault(): [SelectValue | null, string, WantedSearchType] {
        if (value) {
            if ("domain" in value) {
                return [DEHASHED_SEARCH_TYPES.domain, ...getValue(value.domain)];
            } else if ("email" in value) {
                return [DEHASHED_SEARCH_TYPES.email, ...getValue(value.email)];
            } else if ("name" in value) {
                return [DEHASHED_SEARCH_TYPES.name, ...getValue(value.name)];
            } else if ("username" in value) {
                return [DEHASHED_SEARCH_TYPES.username, ...getValue(value.username)];
            } else if ("password" in value) {
                return [DEHASHED_SEARCH_TYPES.password, ...getValue(value.password)];
            } else if ("hashedPassword" in value) {
                return [DEHASHED_SEARCH_TYPES.hashed_password, ...getValue(value.hashedPassword)];
            } else if ("address" in value) {
                return [DEHASHED_SEARCH_TYPES.address, ...getValue(value.address)];
            } else if ("phone" in value) {
                return [DEHASHED_SEARCH_TYPES.phone, ...getValue(value.phone)];
            } else if ("ipAddress" in value) {
                return [DEHASHED_SEARCH_TYPES.ip_address, ...getValue(value.ipAddress)];
            } else if ("vin" in value) {
                return [DEHASHED_SEARCH_TYPES.vin, ...getValue(value.vin)];
            } else {
                const _exhaustiveCheck: never = value;
            }
        }
        return [null, "", "simple"];
    }

    const [defaultType, defaultSearch] = getDefault();

    const [search, setSearch] = useState<string>(defaultSearch);
    const [type, setType] = useState<null | SelectValue>(defaultType);

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

        onUpdate(valueKey, query);
    }

    return (
        <>
            <Select<SelectValue>
                key={valueKey + "_select"}
                required
                options={Object.values(DEHASHED_SEARCH_TYPES)}
                styles={selectStyles("default")}
                value={type}
                onChange={(type) => {
                    setType(type);
                    update(type, search);
                }}
            />
            <Input
                ref={ref}
                key={valueKey + "_value"}
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
