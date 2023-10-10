import React from "react";

type Color = "red" | "primary";

export type TagProps = {
    name: string;
    color?: Color;
} & Omit<React.TextareaHTMLAttributes<HTMLTextAreaElement>, "name" | "color">;

export default function Tag(props: TagProps) {
    const { name, color } = props;

    return (
        <div
            className={
                color !== null && color === "primary" ? "tag-primary tag" : color === "red" ? "tag-red tag" : "tag"
            }
        >
            {name}
        </div>
    );
}
