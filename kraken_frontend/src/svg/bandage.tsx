import React from "react";

type BandageIconProps = {};

export default function BandageIcon(props: BandageIconProps) {
    return (
        <div className={"icon"} {...props}>
            <svg
                className="neon"
                fill="none"
                stroke="#000"
                strokeLinecap={"round"}
                strokeLinejoin={"round"}
                width="800px"
                height="800px"
                strokeWidth="32px"
                viewBox="0 0 512 512"
                xmlns="http://www.w3.org/2000/svg"
            >
                <rect
                    x="-24.43"
                    y="167.88"
                    width="560.87"
                    height="176.25"
                    rx="88.12"
                    ry="88.12"
                    transform="translate(-106.04 256) rotate(-45)"
                />
                <rect
                    x="169.41"
                    y="156.59"
                    width="176"
                    height="196"
                    rx="32"
                    ry="32"
                    transform="translate(255.41 -107.45) rotate(45)"
                />
                <circle cx="256" cy="208" r="12" />
                <circle cx="304" cy="256" r="12" />
                <circle cx="208" cy="256" r="12" />
                <circle cx="256" cy="304" r="12" />
            </svg>
        </div>
    );
}
