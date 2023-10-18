import React from "react";
import "../index.css";

export default function ArrowUpIcon(params: any) {
    return (
        <div className={"icon"} {...params}>
            <svg
                className={"neon"}
                fill="none"
                height="24"
                strokeWidth="1.5"
                viewBox="0 0 24 24"
                width="24"
                xmlns="http://www.w3.org/2000/svg"
            >
                <path d="M6 15L12 9L18 15" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" />
            </svg>
        </div>
    );
}
