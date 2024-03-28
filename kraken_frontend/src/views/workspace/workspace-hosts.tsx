import React, { useEffect } from "react";
import Select from "react-select";
import { Api } from "../../api/api";
import { FullHost } from "../../api/generated";
import Input from "../../components/input";
import OsIcon from "../../components/os-icon";
import { selectStyles } from "../../components/select-menu";
import { ROUTES } from "../../routes";
import "../../styling/workspace-hosts.css";
import ArrowFirstIcon from "../../svg/arrow-first";
import ArrowLastIcon from "../../svg/arrow-last";
import ArrowLeftIcon from "../../svg/arrow-left";
import ArrowRightIcon from "../../svg/arrow-right";
import { handleApiError } from "../../utils/helper";
import TagList from "./components/tag-list";
import { WORKSPACE_CONTEXT } from "./workspace";

type WorkspaceHostsProps = {};

export default function WorkspaceHosts(_: WorkspaceHostsProps) {
    const {
        workspace: { uuid: workspace },
    } = React.useContext(WORKSPACE_CONTEXT);

    const [searchTerm, setSearchTerm] = React.useState("");
    const [hosts, setHosts] = React.useState<Array<FullHost>>([]);
    const [total, setTotal] = React.useState<number | null>(null);
    const [limit, setLimit] = React.useState(28);
    const [offset, setOffsetImpl] = React.useState(0);

    function retrieveHosts() {
        setHosts([]);
        setTotal(null);
        return Api.workspaces.hosts.all(workspace, limit, offset).then(
            handleApiError(({ items, total }) => {
                setHosts(items);
                setTotal(total);
            }),
        );
    }

    useEffect(() => {
        retrieveHosts();
    }, [workspace, offset, limit]);

    if (total === null) return <p>Loading...</p>;

    const lastOffset = Math.floor(total / limit) * limit;
    const setOffset = (offset: number) => {
        if (offset < 0) {
            setOffsetImpl(0);
        } else if (offset > lastOffset) {
            setOffsetImpl(lastOffset);
        } else {
            setOffsetImpl(offset);
        }
    };

    return (
        <div className={"workspace-hosts-container"}>
            <div className={"pane workspace-hosts-search-pane"}>
                <Input placeholder={"Search host"} value={searchTerm} onChange={setSearchTerm} />
            </div>

            <div className={"workspace-hosts-list"}>
                {hosts.map((host) => {
                    return (
                        <div
                            key={host.uuid}
                            className={"workspace-hosts-host pane"}
                            onClick={() => {
                                ROUTES.WORKSPACE_SINGLE_HOST.visit({
                                    w_uuid: workspace,
                                    h_uuid: host.uuid,
                                });
                            }}
                        >
                            <OsIcon os={host.osType} />
                            <div className={"workspace-hosts-host-info"}>
                                <h2 className={"sub-heading"}>{host.ipAddr}</h2>
                                <span>{host.comment}</span>
                                <TagList tags={host.tags} />
                            </div>
                        </div>
                    );
                })}
            </div>
            <div className={"workspace-table-controls"}>
                <div className={"workspace-table-controls-button-container"}>
                    <button className={"workspace-table-button"} disabled={offset === 0} onClick={() => setOffset(0)}>
                        <ArrowFirstIcon />
                    </button>
                    <button
                        className={"workspace-table-button"}
                        disabled={offset === 0}
                        onClick={() => setOffset(offset - limit)}
                    >
                        <ArrowLeftIcon />
                    </button>
                    <button
                        className={"workspace-table-button"}
                        disabled={offset === lastOffset}
                        onClick={() => setOffset(offset + limit)}
                    >
                        <ArrowRightIcon />
                    </button>
                    <button
                        className={"workspace-table-button"}
                        disabled={offset === lastOffset}
                        onClick={() => setOffset(lastOffset)}
                    >
                        <ArrowLastIcon />
                    </button>
                </div>
                <div className={"workspace-table-controls-page-container"}>
                    <span>{`${offset + 1} - ${Math.min(total, offset + limit + 1)} of ${total}`}</span>
                    <Select<{ label: string; value: number }, false>
                        menuPlacement={"auto"}
                        value={{ value: limit, label: String(limit) }}
                        options={[10, 20, 30, 40, 50, 60, 70, 80, 90, 100].map((n) => ({
                            value: n,
                            label: String(n),
                        }))}
                        onChange={(value) => {
                            setLimit(value?.value || limit);
                        }}
                        styles={selectStyles("default")}
                    />
                </div>
            </div>
        </div>
    );
}
