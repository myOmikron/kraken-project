import React from "react";
import "../index.css";

export default function ArrowLastIcon(props: React.HTMLAttributes<HTMLDivElement>) {
    return (
        <div className={"icon"} {...props}>
            <svg
                className="neon"
                fill="#000000"
                width="800px"
                height="800px"
                viewBox="0 0 1024 1024"
                xmlns="http://www.w3.org/2000/svg"
            >
                <rect x="728" y="256" width="96" height="512" />
                <path d="M319.3 264.8l-61.8 61.8L442.9 512 257.5 697.4l61.8 61.8L566.5 512z" />
            </svg>
        </div>
    );
}
