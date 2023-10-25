import React from "react";
import Select, { StylesConfig } from "react-select";
import { GroupBase } from "react-select/dist/declarations/src/types";

type SelectMenuProps = {
    options: Array<SelectValue>;
    theme?: Theme;
    value: null | SelectValue;
    onChange: (value: SelectValue | null) => void;
    id?: string;
};
type SelectMenuState = {};

type Theme = "default" | "red";

type SelectValue = {
    label: string;
    value: any;
};

export default class SelectMenu extends React.Component<SelectMenuProps, SelectMenuState> {
    constructor(props: SelectMenuProps) {
        super(props);

        this.state = {
            type: null,
        };
    }

    render() {
        return (
            <Select<SelectValue>
                id={this.props.id}
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
        singleValue: (styles) => ({ ...styles, color: "#bbb" }),
        indicatorSeparator: (styles) => ({ ...styles, display: "none" }),
    };
}
