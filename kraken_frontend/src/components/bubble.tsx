import React from "react";

/** React props for [`<Bubble />`]{@link Bubble} */
export type BubbleProps = {
    /** The bubble's name */
    name: string;
    /** The bubble's color */
    color?: "red" | "primary";
} & React.HTMLAttributes<HTMLDivElement>;

/**
 * A small div with a round border and fancy background
 *
 * It can be used for displaying tiny pieces of text in a tag like look.
 *
 * Don't confuse it with `<Tag />` which represent kraken's concept of "tags"
 * which can be attached to aggregated objects for filtering.
 */
export default function Bubble(props: BubbleProps) {
    const { name, color, ...passThrough } = props;
    return (
        <div className={color !== undefined ? `bubble bubble-${color}` : "bubble"} {...passThrough}>
            {name}
        </div>
    );
}
