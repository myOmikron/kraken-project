import React from "react";

export type BubbleProps = {
    name: string;
    color?: "red" | "primary";
};

export default function Bubble(props: BubbleProps) {
    const { name, color } = props;
    return <div className={color !== undefined ? `bubble bubble-${color}` : "bubble"}>{name}</div>;
}
