import React from "react";
import { ManualHostCertainty } from "../../../api/generated";
import { Api } from "../../../api/api";
import { handleApiError } from "../../../utils/helper";
import { toast } from "react-toastify";
import Input from "../../../components/input";
import Select from "react-select";
import { selectStyles } from "../../../components/select-menu";
import { WORKSPACE_CONTEXT } from "../workspace";

type CreateHostFormProps = {
    onSubmit: () => void;
};

export function CreateHostForm(props: CreateHostFormProps) {
    const { onSubmit } = props;
    const {
        workspace: { uuid: workspace },
    } = React.useContext(WORKSPACE_CONTEXT);
    const [ip, setIp] = React.useState("");
    const [certy, setCerty] = React.useState<ManualHostCertainty>("SupposedTo");
    return (
        <form
            className={"pane workspace-data-create-form"}
            onSubmit={(event) => {
                event.preventDefault();
                Api.workspaces.hosts.create(workspace, { ipAddr: ip, certainty: certy }).then(
                    handleApiError(() => {
                        toast.success("Added host");
                        onSubmit();
                    })
                );
            }}
        >
            <h2 className={"sub-heading"}>Manually add a host</h2>
            <label>
                Address:
                <Input value={ip} onChange={setIp} required />
            </label>
            <label>
                Certainty:
                <Select<{ value: ManualHostCertainty; label: ManualHostCertainty }>
                    styles={selectStyles("default")}
                    options={Object.values(ManualHostCertainty).map((value) => ({ value, label: value }))}
                    value={{ value: certy, label: certy }}
                    onChange={(newValue) => setCerty(newValue?.value || certy)}
                />
            </label>
            <button className={"button"} type={"submit"}>
                Add
            </button>
        </form>
    );
}
