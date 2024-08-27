import React from "react";
import { toast } from "react-toastify";
import { Api } from "../../../api/api";
import { ManualHttpServiceCertainty, PortProtocol } from "../../../api/generated";
import Checkbox from "../../../components/checkbox";
import Input from "../../../components/input";
import { SelectPrimitive } from "../../../components/select-menu";
import { handleApiError } from "../../../utils/helper";
import { WORKSPACE_CONTEXT } from "../workspace";

/**
 * Props for the <CreateHttpServiceForm> component.
 */
type CreateHttpServiceFormProps = {
    /**
     * Called when the HTTP Service was successfully created.
     */
    onSubmit: () => void;
};

/**
 * Form including all the inputs to manually create a HTTP service data entry.
 *
 * Allows submitting to create in the current workspace.
 */
export function CreateHttpServiceForm(props: CreateHttpServiceFormProps) {
    const { onSubmit } = props;
    const {
        workspace: { uuid: workspace },
    } = React.useContext(WORKSPACE_CONTEXT);
    const [name, setName] = React.useState("");
    const [ip, setIp] = React.useState("");
    const [domain, setDomain] = React.useState("");
    const [version, setVersion] = React.useState("");
    const [basePath, setBasePath] = React.useState("/");
    const [certy, setCerty] = React.useState<ManualHttpServiceCertainty>("SupposedTo");
    const [protocol, setProtocol] = React.useState<PortProtocol>("Tcp");
    const [tls, setTls] = React.useState(true);
    const [port, setPort] = React.useState("443");
    const [sniRequired, setSniRequired] = React.useState(false);
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
                Api.workspaces.httpServices
                    .create(workspace, {
                        name,
                        ipAddr: ip,
                        domain: domain.length > 0 ? domain : undefined,
                        port: parsedPort,
                        basePath,
                        tls,
                        sniRequire: sniRequired,
                        certainty: certy,
                        portProtocol: protocol,
                        version: version.length > 0 ? version : undefined,
                    })
                    .then(
                        handleApiError(() => {
                            toast.success("Added HTTP service");
                            onSubmit();
                        }),
                    );
            }}
        >
            <h2 className={"sub-heading"}>Manually add a HTTP service</h2>
            <label>
                Name:
                <Input value={name} onChange={setName} required />
            </label>
            <label>
                Version:
                <Input value={version} onChange={setVersion} />
            </label>
            <label>
                IP Address:
                <Input value={ip} onChange={setIp} required />
            </label>
            <label>
                Domain:
                <Input value={domain} onChange={setDomain} />
            </label>
            <label>
                TLS:
                <Checkbox
                    value={tls}
                    onChange={(newValue) => {
                        if (newValue && port == "80") setPort("443");
                        if (!newValue && port == "443") setPort("80");
                        setTls(newValue);
                        if (!newValue && sniRequired) setSniRequired(false);
                    }}
                />
            </label>
            <div className="label">
                <label htmlFor="portNumber">Port:</label>
                <div className="flex">
                    <Input id="portNumber" value={port} type="number" onChange={setPort} required min={1} max={65535} />
                    <SelectPrimitive
                        options={Object.values(PortProtocol)}
                        value={protocol}
                        onChange={(value) => setProtocol(value || protocol)}
                    />
                </div>
            </div>
            <label>
                Base Path:
                <Input value={basePath} onChange={setBasePath} />
            </label>
            <label>
                Certainty:
                <SelectPrimitive
                    options={Object.values(ManualHttpServiceCertainty)}
                    value={certy}
                    onChange={(value) => setCerty(value || certy)}
                />
            </label>
            <label>
                SNI Required:
                <Checkbox disabled={!tls} value={sniRequired} onChange={setSniRequired} />
            </label>
            <button className={"button"} type={"submit"}>
                Add
            </button>
        </form>
    );
}
