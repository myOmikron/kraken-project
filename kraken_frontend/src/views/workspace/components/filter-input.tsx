import React from "react";
import { toast } from "react-toastify";
import { SimpleTag } from "../../../api/generated";
import Input from "../../../components/input";
import CheckmarkIcon from "../../../svg/checkmark";
import SearchIcon from "../../../svg/search";
import SettingsIcon from "../../../svg/settings";
import ParserError from "../../../utils/filter/error";
import { tokenize } from "../../../utils/filter/lexer";
import { addExprs, removeExprs } from "../../../utils/filter/mutate";
import {
    parseDomainAST,
    parseGlobalAST,
    parseHostAST,
    parsePortAST,
    parseServiceAST,
} from "../../../utils/filter/parser";

export type FilterInputProps = {
    placeholder?: string;
    value: string;
    onChange: (newValue: string) => void;
    applied: string;
    onApply: (newApplied: string) => void;
    target: "global" | "domain" | "host" | "port" | "service";
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
                    default:
                        tokenize(value);
                        console.warn(`filter target not implemented: ${target}`);
                        return;
                }
                input.setCustomValidity("");
            } catch (e) {
                if (e instanceof ParserError) {
                    let error = e.data;
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
            <button
                className={"button"}
                type={"button"}
                onClick={() => toast.info("There will be an interactive menu soon™")}
            >
                <SettingsIcon />
            </button>
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

export type FilterOutput = FilterInputProps & {
    addTag: (tag: SimpleTag, negate: boolean) => any;
};
export function useFilter(target: FilterInputProps["target"]): FilterOutput {
    const [value, onChange] = React.useState("");
    const [applied, onApply] = React.useState("");
    return {
        placeholder: {
            global: "Global Filter...",
            domain: "Domain Filter...",
            host: "Host Filter...",
            port: "Port Filter...",
            service: "Service Filter...",
        }[target],
        value,
        onChange,
        applied,
        onApply,
        target,
        addTag: (tag, negate) => {
            onChange((v) => {
                let newFilter = negate ? removeExprs(v, "tag", tag.name) : addExprs(v, "tag", tag.name, "and");
                onApply(newFilter);
                return newFilter;
            });
        },
    };
}
