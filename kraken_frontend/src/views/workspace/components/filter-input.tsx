import Input from "../../../components/input";
import React from "react";
import SettingsIcon from "../../../svg/settings";
import SearchIcon from "../../../svg/search";
import CheckmarkIcon from "../../../svg/checkmark";
import ParserError from "../../../utils/filter-ast/error";
import { tokenize } from "../../../utils/filter-ast/lexer";
import { toast } from "react-toastify";

type FilterInputProps = { placeholder?: string; applyFilter: (filter: string) => void };
export default function FilterInput(props: FilterInputProps) {
    const { placeholder, applyFilter } = props;

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
                            tokenize(newValue);
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
