import React, { ElementType, FC, PropsWithChildren, forwardRef } from "react";

export type TableRowProps<E extends ElementType = "div" | FC> = PropsWithChildren<{
    as?: E;
}> &
    React.ComponentPropsWithoutRef<E>;

/**
 * A div, with added keyboard navigation for up/down selecting previous/next
 * element sibling. Pressing enter simulates a click.
 */
export const TableRow = forwardRef(({ as, children, ...props }: TableRowProps, ref) => {
    const Component = as ?? "div";
    return (
        <Component
            ref={ref as any}
            onKeyDownCapture={(e) => {
                if (typeof ref != "object" || !ref) return;
                let row = ref.current as HTMLElement; // XXX: invalid, but good enough cast
                if (!row) return;

                if (e.key == "ArrowUp") {
                    let prev = row.previousElementSibling;
                    if (prev && "focus" in prev && typeof prev.focus == "function") prev.focus();
                } else if (e.key == "ArrowDown") {
                    let next = row.nextElementSibling;
                    if (next && "focus" in next && typeof next.focus == "function") next.focus();
                } else if (e.key == "Enter") {
                    row.click();
                }
            }}
            {...props}
        >
            {children}
        </Component>
    );
});
export default TableRow;
