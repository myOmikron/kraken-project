import React from "react";
import "../styling/attacks.css";
import { Api } from "../api/api";
import { check, handleApiError } from "../utils/helper";
import { toast } from "react-toastify";
import Input from "../components/input";
import Checkbox from "../components/checkbox";
import EditableList from "../components/editable-list";
import Select from "react-select";
import USER_CONTEXT from "../context/user";
import { GetLeech, GetWorkspace } from "../api/generated";

type AttacksProps = {};
type AttacksState = {};

export default class Attacks extends React.Component<AttacksProps, AttacksState> {
    state: AttacksState = {};

    startTcpPortScan(form: TcpPortScanFormValues) {
        const { skipIcmpCheck, targets, exclude } = form;
        const retryInterval = Number(form.retryInterval);
        const maxRetries = Number(form.maxRetries);
        const timeout = Number(form.timeout);
        const concurrentLimit = Number(form.concurrentLimit);
        const ports = form.ports.map((portOrRange) => {
            const port = Number(portOrRange);
            if (isNaN(port) || port < 0) return portOrRange;
            else return port;
        });
        const leech = form.leech?.value || null;
        const workspace = form.workspace?.value || NaN;

        if (
            !check([
                [leech !== null, "Not implemented"], // TODO
                [!isNaN(workspace), "Missing workspace"],
                [true, "Targets must be valid ips"], // TODO checks
                [true, "Exclude must be valid ips"],
                [true, "Ports must be valid port numbers or ranges"],
                [!isNaN(retryInterval), "Retry Interval must be a number"],
                [!isNaN(maxRetries), "Max Retries must be a number"],
                [!isNaN(timeout), "Timeout must be a number"],
                [!isNaN(concurrentLimit), "Concurrent Limit must be a number"],
            ])
        )
            return;

        Api.attacks
            .scanTcpPorts({
                leechId: leech || 0, // TODO
                targets,
                exclude,
                ports,
                retryInterval,
                maxRetries,
                timeout,
                concurrentLimit,
                skipIcmpCheck,
                workspaceId: 1,
            })
            .then(handleApiError(() => toast.success("The attack has began...")));
    }

    render() {
        return (
            <div className="pane">
                <TcpPortScanForm onSubmit={(form) => this.startTcpPortScan(form)} />
            </div>
        );
    }
}

type TcpPortScanFormValues = {
    leech: { value: number; label: string } | null;
    workspace: { value: number; label: string } | null;
    targets: Array<string>;
    exclude: Array<string>;
    ports: Array<string>;
    retryInterval: string;
    maxRetries: string;
    timeout: string;
    concurrentLimit: string;
    skipIcmpCheck: boolean;
};
type TcpPortScanFormProps = {
    onSubmit: (form: TcpPortScanFormValues) => any;
};
function TcpPortScanForm(props: TcpPortScanFormProps) {
    const { onSubmit } = props;

    const [leech, setLeech] = React.useState<{ value: number; label: string } | null>(null);
    const [workspace, setWorkspace] = React.useState<{ value: number; label: string } | null>(null);
    const [targets, setTargets] = React.useState<Array<string>>([]);
    const [exclude, setExclude] = React.useState<Array<string>>([]);
    const [ports, setPorts] = React.useState<Array<string>>([]);
    const [retryInterval, setRetryInterval] = React.useState("100");
    const [maxRetries, setMaxRetries] = React.useState("2");
    const [timeout, setTimeout] = React.useState("3000");
    const [concurrentLimit, setConcurrentLimit] = React.useState("5000");
    const [skipIcmpCheck, setSkipIcmpCheck] = React.useState(false);

    const {
        user: { admin },
    } = React.useContext(USER_CONTEXT);
    const workspaces = useWorkspaces(({ id, name }) => ({ value: id, label: name }));
    const leeches = useLeeches(admin, ({ id, name }) => ({ value: id, label: name }));

    return (
        <form
            className="attack-form"
            onSubmit={(e) => {
                e.preventDefault();
                onSubmit({
                    leech,
                    workspace,
                    targets,
                    exclude,
                    ports,
                    retryInterval,
                    maxRetries,
                    timeout,
                    concurrentLimit,
                    skipIcmpCheck,
                });
            }}
        >
            {admin ? (
                <label>
                    <span className="neon">Leech:</span>
                    <Select options={leeches || []} isLoading={leeches === null} onChange={setLeech} value={leech} />
                </label>
            ) : null}
            <label>
                <span className="neon">Workspace:</span>
                <Select
                    options={workspaces || []}
                    isLoading={workspaces === null}
                    onChange={setWorkspace}
                    value={workspace}
                />
            </label>
            <label>
                <span className="neon">Targets:</span>
                <EditableList value={targets} onChange={setTargets} />
            </label>
            <label>
                <span className="neon">Exclude:</span>
                <EditableList value={exclude} onChange={setExclude} />
            </label>
            <label>
                <span className="neon">Ports:</span>
                <EditableList value={ports} onChange={setPorts} />
            </label>
            <label>
                <span className="neon">Retry Interval:</span>
                <Input value={retryInterval} onChange={setRetryInterval} />
            </label>
            <label>
                <span className="neon">Max Retries:</span>
                <Input value={maxRetries} onChange={setMaxRetries} />
            </label>
            <label>
                <span className="neon">Timeout:</span>
                <Input value={timeout} onChange={setTimeout} />
            </label>
            <label>
                <span className="neon">Concurrent Limit:</span>
                <Input value={concurrentLimit} onChange={setConcurrentLimit} />
            </label>
            <label>
                <span className="neon">Skip ICMP Check:</span>
                <Checkbox value={skipIcmpCheck} onChange={setSkipIcmpCheck} />
            </label>
            <button className="button" type="submit">
                Attack!
            </button>
        </form>
    );
}

function useWorkspaces(): Array<GetWorkspace> | null;
function useWorkspaces<T>(map: (workspace: GetWorkspace) => T): Array<T> | null;
function useWorkspaces<T>(map?: (workspace: GetWorkspace) => T): Array<T> | null {
    const [value, setValue] = React.useState<Array<T> | null>(null);
    React.useEffect(() => {
        Api.workspaces
            .all()
            .then(
                handleApiError(({ workspaces }) =>
                    setValue(map === undefined ? (workspaces as Array<T>) : workspaces.map(map))
                )
            );
    }, []);
    return value;
}

function useLeeches(admin: boolean): Array<GetLeech> | null;
function useLeeches<T>(admin: boolean, map: (leech: GetLeech) => T): Array<T> | null;
function useLeeches<T>(admin: boolean, map?: (leech: GetLeech) => T): Array<T> | null {
    const [value, setValue] = React.useState<Array<T> | null>(null);
    React.useEffect(() => {
        if (admin) {
            Api.admin.leeches
                .all()
                .then(
                    handleApiError(({ leeches }) =>
                        setValue(map === undefined ? (leeches as Array<T>) : leeches.map(map))
                    )
                );
        }
    }, []);
    return value;
}
