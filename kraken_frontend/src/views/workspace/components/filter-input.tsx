import React from "react";
import Popup from "reactjs-popup";
import Input from "../../../components/input";
import CheckmarkIcon from "../../../svg/checkmark";
import SearchIcon from "../../../svg/search";
import SettingsIcon from "../../../svg/settings";
import ParserError from "../../../utils/filter/error";
import { tokenize } from "../../../utils/filter/lexer";
import { addExprRange, addExprs, removeExprRange, removeExprs } from "../../../utils/filter/mutate";
import {
    parseDomainAST,
    parseGlobalAST,
    parseHostAST,
    parseHttpServiceAST,
    parsePortAST,
    parseServiceAST,
} from "../../../utils/filter/parser";
import { FilterEditorUi } from "./filter-editor-ui";

/**
 * Props for the <FilterInput> component.
 */
export type FilterInputProps = {
    /**
     * Shown in the input when no text is entered yet
     */
    placeholder?: string;
    /**
     * The workspace UUID to search data in.
     */
    workspace: string;
    /**
     * The filter.
     */
    value: string;
    /**
     * Called when the user is typing, to update value.
     */
    onChange: (newValue: string) => void;
    /**
     * The filter that is currently applied to the table.
     */
    applied: string;
    /**
     * Called when the filter should be applied, usually on hitting return.
     */
    onApply: (newApplied: string) => void;
    /**
     * The filter AST to use for the filter text.
     */
    target: "global" | "domain" | "host" | "port" | "service" | "httpService";
};

/**
 * An input field for a filter, has a button to show a GUI popup.
 */
export default function FilterInput(props: FilterInputProps) {
    const { placeholder, value, onChange, applied, onApply, target } = props;

    const inputRef = React.useRef() as React.RefObject<HTMLInputElement>;

    // Run parser each time `value` changes
    React.useEffect(() => {
        const input = inputRef.current;
        if (input !== null)
            try {
                switch (target) {
                    case "global":
                        parseGlobalAST(value);
                        break;
                    case "domain":
                        parseDomainAST(value);
                        break;
                    case "host":
                        parseHostAST(value);
                        break;
                    case "port":
                        parsePortAST(value);
                        break;
                    case "service":
                        parseServiceAST(value);
                        break;
                    case "httpService":
                        parseHttpServiceAST(value);
                        break;
                    default:
                        tokenize(value);
                        // eslint-disable-next-line no-console
                        console.warn(`filter target not implemented: ${target}`);
                        return;
                }
                input.setCustomValidity("");
            } catch (e) {
                if (e instanceof ParserError) {
                    input.setCustomValidity(e.message);
                } else throw e;
            }
    }, [value]);

    return (
        <form
            onSubmit={(event) => {
                event.preventDefault();
                onApply(value);
            }}
        >
            <Popup
                trigger={
                    <button className={"button"} type={"button"}>
                        <SettingsIcon />
                    </button>
                }
                on={"click"}
                position={"right center"}
            >
                <div className="pane-thin popup-filter-editor">
                    <FilterEditorUi
                        workspace={props.workspace}
                        ast={props.target}
                        filter={value}
                        onChange={onChange}
                        onApply={onApply}
                    />
                </div>
            </Popup>
            <Input
                ref={inputRef}
                className={"input"}
                placeholder={placeholder || "Filter..."}
                value={value}
                onChange={onChange}
            />
            {value !== applied ? (
                <button className={"button"} type={"submit"} title={"Apply this filter"}>
                    <SearchIcon />
                </button>
            ) : (
                <CheckmarkIcon title={"Filter is applied"} />
            )}
        </form>
    );
}

/** Return type of {@link useFilter `useFilter`} hook */
export type UseFilterReturn = FilterInputProps & {
    /**
     * Adds a `value` to the given filter `column` using an `and` concatenation.
     * If `negate` is true, removes the matching `value` instead.
     */
    addColumn: (column: string, value: string, negate: boolean) => void;
    /**
     * Adds a range to the given filter `column` using an `and` concatenation.
     * If `negate` is true, removes the matching range instead.
     */
    addRange: (column: string, from: string, to: string, negate: boolean) => void;
};

export function useFilter(workspace: string, target: FilterInputProps["target"]): UseFilterReturn {
    const [value, onChange] = React.useState("");
    const [applied, onApply] = React.useState("");
    return {
        placeholder: {
            global: "Global Filter...",
            domain: "Domain Filter...",
            host: "Host Filter...",
            port: "Port Filter...",
            service: "Service Filter...",
            httpService: "HTTP Service Filter...",
        }[target],
        workspace,
        value,
        onChange,
        applied,
        onApply,
        target,
        // eslint-disable-next-line jsdoc/require-jsdoc
        addColumn: (column, value, negate) => {
            onChange((v) => {
                const newFilter = negate ? removeExprs(v, column, value) : addExprs(v, column, value, "and");
                onApply(newFilter);
                return newFilter;
            });
        },
        // eslint-disable-next-line jsdoc/require-jsdoc
        addRange: (column, from, to, negate) => {
            onChange((v) => {
                const newFilter = negate
                    ? removeExprRange(v, column, from, to)
                    : addExprRange(v, column, from, to, "and");
                onApply(newFilter);
                return newFilter;
            });
        },
    };
}
