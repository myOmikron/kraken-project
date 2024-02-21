import React from "react";
import { ManualPortCertainty, PortProtocol } from "../../../api/generated";
import { Api } from "../../../api/api";
import { handleApiError } from "../../../utils/helper";
import { toast } from "react-toastify";
import Input from "../../../components/input";
import Select from "react-select";
import { selectStyles } from "../../../components/select-menu";
import { WORKSPACE_CONTEXT } from "../workspace";

type CreatePortFormProps = {
    onSubmit: () => void;
};

export function CreatePortForm(props: CreatePortFormProps) {
    const { onSubmit } = props;
    const {
        workspace: { uuid: workspace },
    } = React.useContext(WORKSPACE_CONTEXT);
    const [ip, setIp] = React.useState("");
    const [port, setPort] = React.useState("");
    const [certy, setCerty] = React.useState<ManualPortCertainty>("SupposedTo");
    const [proto, setProto] = React.useState<PortProtocol>("Tcp");
    return (
        <form
            className={"pane workspace-data-create-form"}
            onSubmit={(event) => {
                event.preventDefault();
                const parsedPort = Number(port);
                if (Number.isNaN(parsedPort) || parsedPort <= 0 || 65535 < parsedPort) {
                    toast.error("Port must be a number between 1 and 65535");
                    return;
                }
                Api.workspaces.ports
                    .create(workspace, { ipAddr: ip, port: Number(port), certainty: certy, protocol: proto })
                    .then(
                        handleApiError(() => {
                            toast.success("Added port");
                            onSubmit();
                        })
                    );
            }}
        >
            <h2 className={"sub-heading"}>Manually add a port</h2>
            <label>
                Address:
                <Input value={ip} onChange={setIp} required />
            </label>
            <label>
                Port:
                <Input value={port} onChange={setPort} required />
            </label>
            <label>
                Certainty:
                <Select<{ value: ManualPortCertainty; label: ManualPortCertainty }>
                    styles={selectStyles("default")}
                    options={Object.values(ManualPortCertainty).map((value) => ({ value, label: value }))}
                    value={{ value: certy, label: certy }}
                    onChange={(newValue) => setCerty(newValue?.value || certy)}
                />
            </label>
            <label>
                Protocol:
                <Select<{ value: PortProtocol; label: PortProtocol }>
                    styles={selectStyles("default")}
                    options={Object.values(PortProtocol).map((value) => ({ value, label: value }))}
                    value={{ value: proto, label: proto }}
                    onChange={(newValue) => setProto(newValue?.value || proto)}
                />
            </label>
            <button className={"button"} type={"submit"}>
                Add
            </button>
        </form>
    );
}
