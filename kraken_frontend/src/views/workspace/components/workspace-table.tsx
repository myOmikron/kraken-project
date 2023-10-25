import React, { CSSProperties } from "react";
import { handleApiError } from "../../../utils/helper";
import Select from "react-select";
import { selectStyles } from "../../../components/select-menu";
import { Result } from "../../../utils/result";
import { ApiError } from "../../../api/error";
import Input from "../../../components/input";
import "../../../styling/workspace-table.css";
import ArrowLeftIcon from "../../../svg/arrow-left";
import ArrowRightIcon from "../../../svg/arrow-right";
import ArrowFirstIcon from "../../../svg/arrow-first";
import ArrowLastIcon from "../../../svg/arrow-last";

export type WorkspaceDataTableProps<T> = {
    query: (limit: number, offset: number) => Promise<Result<GenericPage<T>, ApiError>>;
    queryDeps?: React.DependencyList;
    children: [React.ReactNode, (item: T) => React.ReactNode];

    /** The number of columns to render (controls css grid) */
    columns?: number;
    type?: "Host" | "Data";
};
export type GenericPage<T> = {
    items: Array<T>;
    limit: number;
    offset: number;
    total: number;
};

export default function WorkspaceTable<T>(props: WorkspaceDataTableProps<T>) {
    const {
        query,
        queryDeps,
        children: [header, renderItem],
        columns,
        type,
    } = props;

    const [limit, setLimit] = React.useState(20);
    const [offset, setRawOffset] = React.useState(0);
    const [total, setTotal] = React.useState(0);
    const [items, setItems] = React.useState<Array<T>>([]);

    React.useEffect(() => {
        query(limit, offset).then(
            handleApiError(({ items, total }) => {
                setItems(items);
                setTotal(total);
            })
        );
    }, [limit, offset, ...(queryDeps || [])]);

    const lastOffset = Math.floor(total / limit) * limit;
    function setOffset(offset: number) {
        if (offset < 0) {
            setRawOffset(0);
        } else if (offset > lastOffset) {
            setRawOffset(lastOffset);
        } else {
            setRawOffset(offset);
        }
    }

    // @ts-ignore
    const style: CSSProperties = { "--columns": columns };
    return (
        <div className={"workspace-table pane"} style={style}>
            <Input
                className={"input workspace-table-filter"}
                placeholder={"Filter..."}
                value={""}
                onChange={console.log}
            />
            {header}
            <div className={type === "Host" ? "workspace-table-body-host" : "workspace-table-body"}>
                {items.map(renderItem)}
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
                        options={[10, 20, 30, 40, 50, 60, 70, 80, 90, 100].map((n) => ({ value: n, label: String(n) }))}
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
