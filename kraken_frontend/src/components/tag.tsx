import React from "react";
import { Color } from "../api/generated";

/** React props for [`<Tag />`]{@link Tag} */
export type TagProps = {
    /** The tag's name */
    name: string;
    /** The tag's color */
    color?: Color;
    /**
     * Optional event handler for click events
     *
     * Pressing `Enter` while focusing on this element will also trigger this handler.
     */
    onClick?: React.EventHandler<React.MouseEvent<HTMLDivElement> | React.KeyboardEvent<HTMLDivElement>>;
};

/** Sample of background color */
const BACKGROUND: Color = {
    r: 5,
    g: 15,
    b: 34,
    a: 255,
};

/**
 * A small div with a colored background representing a tag
 *
 * "Tag" in this context doesn't refer to the generic concept of tags.
 * Instead, it explicitly refers to the tags kraken associates with aggregated objects.
 *
 * Use `<Bubble />` if you need a component for some generic concept of a tag.
 */
export default function Tag(props: TagProps) {
    const { name, color } = props;

    const style: Record<string, string> = {};
    if (color !== undefined) {
        let { r, g, b } = color;
        const { a } = color;
        style["--color"] = `rgba(${r}, ${g}, ${b}, ${a / 255})`;

        // Apply alpha and blend with background
        r = r * (a / 255) + BACKGROUND.r * (1 - a / 255);
        g = g * (a / 255) + BACKGROUND.g * (1 - a / 255);
        b = b * (a / 255) + BACKGROUND.b * (1 - a / 255);
        // Calculate relative luma (See wikipedia)
        const luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        style.color = luma > 128 ? "black" : "white";
    }

    return (
        <div
            className={`tag ${props.onClick ? "interactive" : ""}`}
            style={style}
            onClick={props.onClick}
            onKeyDown={(e) => {
                if (e.key == "Enter") {
                    props.onClick?.(e);
                    e.preventDefault();
                }
            }}
            tabIndex={0}
            title={name}
        >
            {name}
        </div>
    );
}
