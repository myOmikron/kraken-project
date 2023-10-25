import React, { CSSProperties } from "react";
import { Color } from "../api/generated";

export type TagProps = {
    name: string;
    color?: Color;
};

export default function Tag(props: TagProps) {
    const { name, color } = props;

    let style: CSSProperties = {};
    if (color !== undefined) {
        const { r, g, b, a } = color;
        // @ts-ignore
        style["--color"] = `rgba(${r}, ${g}, ${b}, ${a / 255})`;
    }

    return (
        <div className={"tag"} style={style}>
            {name}
        </div>
    );
}
