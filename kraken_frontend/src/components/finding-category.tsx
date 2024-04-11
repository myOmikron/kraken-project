import { EventHandler } from "react";
import { Color } from "../api/generated";

export type FindingCategoryProps = {
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

export default function FindingCategory(props: FindingCategoryProps) {
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
            className={`finding-category ${props.onClick ? "interactive" : ""}`}
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
