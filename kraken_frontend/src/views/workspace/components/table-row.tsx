import React, { PropsWithChildren, forwardRef } from "react";

export type TableRowProps = PropsWithChildren<{}> & React.ComponentPropsWithoutRef<"div">;

/**
 * A div, with added keyboard navigation for up/down selecting previous/next
 * element sibling. Pressing enter simulates a click.
 */
export const TableRow = forwardRef<HTMLDivElement, TableRowProps>(({ children, ...props }, ref) => {
    const innerRef = React.useRef<HTMLDivElement | null>(null);
    return (
        <div
            ref={(div) => {
                innerRef.current = div;
                if (ref === null) return;
                else if (typeof ref === "object") ref.current = div;
                else ref(div);
            }}
            onKeyDownCapture={(e) => {
                if (innerRef.current === null) return;

                if (e.key == "ArrowUp") {
                    const prev = innerRef.current.previousElementSibling;
                    if (prev && "focus" in prev && typeof prev.focus == "function") prev.focus();
                } else if (e.key == "ArrowDown") {
                    const next = innerRef.current.nextElementSibling;
                    if (next && "focus" in next && typeof next.focus == "function") next.focus();
                } else if (e.key == "Enter") {
                    innerRef.current.click();
                }
            }}
            {...props}
        >
            {children}
        </div>
    );
});
export default TableRow;
