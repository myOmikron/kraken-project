import React from "react";

type BookIconProps = {};

export default function BookIcon(props: BookIconProps) {
    return (
        <div className={"icon"} {...props}>
            <svg
                className="neon"
                fill="none"
                stroke="#000"
                strokeLinecap="round"
                strokeLinejoin="round"
                width="800px"
                height="800px"
                strokeWidth="32px"
                viewBox="0 0 512 512"
                xmlns="http://www.w3.org/2000/svg"
            >
                <path d="M256,160c16-63.16,76.43-95.41,208-96a15.94,15.94,0,0,1,16,16V368a16,16,0,0,1-16,16c-128,0-177.45,25.81-208,64-30.37-38-80-64-208-64-9.88,0-16-8.05-16-17.93V80A15.94,15.94,0,0,1,48,64C179.57,64.59,240,96.84,256,160Z" />
                <line x1="256" y1="160" x2="256" y2="448" />
            </svg>
        </div>
    );
}
