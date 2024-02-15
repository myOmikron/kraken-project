import { CSSProperties, useCallback, useEffect, useRef, useState } from "react";
import { AttackCategory, AttackType } from "../views/workspace/workspace-attacks";

export type AttacksParams = {
    activeAttackCategory: AttackCategory | null;
    activeAttack: AttackType | null;
    onAttackHover: (attack_type: AttackType | null) => void;
    onAttackSelect: (attack_type: AttackType) => void;
    disabled: Partial<Record<AttackType, boolean>>;
    onClickOutside?: (e: React.MouseEvent<SVGSVGElement>) => any;

    className?: string;
};

type HexCssProperties = CSSProperties & {
    "--cx": number;
    "--cy": number;
};

export default function AttacksIcon(params: AttacksParams) {
    // global mouse effects:
    let svg = useRef<SVGSVGElement>(null);
    useEffect(() => {
        if (svg.current) {
            let root = svg.current;
            let viewBox = root.viewBox.baseVal;
            let cb = function (e: MouseEvent) {
                let rect = root.getBoundingClientRect();
                let xOffset = (viewBox.x / viewBox.width) * rect.width;
                let yOffset = (viewBox.y / viewBox.height) * rect.height;
                root.style.setProperty("--mouse-x", e.clientX - rect.x + xOffset + "");
                root.style.setProperty("--mouse-y", e.clientY - rect.y + yOffset + "");
            };
            document.addEventListener("mousemove", cb);
            return () => {
                document.removeEventListener("mousemove", cb);
            };
        }
    }, [svg]);

    // numbers in hex tiles:
    const width = 11;
    const height = 9;

    let { className, ...hexProps } = params;

    return (
        <svg
            ref={svg}
            xmlns="http://www.w3.org/2000/svg"
            xmlSpace="preserve"
            width={width * 60}
            height={height * 70}
            viewBox={`${(-width * 60) / 2} ${(-height * 70) / 2} ${width * 60} ${height * 70}`}
            className={`kraken-attacks ${params.className ?? ""}`}
            onClick={(e) => {
                if ("tagName" in e.target && (e.target.tagName + "").toUpperCase() === "SVG" && params.onClickOutside)
                    params.onClickOutside(e);
            }}
        >
            {/* kraken box */}
            <Hex {...hexProps} x={0} y={0} scale={2} padding={12}>
                <path
                    d="M64.266 92.241s.593 3.587 6.382 7.804c0 0 1.322 12.712 7.048 12.838 0 0 1.51-.063 1.322-1.385-.16-1.123-.503-2.769.629-3.272 1.519-.675 4.216-.063 4.279 2.517.063 2.58 0 6.167-5.16 7.111 0 0-4.594.629-6.923-2.643-2.329-3.272-6.697-13.656-7.578-15.293-.881 1.636-5.25 12.02-7.578 15.293-2.328 3.273-6.923 2.643-6.923 2.643-5.16-.944-5.223-4.531-5.16-7.111.063-2.58 2.76-3.193 4.279-2.517 1.133.503.79 2.149.629 3.272-.189 1.322 1.322 1.385 1.322 1.385 5.727-.126 7.048-12.838 7.048-12.838 5.792-4.217 6.384-7.804 6.384-7.804zm9.041-70.44-.101-3.707c-4.6-5.16-8.952-7.997-8.952-7.997s-19.572 12.76-19.572 33.371c0 15.379 6.964 21.183 6.964 21.183s1.451 2.902-2.176 4.933c0 0 .58 4.498 2.902 7.254 0 0-1.161 1.886-3.917 1.596-2.757-.29-10.592-2.176-9.866-11.027.726-8.851 3.192-13.929 2.321-20.893S36.121 32.44 29.883 32.73c-6.239.29-10.31 5.039-10.446 9.141-.145 4.353 3.047 7.69 6.384 7.69s4.208-1.741 4.062-2.757c-.145-1.016-.435-1.886-2.467-2.031-2.032-.145-2.902-.871-2.902-2.467s1.016-3.627 4.643-3.482c3.627.145 5.949 6.819 5.513 10.011-.435 3.192-3.627 14.654-3.047 18.716.58 4.063 1.596 13.348 14.074 16.685 0 0 6.529 1.596 8.996-1.161 0 0-1.016 2.902.58 5.368 0 0 .913 5.208-5.513 7.545-4.788 1.741-12.913 2.321-13.783-2.321 0 0-.419-4.597 2.612-4.063 2.467.435 3.047 2.467 2.176 4.498 0 0 2.612 2.612 5.368.145 0 0 1.628-9.192-7.98-9.576-7.254-.29-7.69 7.254-7.69 8.56 0 1.306-.145 8.27 12.333 9.286s18.267-8.27 21.458-14.074c3.192 5.804 8.981 15.089 21.458 14.074 12.477-1.015 12.333-7.98 12.333-9.286 0-1.306-.435-8.85-7.69-8.56-9.608.384-7.98 9.576-7.98 9.576 2.757 2.467 5.368-.145 5.368-.145-.871-2.031-.29-4.063 2.176-4.498 3.031-.535 2.612 4.063 2.612 4.063-.871 4.643-8.996 4.062-13.783 2.321-6.426-2.337-5.513-7.545-5.513-7.545 1.596-2.467.58-5.368.58-5.368 2.467 2.757 8.996 1.161 8.996 1.161C95.288 80.9 96.304 71.614 96.884 67.552c.58-4.062-2.612-15.525-3.047-18.716-.435-3.192 1.886-9.866 5.513-10.011 3.627-.145 4.643 1.886 4.643 3.482 0 1.596-.871 2.321-2.902 2.467-2.031.146-2.321 1.016-2.467 2.031-.145 1.016.725 2.757 4.062 2.757s6.529-3.337 6.384-7.69c-.137-4.101-4.208-8.85-10.446-9.141-6.239-.29-10.156 6.819-11.027 13.783-.871 6.964 1.596 12.042 2.321 20.893.725 8.85-7.109 10.737-9.866 11.027-2.757.29-3.917-1.596-3.917-1.596 2.321-2.757 2.902-7.254 2.902-7.254-3.627-2.031-2.176-4.933-2.176-4.933s6.964-5.804 6.964-21.183c0-8.702-3.489-16.004-7.521-21.538zM60.968 73.865l-1.888 3.021s-4.909-2.895-4.657-5.286c.252-2.391 1.007-2.266 1.385-2.266.378 0 5.16 4.531 5.16 4.531zm12.199-4.531c.378 0 1.133-.126 1.385 2.266.252 2.391-4.657 5.286-4.657 5.286l-1.888-3.021s4.782-4.531 5.16-4.531zM19.621 91.109c-.378-4.405 6.356-9.566 7.174-10.321.818-.755.532-1.569.126-2.266-.441-.755-1.762-1.007-3.021-.755-1.441.288-10.069 4.909-9.377 14.223.692 9.314 8.685 14.537 15.607 14.412 6.923-.126 8.37-2.266 8.37-2.266-2.014-.441-5.853-2.958-5.853-2.958-8.306.377-12.649-5.664-13.026-10.069zm88.758 0c.378-4.405-6.356-9.566-7.174-10.321-.818-.755-.532-1.569-.126-2.266.441-.755 1.762-1.007 3.021-.755 1.441.288 10.069 4.909 9.377 14.223-.692 9.314-8.685 14.537-15.607 14.412-6.923-.126-9.086-1.825-9.086-1.825 2.014-.441 6.545-2.769 6.545-2.769 8.307.377 12.673-6.294 13.05-10.699z"
                    style={{
                        fill: "#fff",
                    }}
                    className={"neon"}
                    transform="scale(0.75) translate(-64, -70)"
                />
            </Hex>
            <Hex
                {...hexProps}
                x={1}
                y={0}
                scale={2}
                padding={12}
                text="Other"
                className="category-text"
                categoryType={AttackCategory.Other}
            />
            <Hex
                {...hexProps}
                x={-1}
                y={0}
                scale={2}
                padding={12}
                text="Hosts"
                className="category-text"
                categoryType={AttackCategory.Hosts}
            />
            <Hex
                {...hexProps}
                x={-1}
                y={-1}
                scale={2}
                padding={12}
                text="Ports"
                className={`category-text kraken-attacks-hex-unavailable`}
                categoryType={AttackCategory.Ports}
            />
            <Hex
                {...hexProps}
                x={0}
                y={-1}
                scale={2}
                padding={12}
                text="Services"
                className="category-text"
                categoryType={AttackCategory.Services}
            />
            <Hex
                {...hexProps}
                x={-1}
                y={1}
                scale={2}
                padding={12}
                text="Domains"
                className="category-text"
                categoryType={AttackCategory.Domains}
            />
            <Hex
                {...hexProps}
                x={0}
                y={1}
                scale={2}
                padding={12}
                className="category-text kraken-attacks-hex-unavailable"
            />
            {/* Domains */}
            <Hex
                {...hexProps}
                offsetX={-22}
                offsetY={34}
                x={-2}
                y={3}
                scale={1}
                attackType={AttackType.BruteforceSubdomains}
                text="BSd"
            />
            <Hex
                {...hexProps}
                offsetX={-22}
                offsetY={34}
                x={-2}
                y={4}
                scale={1}
                attackType={AttackType.CertificateTransparency}
                text="CT"
            />
            <Hex
                {...hexProps}
                offsetX={-22}
                offsetY={34}
                x={-3}
                y={4}
                scale={1}
                attackType={AttackType.DnsResolution}
                text="DR"
            />
            <Hex
                {...hexProps}
                offsetX={-22}
                offsetY={34}
                x={-4}
                y={4}
                scale={1}
                attackType={AttackType.DnsTxtScan}
                text="Txt"
            />
            {/* Ports */}
            <Hex
                {...hexProps}
                offsetX={-22}
                offsetY={-34}
                x={-2}
                y={-3}
                scale={1}
                className="kraken-attacks-hex-unavailable"
            />
            <Hex
                {...hexProps}
                offsetX={-22}
                offsetY={-34}
                x={-2}
                y={-4}
                scale={1}
                className="kraken-attacks-hex-unavailable"
            />
            <Hex
                {...hexProps}
                offsetX={-22}
                offsetY={-34}
                x={-3}
                y={-4}
                scale={1}
                className="kraken-attacks-hex-unavailable"
            />
            {/* Services */}
            <Hex
                {...hexProps}
                offsetX={22}
                offsetY={-34}
                x={1}
                y={-3}
                scale={1}
                attackType={AttackType.ServiceDetection}
                text="SvD"
            />
            <Hex
                {...hexProps}
                offsetX={22}
                offsetY={-34}
                x={2}
                y={-4}
                scale={1}
                attackType={AttackType.UdpServiceDetection}
                text="UDP"
            />
            <Hex
                {...hexProps}
                offsetX={22}
                offsetY={-34}
                x={3}
                y={-4}
                scale={1}
                className="kraken-attacks-hex-unavailable"
            />
            <Hex
                {...hexProps}
                offsetX={22}
                offsetY={-34}
                x={4}
                y={-4}
                scale={1}
                className="kraken-attacks-hex-unavailable"
            />
            {/* Other */}
            <Hex {...hexProps} offsetX={40} x={3} y={0} scale={1} attackType={AttackType.Dehashed} text="Dh" />
            {/* Hosts */}
            <Hex {...hexProps} offsetX={-40} x={-3} y={0} scale={1} attackType={AttackType.OsDetection} text="OS" />
            <Hex {...hexProps} offsetX={-40} x={-4} y={0} scale={1} attackType={AttackType.HostAlive} text="HA" />
        </svg>
    );
}

