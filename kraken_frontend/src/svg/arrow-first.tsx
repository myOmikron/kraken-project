import React from "react";
import "../index.css";

export default function ArrowFirstIcon(props: React.HTMLAttributes<HTMLDivElement>) {
    return (
        <div className={"icon"} {...props}>
            <svg
                className="neon"
                width="800px"
                height="800px"
                viewBox="0 0 1024 1024"
                xmlns="http://www.w3.org/2000/svg"
            >
                <rect x="228" y="256" width="96" height="512" />
                <path d="M704.7 759.2l61.8-61.8L581.1 512l185.4-185.4-61.8-61.8L457.5 512z" />
            </svg>
        </div>
    );
}
