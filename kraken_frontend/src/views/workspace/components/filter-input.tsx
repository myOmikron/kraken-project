import Input from "../../../components/input";
import React from "react";
import SettingsIcon from "../../../svg/settings";
import SearchIcon from "../../../svg/search";
import CheckmarkIcon from "../../../svg/checkmark";
import ParserError from "../../../utils/filter/error";
import { tokenize } from "../../../utils/filter/lexer";
import { toast } from "react-toastify";
import {
    parseDomainAST,
    parseGlobalAST,
    parseHostAST,
    parsePortAST,
    parseServiceAST,
} from "../../../utils/filter/parser";

export type FilterInputProps = {
    placeholder?: string;
    applyFilter: (filter: string) => void;
    target: "global" | "domain" | "host" | "port" | "service";
};
export default function FilterInput(props: FilterInputProps) {
    const { placeholder, applyFilter, target } = props;

    const inputRef = React.useRef() as React.RefObject<HTMLInputElement>;
    const [changed, setChanged] = React.useState(false);
    const [value, setValue] = React.useState("");

    return (
        <form
            onSubmit={(event) => {
                event.preventDefault();
                applyFilter(value);
                setChanged(false);
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
                onChange={(newValue) => {
                    setValue(newValue);
                    setChanged(true);

                    const input = inputRef.current;
                    if (input !== null)
                        try {
                            switch (target) {
                                case "global":
                                    parseGlobalAST(newValue);
                                    break;
                                case "domain":
                                    parseDomainAST(newValue);
                                    break;
                                case "host":
                                    parseHostAST(newValue);
                                    break;
                                case "port":
                                    parsePortAST(newValue);
                                    break;
                                case "service":
                                    parseServiceAST(newValue);
                                    break;
                                default:
                                    tokenize(newValue);
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
                }}
            />
            {changed ? (
                <button className={"button"} type={"submit"} title={"Apply this filter"}>
                    <SearchIcon />
                </button>
            ) : (
                <CheckmarkIcon title={"Filter is applied"} />
            )}
        </form>
    );
}
