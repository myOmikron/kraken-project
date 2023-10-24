import React from "react";
import { handleApiError } from "../../../utils/helper";
import Select from "react-select";
import { selectStyles } from "../../../components/select-menu";
import { Result } from "../../../utils/result";
import { ApiError } from "../../../api/error";

export type WorkspaceDataTableProps<T> = {
    query: (limit: number, offset: number) => Promise<Result<GenericPage<T>, ApiError>>;
    queryDeps?: React.DependencyList;
    children: [React.ReactNode, (item: T) => React.ReactNode];
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
    } = props;

    const [limit, setLimit] = React.useState(10);
    const [page, setRawPage] = React.useState(1);
    const [total, setTotal] = React.useState(0);
    const [items, setItems] = React.useState<Array<T>>([]);

    React.useEffect(() => {
        query(limit, limit * (page - 1)).then(
            handleApiError(({ items, total }) => {
                setItems(items);
                setTotal(total);
            }),
        );
    }, [limit, page, ...(queryDeps || [])]);

    const lastPage = Math.ceil(total / limit) || 1;
    function setPage(page: number) {
        if (page <= 0) {
            setRawPage(1);
        } else if (page > lastPage) {
            setRawPage(lastPage);
        } else {
            setRawPage(page);
        }
    }

    return (
        <>
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
                <button className={"button"} disabled={page === 1} onClick={() => setPage(1)}>
                    First
                </button>
                <button className={"button"} disabled={page === 1} onClick={() => setPage(page - 1)}>
                    Prev
                </button>
                <form
                    onSubmit={(event) => {
                        event.preventDefault();
                        const input = event.currentTarget["page"] as HTMLInputElement;
                        const page = Number(input.value);
                        input.value = "";
                        setPage(page);
                    }}
                >
                    <input
                        className={"input"}
                        title={"Page number"}
                        name={"page"}
                        pattern={"[1-9]\\d*"}
                        placeholder={`Page ${page} of ${lastPage}`}
                    />
                </form>
                <button className={"button"} disabled={page === lastPage} onClick={() => setPage(page + 1)}>
                    Next
                </button>
                <button className={"button"} disabled={page === lastPage} onClick={() => setPage(Infinity)}>
                    Last
                </button>
            </div>
        </>
    );
}
