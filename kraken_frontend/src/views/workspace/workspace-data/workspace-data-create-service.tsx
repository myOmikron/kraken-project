import React from "react";
import { ManualServiceCertainty, PortProtocol } from "../../../api/generated";
import { Api } from "../../../api/api";
import { handleApiError } from "../../../utils/helper";
import { toast } from "react-toastify";
import Input from "../../../components/input";
import Select from "react-select";
import { SelectPrimitive, selectStyles } from "../../../components/select-menu";
import { WORKSPACE_CONTEXT } from "../workspace";
import Checkbox from "../../../components/checkbox";

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
    const [raw, setRaw] = React.useState(false);
    const [tls, setTls] = React.useState(false);
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
                if ((port.length === 0) !== (protocol === null)) {
                    toast.error("Either specify both port and protocol or neither");
                    return;
                }
                Api.workspaces.services
                    .create(workspace, {
                        name,
                        host: ip,
                        port: port.length === 0 ? undefined : Number(port),
                        protocols: (() => {
                            switch (protocol) {
                                case null:
                                    return undefined;
                                case "Tcp":
                                    return { tcp: { raw, tls } };
                                case "Udp":
                                    return { udp: { raw: true } };
                                case "Sctp":
                                    return { sctp: { raw: true } };
                                case "Unknown":
                                    return { unknown: {} };
                            }
                        })(),
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
                <SelectPrimitive
                    isClearable
                    options={Object.values(PortProtocol)}
                    value={protocol}
                    onChange={(value) => setProtocol(value)}
                />
            </label>
            <label>
                Certainty:
                <SelectPrimitive
                    options={Object.values(ManualServiceCertainty)}
                    value={certy}
                    onChange={(value) => setCerty(value || certy)}
                />
            </label>
            {protocol == PortProtocol.Tcp ? (
                <div>
                    <label>
                        Raw: <Checkbox value={raw} onChange={setRaw} />
                    </label>
                    <label>
                        TLS: <Checkbox value={tls} onChange={setTls} />
                    </label>
                </div>
            ) : null}
            <button className={"button"} type={"submit"}>
                Add
            </button>
        </form>
    );
}
