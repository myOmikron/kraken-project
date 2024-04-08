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

export type FilterInputProps = {
    placeholder?: string;
    workspace: string;
    value: string;
    onChange: (newValue: string) => void;
    applied: string;
    onApply: (newApplied: string) => void;
    target: "global" | "domain" | "host" | "port" | "service" | "httpService";
};
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
    addColumn: (column: string, value: string, negate: boolean) => void;
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
        addColumn: (column, value, negate) => {
            onChange((v) => {
                const newFilter = negate ? removeExprs(v, column, value) : addExprs(v, column, value, "and");
                onApply(newFilter);
                return newFilter;
            });
        },
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
