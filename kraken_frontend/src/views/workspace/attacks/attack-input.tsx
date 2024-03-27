import React, { forwardRef, useEffect, useRef, useState } from "react";
import Select from "react-select";
import { Api } from "../../../api/api";
import { PortOrRange, Query, SearchType } from "../../../api/generated";
import Checkbox from "../../../components/checkbox";
import Input from "../../../components/input";
import { selectStyles } from "../../../components/select-menu";
import { handleApiError } from "../../../utils/helper";
import { parseUserPorts } from "../../../utils/ports";

/**
 * The react component props for every attack input that processes a value of
 * type `T`.
 *
 * This is used in {@link GenericAttackForm} for every attack input.
 *
 * The component with this prop type is passed into an {@link AttackInput} of
 * matching template type `T`.
 */
export type AttackInputProps<T> = {
    /** A form-unique string that can be used as prefix for HTML IDs / label `for` */
    valueKey: string;
    /** Human readable text that is printed next to the input as a label */
    label: string;
    /** Equals to the corresponding `AttackInput.required` */
    required: boolean;
    /**
     * The current value, which may already start off using some data based on the
     * wanted prefill generated from the selected data and the `AttackInput` definition.
     */
    value: T | undefined;
    /** Update event handler called when the input changes. */
    onUpdate: (v: T | undefined) => void;
} & Omit<React.HTMLProps<HTMLElement>, "onChange" | "onUpdate" | "ref" | "required" | "value" | "valueKey">;

/**
 * Input component to manipulate a string attack input.
 */
export const StringAttackInput = forwardRef<HTMLInputElement, AttackInputProps<string>>((props, ref) => {
    const { value, label, valueKey, onUpdate, ...htmlProps } = props;

    return (
        <React.Fragment key={valueKey}>
            <label htmlFor={valueKey + "_input"}>{label || valueKey}</label>
            <Input ref={ref} id={valueKey + "_input"} value={value || ""} onChange={onUpdate} {...htmlProps} />
        </React.Fragment>
    );
});

/**
 * Input component to manipulate an attack input on string basis, with a
 * `deserialize` & `serialize` callback for any custom types.
 *
 * Thrown exceptions in deserialize will be shown as error message to the user
 * through HTML5 input validity and prevent form submission.
 */
export function ConvertingAttackInput<T>(
    props: AttackInputProps<T> & {
        /**
         * Callback used when converting user input (string) into the value to
         * be used in the attack API.
         */
        deserialize: (v: string) => T | undefined;
        /**
         * Callback used when converting the attack API value to a human-readable
         * and editable string.
         */
        serialize: (v: T | undefined) => string;
        /**
         * Optional parameter, much like `ref` on a regular component, but will
         * apply to the HTMLInputElement, so it can be focused, etc.
         */
        inputRef?: React.ForwardedRef<HTMLInputElement>;
    },
) {
    const [errorInput, setErrorInput] = useState<string | undefined>(undefined);

    const { inputRef, value, label, valueKey, onUpdate, serialize, deserialize, ...htmlProps } = props;

    const ref = useRef<HTMLInputElement | null>();

    return (
        <React.Fragment key={valueKey}>
            <label htmlFor={valueKey + "_input"}>{label || valueKey}</label>
            <Input
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
                        // This console log can be used if users are having
                        // problems with rejected inputs in deployment so we can
                        // ask them to open the console and send us the warning.
                        // eslint-disable-next-line no-console
                        console.warn("invalid input:", v, e);
                        setErrorInput(v);
                        ref.current?.setCustomValidity(
                            e && typeof e == "object" && "message" in e ? "" + e?.message : "Invalid input",
                        );
                        return;
                    }
                    setErrorInput(undefined);
                    ref.current?.setCustomValidity("");
                    onUpdate(newValue);
                }}
                {...htmlProps}
            />
        </React.Fragment>
    );
}

/**
 * Input for a port list (`PortOrRange[] | undefined`), using a
 * {@link ConvertingAttackInput}.
 *
 * Uses {@link parseUserPorts} to parse the user input.
 */
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

/**
 * Input for whole integers, using a {@link ConvertingAttackInput}.
 */
