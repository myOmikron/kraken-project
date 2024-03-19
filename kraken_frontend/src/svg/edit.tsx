import React from "react";

export default function EditIcon(props: React.HTMLAttributes<HTMLDivElement>) {
    return (
        <div className={"icon"} {...props}>
            <svg
                className={"neon"}
                width="800px"
                height="800px"
                viewBox="0 0 24 24"
                fill="none"
                xmlns="http://www.w3.org/2000/svg"
            >
                <path
                    d="M18 10L21 7L17 3L14 6M18 10L8 20H4V16L14 6M18 10L14 6"
                    stroke="#000000"
                    strokeWidth="1.5"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                />
            </svg>
        </div>
    );
}
