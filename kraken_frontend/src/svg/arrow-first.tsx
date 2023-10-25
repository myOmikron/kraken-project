import React from "react";
import "../index.css";

export default function ArrowFirstIcon(params: any) {
    return (
        <div className={"pagination-icon"} {...params}>
            <svg
                className="neon"
                width="800px"
                height="800px"
                viewBox="0 0 1024 1024"
                xmlns="http://www.w3.org/2000/svg"
            >
                <g fill="#000000" fillRule="nonzero">
                    <path d="M358.989 657.36h51.154V366.64H358.99zM631.348 656.96l36.118-36.29L559.112 511.8l108.354-108.87-36.118-36.29L486.875 511.8z" />
                </g>
            </svg>
        </div>
    );
}
