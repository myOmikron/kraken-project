import React, { HTMLProps } from "react";

type ArrowDownIconProps = {
    inverted?: boolean;
} & HTMLProps<HTMLDivElement>;

export default function ArrowDownIcon({ inverted, ...props }: ArrowDownIconProps) {
    return (
        <div className={"icon"} {...props}>
            <svg
                className={inverted !== undefined && inverted ? "inverted neon" : "neon"}
                fill="none"
                height="24"
                strokeWidth="1.5"
                viewBox="0 0 24 24"
                width="24"
                xmlns="http://www.w3.org/2000/svg"
            >
                <path d="M6 9L12 15L18 9" stroke="red" strokeLinecap="round" strokeLinejoin="round" />
            </svg>
        </div>
    );
}
