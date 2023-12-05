import Input from "../../../components/input";
import React from "react";
import SettingsIcon from "../../../svg/settings";
import SearchIcon from "../../../svg/search";
import CheckmarkIcon from "../../../svg/checkmark";

type FilterInputProps = { placeholder?: string; applyFilter: (filter: string) => void };
export default function FilterInput(props: FilterInputProps) {
    const { placeholder, applyFilter } = props;

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
            <button className={"button"} type={"button"}>
                <SettingsIcon />
            </button>
            <Input
                className={"input"}
                placeholder={placeholder || "Filter..."}
                value={value}
                onChange={(newValue) => {
                    setValue(newValue);
                    setChanged(true);
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
