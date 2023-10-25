import React from "react";
import "../index.css";

export default function ArrowLeftIcon(params: any) {
    return (
        <div className={"icon"} {...params}>
            <svg
                className={"neon"}
                fill="#000000"
                width="800px"
                height="800px"
                viewBox="0 0 1024 1024"
                xmlns="http://www.w3.org/2000/svg"
            >
                <path d="M604.7 759.2l61.8-61.8L481.1 512l185.4-185.4-61.8-61.8L357.5 512z" />
            </svg>
        </div>
    );
}
