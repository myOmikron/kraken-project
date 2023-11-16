import React from "react";

export type BubbleProps = {
    name: string;
    color?: "red" | "primary";
} & React.HTMLAttributes<HTMLDivElement>;

export default function Bubble(props: BubbleProps) {
    const { name, color, ...passThrough } = props;
    return (
        <div className={color !== undefined ? `bubble bubble-${color}` : "bubble"} {...passThrough}>
            {name}
        </div>
    );
}
