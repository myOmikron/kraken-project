import React from "react";
import "../index.css";

export default function FileIcon(props: React.HTMLAttributes<HTMLDivElement>) {
    return (
        <div className={"icon"} {...props}>
            <svg
                className="neon"
                fill="#000000"
                width="800px"
                height="800px"
                viewBox="0 0 32 32"
                version="1.1"
                xmlns="http://www.w3.org/2000/svg"
            >
                <path d="M26.731 9.902c-0.005-0.035-0.011-0.066-0.019-0.095l0.001 0.005c-0.031-0.134-0.094-0.249-0.182-0.342l0 0-8-8c-0.092-0.087-0.207-0.15-0.335-0.181l-0.005-0.001c-0.027-0.008-0.059-0.014-0.092-0.019l-0.003-0c-0.026-0.007-0.059-0.014-0.092-0.019l-0.004-0h-12c-0.414 0-0.75 0.336-0.75 0.75v0 28c0 0.414 0.336 0.75 0.75 0.75h20c0.414-0 0.75-0.336 0.75-0.75v0-20c-0.005-0.038-0.012-0.071-0.020-0.103l0.001 0.005zM24.189 9.25h-5.439v-5.439zM6.75 29.25v-26.5h10.5v7.25c0 0.414 0.336 0.75 0.75 0.75h7.25v18.5z"></path>
            </svg>
        </div>
    );
}
