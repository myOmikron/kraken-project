import Select, { StylesConfig } from "react-select";
import { GroupBase } from "react-select/dist/declarations/src/types";

/** The different themes supported by [`selectStyles`]{@link selectStyles} */
type Theme = "default" | "red";

/** React props for [`<SelectPrimitive />`]{@link SelectPrimitive} */
export type SelectPrimitiveProps<T extends { toString(): string }> = {
    /** Array of options presented by the select menu */
    options: Array<T>;
    /** The select menu's theme */
    theme?: Theme;
    /** The current selected option */
    value: null | T;
    /** Callback invoked when the user changed the selection */
    onChange: (value: T | null) => void;
    /** Html's `id` attribute */
    id?: string;
    /** Html's `required` attribute */
    required?: boolean;
    /** Is the selected value clearable? */
    isClearable?: boolean;
};

/**
 * A [`<Select />`]{@link Select} which works with primitives (i.e. strings and numbers) instead of objects
 *
 * It actually supports any data with a useful `toString` implementation.
 */
export function SelectPrimitive<T extends { toString(): string }>(props: SelectPrimitiveProps<T>) {
    return (
        <Select<{ /** The option's displayed label */ label: string; /** The option's value */ value: T }>
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
        control: (styles) => ({
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
 * Generates the styling information to be passed to [`<Select />`]{@link Select} for a given theme.
 *
 * @param theme the theme to use
 * @returns styling information to be passed to [`<Select />`]{@link Select}
 */
export function selectStyles<Option, IsMulti extends boolean, Group extends GroupBase<Option>>(
    theme: Theme,
): StylesConfig<Option, IsMulti, Group> {
    /* eslint-disable jsdoc/require-jsdoc */
    return {
        control: (styles) => ({
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
            zIndex: 9999,
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
    /* eslint-enable jsdoc/require-jsdoc */
}
