type InformationIconProps = {};

export default function InformationIcon(props: InformationIconProps) {
    return (
        <div className={"icon"} {...props}>
            <svg
                className="neon"
                fill="none"
                stroke="#000"
                strokeLinecap="round"
                width="800px"
                height="800px"
                strokeWidth="40px"
                viewBox="0 0 512 512"
                xmlns="http://www.w3.org/2000/svg"
            >
                <polyline points="196 220 260 220 260 392" strokeLinejoin="round" />
                <line x1="187" y1="396" x2="325" y2="396" strokeMiterlimit="10" />
                <path d="M256,160a32,32,0,1,1,32-32A32,32,0,0,1,256,160Z" />
            </svg>
        </div>
    );
}
