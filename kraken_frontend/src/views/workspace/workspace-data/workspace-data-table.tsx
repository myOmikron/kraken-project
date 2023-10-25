import React, { CSSProperties } from "react";
import { handleApiError } from "../../../utils/helper";
import Select from "react-select";
import { selectStyles } from "../../../components/select-menu";
import { Result } from "../../../utils/result";
import { ApiError } from "../../../api/error";
import Input from "../../../components/input";

export type WorkspaceDataTableProps<T> = {
    query: (limit: number, offset: number) => Promise<Result<GenericPage<T>, ApiError>>;
    queryDeps?: React.DependencyList;
    children: [React.ReactNode, (item: T) => React.ReactNode];

    /** The number of columns to render (controls css grid) */
    columns?: number;
};
export type GenericPage<T> = {
    items: Array<T>;
    limit: number;
    offset: number;
    total: number;
};

export default function WorkspaceDataTable<T>(props: WorkspaceDataTableProps<T>) {
    const {
        query,
        queryDeps,
        children: [header, renderItem],
        columns,
    } = props;

    const [limit, setLimit] = React.useState(10);
    const [offset, setRawOffset] = React.useState(0);
    const [total, setTotal] = React.useState(0);
    const [items, setItems] = React.useState<Array<T>>([]);

    React.useEffect(() => {
        query(limit, offset).then(
            handleApiError(({ items, total }) => {
                setItems(items);
                setTotal(total);
            }),
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
        <div className={"workspace-data-table pane"} style={style}>
            <Input
                className={"input workspace-data-filter"}
                placeholder={"Filter..."}
                value={""}
                onChange={console.log}
            />
            {header}
            <div className={"workspace-data-table-body"}>{items.map(renderItem)}</div>
            <div className={"workspace-data-table-controls"}>
                <Select<{ label: string; value: number }, false>
                    menuPlacement={"auto"}
                    value={{ value: 10, label: String(limit) }}
                    options={[10, 20, 30, 40, 50].map((n) => ({ value: n, label: String(n) }))}
                    onChange={(value) => {
                        setLimit(value?.value || limit);
                    }}
                    styles={selectStyles("default")}
                />
                <button className={"button"} disabled={offset === 0} onClick={() => setOffset(0)}>
                    First
                </button>
                <button className={"button"} disabled={offset === 0} onClick={() => setOffset(offset - limit)}>
                    Prev
                </button>
                <span>{`${offset + 1} - ${Math.min(total, offset + limit + 1)} of ${total}`}</span>
                <button className={"button"} disabled={offset === lastOffset} onClick={() => setOffset(offset + limit)}>
                    Next
                </button>
                <button className={"button"} disabled={offset === lastOffset} onClick={() => setOffset(lastOffset)}>
                    Last
                </button>
            </div>
        </div>
    );
}
