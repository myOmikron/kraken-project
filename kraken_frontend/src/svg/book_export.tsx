type BookExportIconProps = {};

export default function BookExportIcon(props: BookExportIconProps) {
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
                <path d="M 256 160 c 16 -63.16 76.43 -95.41 208 -96 a 15.94 15.94 0 0 1 16 16 V 368 a 16 16 0 0 1 -16 16 C 390 384 352 393 336 398 M 32 366.07 V 80 A 15.94 15.94 0 0 1 48 64 C 179.57 64.59 240 96.84 256 160" />
                <line x1="256" y1="160" x2="256" y2="240" />
                <g transform="scale(13 13) translate(2.5 15)" strokeWidth="3">
                    <path d="M4 12C4 16.4183 7.58172 20 12 20C16.4183 20 20 16.4183 20 12" />
                    <path d="M12 14L12 4M12 4L15 7M12 4L9 7" />
                </g>
            </svg>
        </div>
    );
}
