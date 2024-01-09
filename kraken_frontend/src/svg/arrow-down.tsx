import React from "react";

type ArrowDownIconProps = {
    inverted?: boolean;
};

export default function ArrowDownIcon(props: ArrowDownIconProps) {
    const { inverted } = props;
    return (
        <div className={"icon"} {...props}>
            <svg
                className={inverted !== null && inverted ? "inverted neon" : "neon"}
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
