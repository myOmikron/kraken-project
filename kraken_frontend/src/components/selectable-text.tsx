import React from "react";

type SelectableTextProps = {
    /// What kind of element should be used, defaults to `div` if unset.
    as?: string;
    children: any;
} & React.HTMLAttributes<HTMLElement>;

/**
 * Component that automatically selects the whole content when you double click
 * on it.
 *
 * This is useful for data elements so that the user can quickly double-click
 * select and copy the text without trailing or leading extra whitespace
 * otherwise added by the browser when not accurately selecting.
 *
 * Use sparingly! Don't use this on multi-line text, since the user won't expect
 * this selection behavior on it. Don't use on lists of data where the user
 * would likely want to sometimes select only a single word or part of the list.
 *
 * Do use this inside table cells, especially for things such as domains or IPs.
 * Do not use this for free-form content.
 *
 * Doesn't select everything if the user dragged the mouse to select text with
 * word-select mode (double click).
 *
 * Doesn't select everything if the user held ctrl or shift during the double
 * click.
 */
export default function SelectableText(props: SelectableTextProps) {
    // if mouse has been moved this many pixels from the first click, don't select everything
    const MOVE_THRESHOLD = 5;
    const As = (props.as ?? "div") as any;

    let div = React.useRef(null);
    let location = React.useRef([0, 0]);
    return (
        <As
            {...props}
            ref={div}
            onMouseDown={(e: MouseEvent) => {
                location.current = [e.clientX, e.clientY];
            }}
            onMouseMove={(e: MouseEvent) => {
                let [x1, y1] = location.current;
                let [x2, y2] = [e.clientX, e.clientY];
                let d = (x1 - x2) * (x1 - x2) + (y1 - y2) * (y1 - y2);
                if (d > MOVE_THRESHOLD * MOVE_THRESHOLD) location.current = [NaN, NaN];
            }}
            onDoubleClick={(e: MouseEvent) => {
                const selection = window.getSelection();
                let [x1, y1] = location.current;
                let [x2, y2] = [e.clientX, e.clientY];
                let d = (x1 - x2) * (x1 - x2) + (y1 - y2) * (y1 - y2);
                if (div.current && selection && !e.ctrlKey && !e.shiftKey && d < MOVE_THRESHOLD * MOVE_THRESHOLD) {
                    const range = document.createRange();
                    range.selectNodeContents(div.current);
                    selection.removeAllRanges();
                    selection.addRange(range);
                }
            }}
        >
            {props.children}
        </As>
    );
}
