import { CSSProperties, EventHandler } from "react";
import { Color } from "../api/generated";

export type TagProps = {
    name: string;
    color?: Color;
    onClick?: EventHandler<React.MouseEvent<HTMLDivElement> | React.KeyboardEvent<HTMLDivElement>>;
};

/** Sample of background color */
const BACKGROUND: Color = {
    r: 5,
    g: 15,
    b: 34,
    a: 255,
};

export default function Tag(props: TagProps) {
    const { name, color } = props;

    let style: CSSProperties = {};
    if (color !== undefined) {
        let { r, g, b, a } = color;
        // @ts-ignore
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
        >
            {name}
        </div>
    );
}