function Hex(props: {
    x: number;
    y: number;
    offsetX?: number;
    offsetY?: number;
    padding?: number;
    scale?: number;
    className?: string;
    text?: string;
    children?: any;
    attackType?: AttackType;
    categoryType?: AttackCategory;
    activeAttackCategory: AttackCategory | null;
    activeAttack: AttackType | null;
    onAttackHover: (attack_type: AttackType | null) => void;
    onAttackSelect: (attack_type: AttackType) => void;
    disabled: Partial<Record<AttackType, boolean>>;
    onClickOutside?: (e: React.MouseEvent<SVGSVGElement>) => any;
}) {
    // we use useState so react caches the random value for us.
    const [delay] = useState(Math.random() * 0.1);
    const [wasClicked, setWasClicked] = useState(false);

    if (props.attackType && props.activeAttack === props.attackType && !wasClicked) setWasClicked(true);

    const mouseHandler = useCallback(
        (attackType: AttackType) =>
            props.disabled[attackType] || false
                ? {}
                : {
                      onClick: () => props.onAttackSelect(attackType),
                      onMouseEnter: () => props.onAttackHover(attackType),
                      onMouseLeave: () => props.onAttackHover(null),
                  },
        [props.disabled, props.onAttackSelect, props.onAttackHover, props.onAttackHover],
    );

    const padding = props.padding ?? 4;
    const ry = 35 * (props.scale ?? 1);
    const rx = (Math.sqrt(3) * ry) / 2;
    const y1 = ry / 2;
    const dx = 2 * rx + padding;
    const dy = 3 * y1 + padding;
    const rowOffset = props.y % 2 == 0 ? 0 : dx / 2;
    const cx = props.x * dx + rowOffset + (props.offsetX ?? 0);
    const cy = props.y * dy + (props.offsetY ?? 0);
    const posOffset = Math.max(Math.abs(cx), Math.abs(cy)) / 1600;
    return (
        <>
            <path
                d={
                    `M${cx - rx} ${cy - y1}` +
                    `l${rx} ${-y1}` +
                    `l${rx} ${y1}` +
                    `l0 ${ry}` +
                    `l${-rx} ${y1}` +
                    `l${-rx} ${-y1}` +
                    `Z`
                }
                style={
                    {
                        animationDelay: `${posOffset + delay}s`,
                        transformOrigin: `${cx}px ${cy}px`,
                        "--cx": cx,
                        "--cy": cy,
                    } as HexCssProperties
                }
                {...(props.attackType ? mouseHandler(props.attackType) : {})}
                className={`kraken-attacks-hex ${wasClicked ? "was-clicked" : ""} ${props.className ?? ""} ${
                    props.attackType && props.activeAttack === props.attackType ? "kraken-attacks-hex-selected" : ""
                } ${
                    props.activeAttackCategory === props.categoryType ? "kraken-attacks-hex-box-selected" : ""
                } ${props.attackType ? (!props.disabled[props.attackType] ? "kraken-attacks-clickable" : "kraken-attacks-hex-unavailable") : ""}`}
            ></path>
            {(props.children || props.text) && (
                <g
                    style={{
                        animationDelay: `${posOffset + delay + 0.2}s`,
                    }}
                    transform={`translate(${cx}, ${cy})`}
                    className={props.className}
                >
                    {props.text && (
                        <text
                            xmlSpace="preserve"
                            className={"kraken-attacks-hex-text"}
                            textAnchor="middle"
                            dominantBaseline="middle"
                            y={2}
                        >
                            {props.text}
                        </text>
                    )}
                    {props.children}
                </g>
            )}
        </>
    );
}
