import React from "react";
import { ManualServiceCertainty } from "../../../api/generated";
import { Api } from "../../../api/api";
import { handleApiError } from "../../../utils/helper";
import { toast } from "react-toastify";
import Input from "../../../components/input";
import Select from "react-select";
import { selectStyles } from "../../../components/select-menu";

type CreateServiceFormProps = {
    workspace: string;
    onSubmit: () => void;
};

export function CreateServiceForm(props: CreateServiceFormProps) {
    const { workspace, onSubmit } = props;
    const [name, setName] = React.useState("");
    const [ip, setIp] = React.useState("");
    const [port, setPort] = React.useState("");
    const [certy, setCerty] = React.useState<ManualServiceCertainty>("SupposedTo");
    return (
        <form
            className={"pane workspace-data-create-form"}
            onSubmit={(event) => {
                event.preventDefault();
                Api.workspaces.services
                    .create(workspace, {
                        name,
                        host: ip,
                        port: port.length === 0 ? undefined : Number(port),
                        certainty: certy,
                    })
                    .then(
                        handleApiError(() => {
                            toast.success("Added service");
                            onSubmit();
                        }),
                    );
            }}
        >
            <h2 className={"sub-heading"}>Manually add a service</h2>
            <label>
                Name:
                <Input value={name} onChange={setName} />
            </label>
            <label>
                Address:
                <Input value={ip} onChange={setIp} />
            </label>
            <label>
                Port:
                <Input value={port} onChange={setPort} type={"number"} />
            </label>
            <label>
                Certainty:
                <Select<{ value: ManualServiceCertainty; label: ManualServiceCertainty }>
                    styles={selectStyles("default")}
                    options={Object.values(ManualServiceCertainty).map((value) => ({ value, label: value }))}
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