export const NumberAttackInput = forwardRef<
    HTMLInputElement,
    AttackInputProps<number> & {
        /**
         * Minimum allowed number, defaults to 1.
         *
         * This should eventually be configurable through the `AttackInput`,
         * currently never set.
         */
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
 *
 * @see {@link NumberAttackInput}
 */
export const NullNumberAttackInput = forwardRef<
    HTMLInputElement,
    AttackInputProps<number | null> & {
        /**
         * Minimum allowed number, defaults to 1.
         *
         * This should eventually be configurable through the `AttackInput`,
         * currently never set.
         */
        minimum?: number;
    }
>((props, ref) => {
    const minimum = props.minimum ?? 1;

    return (
        <ConvertingAttackInput<number | null>
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

/**
 * Just a {@link NumberAttackInput}, but indicating that the number is in
 * milliseconds to the user.
 */
export const DurationAttackInput = forwardRef<
    HTMLInputElement,
    // Omit because of this error: https://stackoverflow.com/questions/70198671/react-nested-forwardref
    Omit<
        AttackInputProps<number> & {
            /**
             * Minimum allowed number, defaults to 1.
             *
             * This should eventually be configurable through the `AttackInput`,
             * currently never set.
             */
            minimum?: number;
        },
        "ref"
    >
>((props, ref) => {
    return <NumberAttackInput ref={ref} placeholder="time in ms" {...props} label={props.label + " (ms)"} />;
});

/**
 * A checkbox component for a boolean attack input.
 */
export const BooleanAttackInput = forwardRef((props: AttackInputProps<boolean>, ref) => {
    const { value, label, valueKey, onUpdate, ...htmlProps } = props;

    return (
        <div className="checkbox" key={valueKey}>
            <Checkbox ref={ref} id={valueKey + "_input"} value={value ?? false} onChange={onUpdate} {...htmlProps} />
            <label htmlFor={valueKey + "_input"}>{label}</label>
        </div>
    );
});

/**
 * A dropdown select input where you can select a wordlist. The value is passed
 * as UUID into the API / value / onChange parameter.
 */
// Don't want to rely on implementation details of Select library / what the select ref is.
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const WordlistAttackInput = forwardRef<any, AttackInputProps<string>>((props, ref) => {
    /** Value type used in this `Select` component */
    type SelectType = {
        /** Select option label */
        label: string;
        /** Select option value (UUID) */
        value: string;
    };
    const [wordlists, setWordlists] = useState<SelectType[] | null>(null);

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
        <React.Fragment key={valueKey}>
            <label htmlFor={valueKey + "_input"}>{label || valueKey}</label>
            {wordlists === null ? (
                <Input value="Loading..." onChange={() => {}} readOnly />
            ) : (
                <Select<SelectType>
                    id={"wordlist"}
                    ref={ref}
                    required
                    options={wordlists}
                    styles={selectStyles("default")}
                    value={wordlists.find((v) => v.value == value) ?? null}
                    onChange={(wordlist) => {
                        onUpdate(wordlist?.value);
                    }}
                />
            )}
        </React.Fragment>
    );
});

/**
 * Dehashed query type as defined by the kraken dehashed API. The strings here
 * correspond to the object keys for each different variant type.
 *
 * Used in the `DehashedAttackInput`
 */
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

/** Value type used in the `DehashedAttackInput` dropdown Select component */
type DehashedSelectType = {
    /** Select option label */
    label: string;
    /** Select option value */
    value: DehashedQueryType;
};

/**
 * All dropdown values that are selectable in the `DehashedAttackInput`.
 */
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
    [index: string]: DehashedSelectType;
};

// TODO: allow switching between simple/exact/regex + possibly add OR & AND here
/**
 * Search types for the Dehashed query "SearchType" variant. Currently only
 * simple is ever output for human input, but the input will render from
 * these and may be extended upon in the future as well.
 */
type WantedSearchType = "simple" | "exact" | "regex";

/**
 * For a given already existing API value, get the `WantedSearchType` it
 * corresponds to.
 *
 * @param v The search of type `SearchType` to extract the value from.
 *
 * @returns A tuple of 2 elements:
 * 1. the input as a string to render in the input box
 * 2. The type how the string is interpreted in the API later. (currently not exposed in the UI)
 */
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

/**
 * Get the default value for the dehashed input, based on the passed in
 * value.
 *
 * @returns A tuple of 3 elements based on the `props.value`:
 * 1. The value for the dropdown select on the left
 * 2. the input as a string to render in the input box
 * 3. The type how the string is interpreted in the API later. (currently not exposed in the UI)
 */
function getDefault(value: Query | undefined): [DehashedSelectType | null, string, WantedSearchType] {
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

/**
 * An attack input specially made for the dehashed attack. Can select the search
 * type using a dropdown in the first colum and the value using an input in the
 * second column.
 *
 * May be extended to provide more rows of settings in the future.
 */
export const DehashedAttackInput = forwardRef<HTMLInputElement, AttackInputProps<Query>>((props, ref) => {
    const { value, valueKey, onUpdate, ...htmlProps } = props;

    const [loadedValueKey, setLoadedValueKey] = useState<string>();
    const [search, setSearch] = useState<string>("");
    const [type, setType] = useState<null | DehashedSelectType>(null);

    useEffect(() => {
        if (loadedValueKey === valueKey) return;
        if (typeof value == "object") {
            const [defaultType, defaultSearch] = getDefault(value);
            setType(defaultType);
            setSearch(defaultSearch);
            setLoadedValueKey(valueKey);
        }
    }, [valueKey, value]);

    /**
     * Updates the full `Query` API input value and sends it to the form using
     * `onUpdate`.
     *
     * @param type the dropdown value defining the API object type, or undefined
     * to clear the whole API value.
     * @param search the string to search for - currently always emitted as
     * `simple` query and not yet using the `WantedSearchType`.
     */
    function update(type: null | DehashedSelectType, search: string) {
        let query;
        switch (type?.value) {
            case undefined:
                break;
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
        }

        onUpdate(query);
    }

    return (
        <React.Fragment key={valueKey}>
            <Select<DehashedSelectType>
                required={props.required}
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
                placeholder={"dehashed query"}
                {...htmlProps}
                value={search}
                onChange={(search) => {
                    setSearch(search);
                    update(type, search);
                }}
            />
        </React.Fragment>
    );
});
