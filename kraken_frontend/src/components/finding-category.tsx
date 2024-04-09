import { EventHandler } from "react";

export type FindingCategoryProps = {
    name: string;
    onClick?: EventHandler<React.MouseEvent<HTMLDivElement> | React.KeyboardEvent<HTMLDivElement>>;
};

export default function FindingCategory(props: FindingCategoryProps) {
    const { name } = props;

    const style: Record<string, string> = {};

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
