import React from "react";
import { ManualServiceCertainty, PortProtocol } from "../../../api/generated";
import { Api } from "../../../api/api";
import { handleApiError } from "../../../utils/helper";
import { toast } from "react-toastify";
import Input from "../../../components/input";
import Select from "react-select";
import { selectStyles } from "../../../components/select-menu";
import { WORKSPACE_CONTEXT } from "../workspace";

type CreateServiceFormProps = {
    onSubmit: () => void;
};

export function CreateServiceForm(props: CreateServiceFormProps) {
    const { onSubmit } = props;
    const {
        workspace: { uuid: workspace },
    } = React.useContext(WORKSPACE_CONTEXT);
    const [name, setName] = React.useState("");
    const [ip, setIp] = React.useState("");
    const [port, setPort] = React.useState("");
    const [protocol, setProtocol] = React.useState<PortProtocol | null>(null);
    const [certy, setCerty] = React.useState<ManualServiceCertainty>("SupposedTo");
    return (
        <form
            className={"pane workspace-data-create-form"}
            onSubmit={(event) => {
                event.preventDefault();
                if (port.length > 0) {
                    const parsedPort = Number(port);
                    if (Number.isNaN(parsedPort) || parsedPort <= 0 || 65535 < parsedPort) {
                        toast.error("Port must be a number between 1 and 65535");
                        return;
                    }
                }
                Api.workspaces.services
                    .create(workspace, {
                        name,
                        host: ip,
                        port: port.length === 0 ? undefined : Number(port),
                        protocol: protocol === null ? undefined : protocol,
                        certainty: certy,
                    })
                    .then(
                        handleApiError(() => {
                            toast.success("Added service");
                            onSubmit();
                        })
                    );
            }}
        >
            <h2 className={"sub-heading"}>Manually add a service</h2>
            <label>
                Name:
                <Input value={name} onChange={setName} required />
            </label>
            <label>
                Address:
                <Input value={ip} onChange={setIp} required />
            </label>
            <label>
                Port:
                <Input value={port} onChange={setPort} />
            </label>
            <label>
                Protocol:
                <Select<{ value: PortProtocol | null; label: PortProtocol | null }>
                    isClearable={true}
                    styles={selectStyles("default")}
                    options={Object.values(PortProtocol).map((value) => ({ value, label: value }))}
                    value={{ value: protocol, label: protocol }}
                    onChange={(v) => setProtocol(v == null ? v : v.value)}
                />
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
