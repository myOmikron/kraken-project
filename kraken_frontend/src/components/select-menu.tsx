import React from "react";
import Select, { StylesConfig } from "react-select";
import { GroupBase } from "react-select/dist/declarations/src/types";

type SelectMenuProps = {
    options: Array<SelectValue>;
    theme?: Theme;
    value: null | SelectValue;
    onChange: (value: SelectValue | null) => void;
    id?: string;
    required?: boolean;
};
type SelectMenuState = {};

type Theme = "default" | "red";

type SelectValue = {
    label: string;
    value: any;
};

export default class SelectMenu extends React.Component<SelectMenuProps, SelectMenuState> {
    declare ref: React.Ref<any>;

    constructor(props: SelectMenuProps) {
        super(props);

        this.ref = React.createRef();

        this.state = {
            type: null,
        };
    }

    render() {
        return (
            <Select<SelectValue>
                ref={this.ref}
                id={this.props.id}
                required={this.props.required}
                options={this.props.options}
                onChange={(type) => {
                    this.props.onChange(type);
                }}
                autoFocus={false}
                value={this.props.value}
                styles={selectStyles(this.props.theme || "default")}
            />
        );
    }
}

export type SelectPrimitiveProps<T extends { toString(): string }> = {
    options: Array<T>;
    theme?: Theme;
    value: null | T;
    onChange: (value: T | null) => void;
    id?: string;
    required?: boolean;
    isClearable?: boolean;
};

/** A {@link Select `<Select />} which works with primitives (i.e. strings and numbers) instead of objects */
export function SelectPrimitive<T extends { toString(): string }>(props: SelectPrimitiveProps<T>) {
    return (
        <Select<{ label: string; value: T }>
            id={props.id}
            required={props.required}
            options={props.options.map((t) => ({ label: t.toString(), value: t }))}
            onChange={(value) => {
                props.onChange(value && value.value);
            }}
            autoFocus={false}
            value={props.value && { label: props.value.toString(), value: props.value }}
            styles={selectStyles(props.theme || "default")}
            isClearable={props.isClearable}
        />
    );
}

export function clearSelectStyles<Option, IsMulti extends boolean, Group extends GroupBase<Option>>(): StylesConfig<
    Option,
    IsMulti,
    Group
> {
    return {
        ...selectStyles("default"),
        control: (styles, state) => ({
            ...styles,
            ":hover": {
                boxShadow: "inset 0 -3px 3em var(--primary-op), 0 0 15em #0cf3, 0 0 .5em #0ff2",
            },
            transition: "box-shadow 0.3s ease-in-out",
            backgroundColor: "transparent",
            fontSize: "1.5em",
            boxShadow: "none",
            border: "none",
            // from .sub-heading, adjusted to work without filter:
            fontFamily: "Roboto-Light, sans-serif",
            fontWeight: "normal",
            color: "white",
            textShadow: "0 0 4px var(--primary)",
        }),
    };
}

/**
 * Generates the styling information to be passed to `<Select />` for a given theme
 */
export function selectStyles<Option, IsMulti extends boolean, Group extends GroupBase<Option>>(
    theme: Theme,
): StylesConfig<Option, IsMulti, Group> {
    return {
        control: (styles, state) => ({
            ...styles,
            backgroundColor: "black",
            boxShadow:
                theme === "red"
                    ? "box-shadow: inset 0 -3px 2em var(--red-button), 0 0 10em var(--red-button), 0 0 .5em var(--red-button)"
                    : "inset 0 -3px 2em var(--primary-op), 0 0 10em #0cf3, 0 0 .5em #0ff2",
            borderColor: theme === "red" ? "var(--red-button)" : "var(--primary-op)",
            ":hover": {
                borderColor: theme === "red" ? "rgba(194,69,69,0.5)" : "rgba(103,186,232,0.5)",
            },
        }),
        option: (styles, state) => {
            const hover = state.isSelected
                ? {}
                : {
                      ":hover": {
                          boxShadow:
                              theme === "red"
                                  ? "box-shadow: inset 0 -3px 2em var(--red-button), 0 0 10em var(--red-button), 0 0 .5em var(--red-button)"
                                  : "inset 0 -3px 2em var(--primary-op), 0 0 10em #0cf3, 0 0 .5em #0ff2",
                          backgroundColor: "var(--level-1)",
                          transition:
                              "padding linear 500ms, margin-top linear 500ms, background-color ease-in-out 500ms",
                      },
                  };

            return {
                ...styles,
                backgroundColor: state.isSelected
                    ? theme === "red"
                        ? "var(--red-button)"
                        : "var(--primary-op)"
                    : "none",
                color: "#bbb",
                ...hover,
            };
        },
        menu: (styles) => ({
            ...styles,
            background:
                theme === "red"
                    ? "url(data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAIAAAACCAYAAABytg0kAAAAFUlEQVQIW2P8//+/FCMj4zNGBigAADqJBAO/UCEeAAAAAElFTkSuQmCC), linear-gradient(0deg,#170000,black)"
                    : "url(data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAIAAAACCAYAAABytg0kAAAAFUlEQVQIW2P8//+/FCMj4zNGBigAADqJBAO/UCEeAAAAAElFTkSuQmCC), linear-gradient(0deg,#00263a,#001417)",
        }),
        multiValueLabel: (styles) => ({ ...styles, color: "#bbb" }),
        singleValue: (styles) => ({ ...styles, color: "#bbb" }),
        multiValue: (styles) => ({ ...styles, backgroundColor: "none" }),
        indicatorSeparator: (styles) => ({ ...styles, display: "none" }),
    };
}
