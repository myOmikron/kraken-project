import { AttackCategory, AttackType } from "../views/workspace/workspace-attacks";

export type AttacksParams = {
    activeAttackCategory: AttackCategory | null;
    activeAttack: AttackType | null;
    onAttackHover: (attack_type: AttackType | null) => void;
    onAttackSelect: (attack_type: AttackType) => void;
    disabled: Partial<Record<AttackType, boolean>>;
};

export default function AttacksIcon(params: AttacksParams) {
    const { activeAttackCategory, onAttackHover, onAttackSelect, activeAttack, disabled } = params;

    const mouseHandler = (attackType: AttackType) =>
        disabled[attackType] || false
            ? {}
            : {
                  onClick: () => onAttackSelect(attackType),
                  onMouseEnter: () => onAttackHover(attackType),
                  onMouseLeave: () => onAttackHover(null),
              };

    return (
        <svg
            xmlns="http://www.w3.org/2000/svg"
            xmlSpace="preserve"
            width="200mm"
            height="182mm"
            viewBox="0 0 200 182"
            className={"kraken-attacks"}
        >
            {/* kraken box */}
            <path
                d="m81.966 81.46-40.05 23.324L1.694 81.763l-.175-46.346 40.049-23.324L81.79 35.114Z"
                className={"kraken-attacks-hex"}
                transform="matrix(.46387 0 0 .46387 82.767 64.555)"
            />
            {/* kraken logo */}
            <path
                d="M64.266 92.241s.593 3.587 6.382 7.804c0 0 1.322 12.712 7.048 12.838 0 0 1.51-.063 1.322-1.385-.16-1.123-.503-2.769.629-3.272 1.519-.675 4.216-.063 4.279 2.517.063 2.58 0 6.167-5.16 7.111 0 0-4.594.629-6.923-2.643-2.329-3.272-6.697-13.656-7.578-15.293-.881 1.636-5.25 12.02-7.578 15.293-2.328 3.273-6.923 2.643-6.923 2.643-5.16-.944-5.223-4.531-5.16-7.111.063-2.58 2.76-3.193 4.279-2.517 1.133.503.79 2.149.629 3.272-.189 1.322 1.322 1.385 1.322 1.385 5.727-.126 7.048-12.838 7.048-12.838 5.792-4.217 6.384-7.804 6.384-7.804zm9.041-70.44-.101-3.707c-4.6-5.16-8.952-7.997-8.952-7.997s-19.572 12.76-19.572 33.371c0 15.379 6.964 21.183 6.964 21.183s1.451 2.902-2.176 4.933c0 0 .58 4.498 2.902 7.254 0 0-1.161 1.886-3.917 1.596-2.757-.29-10.592-2.176-9.866-11.027.726-8.851 3.192-13.929 2.321-20.893S36.121 32.44 29.883 32.73c-6.239.29-10.31 5.039-10.446 9.141-.145 4.353 3.047 7.69 6.384 7.69s4.208-1.741 4.062-2.757c-.145-1.016-.435-1.886-2.467-2.031-2.032-.145-2.902-.871-2.902-2.467s1.016-3.627 4.643-3.482c3.627.145 5.949 6.819 5.513 10.011-.435 3.192-3.627 14.654-3.047 18.716.58 4.063 1.596 13.348 14.074 16.685 0 0 6.529 1.596 8.996-1.161 0 0-1.016 2.902.58 5.368 0 0 .913 5.208-5.513 7.545-4.788 1.741-12.913 2.321-13.783-2.321 0 0-.419-4.597 2.612-4.063 2.467.435 3.047 2.467 2.176 4.498 0 0 2.612 2.612 5.368.145 0 0 1.628-9.192-7.98-9.576-7.254-.29-7.69 7.254-7.69 8.56 0 1.306-.145 8.27 12.333 9.286s18.267-8.27 21.458-14.074c3.192 5.804 8.981 15.089 21.458 14.074 12.477-1.015 12.333-7.98 12.333-9.286 0-1.306-.435-8.85-7.69-8.56-9.608.384-7.98 9.576-7.98 9.576 2.757 2.467 5.368-.145 5.368-.145-.871-2.031-.29-4.063 2.176-4.498 3.031-.535 2.612 4.063 2.612 4.063-.871 4.643-8.996 4.062-13.783 2.321-6.426-2.337-5.513-7.545-5.513-7.545 1.596-2.467.58-5.368.58-5.368 2.467 2.757 8.996 1.161 8.996 1.161C95.288 80.9 96.304 71.614 96.884 67.552c.58-4.062-2.612-15.525-3.047-18.716-.435-3.192 1.886-9.866 5.513-10.011 3.627-.145 4.643 1.886 4.643 3.482 0 1.596-.871 2.321-2.902 2.467-2.031.146-2.321 1.016-2.467 2.031-.145 1.016.725 2.757 4.062 2.757s6.529-3.337 6.384-7.69c-.137-4.101-4.208-8.85-10.446-9.141-6.239-.29-10.156 6.819-11.027 13.783-.871 6.964 1.596 12.042 2.321 20.893.725 8.85-7.109 10.737-9.866 11.027-2.757.29-3.917-1.596-3.917-1.596 2.321-2.757 2.902-7.254 2.902-7.254-3.627-2.031-2.176-4.933-2.176-4.933s6.964-5.804 6.964-21.183c0-8.702-3.489-16.004-7.521-21.538zM60.968 73.865l-1.888 3.021s-4.909-2.895-4.657-5.286c.252-2.391 1.007-2.266 1.385-2.266.378 0 5.16 4.531 5.16 4.531zm12.199-4.531c.378 0 1.133-.126 1.385 2.266.252 2.391-4.657 5.286-4.657 5.286l-1.888-3.021s4.782-4.531 5.16-4.531zM19.621 91.109c-.378-4.405 6.356-9.566 7.174-10.321.818-.755.532-1.569.126-2.266-.441-.755-1.762-1.007-3.021-.755-1.441.288-10.069 4.909-9.377 14.223.692 9.314 8.685 14.537 15.607 14.412 6.923-.126 8.37-2.266 8.37-2.266-2.014-.441-5.853-2.958-5.853-2.958-8.306.377-12.649-5.664-13.026-10.069zm88.758 0c.378-4.405-6.356-9.566-7.174-10.321-.818-.755-.532-1.569-.126-2.266.441-.755 1.762-1.007 3.021-.755 1.441.288 10.069 4.909 9.377 14.223-.692 9.314-8.685 14.537-15.607 14.412-6.923-.126-9.086-1.825-9.086-1.825 2.014-.441 6.545-2.769 6.545-2.769 8.307.377 12.673-6.294 13.05-10.699z"
                style={{
                    fill: "#fff",
                }}
                className={"neon"}
                transform="matrix(.25636 0 0 .25636 85.723 75.256)"
            />
            {/* services box */}
            <path
                d="m81.966 81.46-40.05 23.324L1.694 81.763l-.175-46.346 40.049-23.324L81.79 35.114Z"
                className={`kraken-attacks-hex ${
                    activeAttackCategory === "services" ? "kraken-attacks-hex-box-selected" : ""
                } ${disabled.service_detection ? "kraken-attacks-hex-unavailable" : ""}`}
                transform="matrix(.46387 0 0 .46387 103.637 28.54)"
            />
            {/* service detection */}
            <path
                d="m81.966 81.46-40.05 23.324L1.694 81.763l-.175-46.346 40.049-23.324L81.79 35.114Z"
                {...mouseHandler(AttackType.ServiceDetection)}
                className={`kraken-attacks-hex ${
                    activeAttack === AttackType.ServiceDetection ? "kraken-attacks-hex-selected" : ""
                } ${
                    !disabled[AttackType.ServiceDetection]
                        ? "kraken-attacks-clickable"
                        : "kraken-attacks-hex-unavailable"
                }`}
                transform="matrix(.23193 0 0 .23193 129.444 15.441)"
            />
            {/* UDP service detection */}
            <path
                d="m81.966 81.46-40.05 23.324L1.694 81.763l-.175-46.346 40.049-23.324L81.79 35.114Z"
                {...mouseHandler(AttackType.UdpServiceDetection)}
                className={`kraken-attacks-hex ${
                    activeAttack === AttackType.UdpServiceDetection ? "kraken-attacks-hex-selected" : ""
                } ${
                    !disabled[AttackType.UdpServiceDetection]
                        ? "kraken-attacks-clickable"
                        : "kraken-attacks-hex-unavailable"
                }`}
                transform="matrix(.23193 0 0 .23193 139.54 -1.983)"
            />
            {/* services 3 */}
            <path
                d="m81.966 81.46-40.05 23.324L1.694 81.763l-.175-46.346 40.049-23.324L81.79 35.114Z"
                className={"kraken-attacks-hex-unavailable kraken-attacks-hex"}
                transform="matrix(.23193 0 0 .23193 159.442 -2.096)"
            />
            {/* services 4 */}
            <path
                d="m81.966 81.46-40.05 23.324L1.694 81.763l-.175-46.346 40.049-23.324L81.79 35.114Z"
                className={"kraken-attacks-hex-unavailable kraken-attacks-hex"}
                transform="matrix(.23193 0 0 .23193 179.499 -2.537)"
            />
            {/* domains box */}
            <path
                d="m81.966 81.46-40.05 23.324L1.694 81.763l-.175-46.346 40.049-23.324L81.79 35.114Z"
                className={`kraken-attacks-hex ${
                    activeAttackCategory === "domains" ? "kraken-attacks-hex-box-selected" : ""
                } ${
                    disabled.certificate_transparency && disabled.bruteforce_subdomains && disabled.dns_resolution
                        ? "kraken-attacks-hex-unavailable"
                        : ""
                }`}
                transform="matrix(.46387 0 0 .46387 62.247 99.997)"
            />
            {/* bruteforce subdomains */}
            <path
                d="m81.966 81.46-40.05 23.324L1.694 81.763l-.175-46.346 40.049-23.324L81.79 35.114Z"
                {...mouseHandler(AttackType.BruteforceSubdomains)}
                className={`kraken-attacks-hex ${
                    activeAttack === AttackType.BruteforceSubdomains ? "kraken-attacks-hex-selected" : ""
                } ${
                    !disabled[AttackType.BruteforceSubdomains]
                        ? "kraken-attacks-clickable"
                        : "kraken-attacks-hex-unavailable"
                }`}
                transform="matrix(.23193 0 0 .23193 56.23 139.609)"
            />
            {/* certificate transparency */}
            <path
                d="m81.966 81.46-40.05 23.324L1.694 81.763l-.175-46.346 40.049-23.324L81.79 35.114Z"
                {...mouseHandler(AttackType.CertificateTransparency)}
                className={`kraken-attacks-hex ${
                    activeAttack === AttackType.CertificateTransparency ? "kraken-attacks-hex-selected" : ""
                } ${
                    !disabled[AttackType.CertificateTransparency]
                        ? "kraken-attacks-clickable"
                        : "kraken-attacks-hex-unavailable"
                }`}
                transform="matrix(.23193 0 0 .23193 45.911 157.022)"
            />
            {/* dns resolution */}
            <path
                d="m81.966 81.46-40.05 23.324L1.694 81.763l-.175-46.346 40.049-23.324L81.79 35.114Z"
                {...mouseHandler(AttackType.DnsResolution)}
                className={`kraken-attacks-hex ${
                    activeAttack === AttackType.DnsResolution ? "kraken-attacks-hex-selected" : ""
                } ${
                    !disabled[AttackType.DnsResolution] ? "kraken-attacks-clickable" : "kraken-attacks-hex-unavailable"
                }`}
                transform="matrix(.23193 0 0 .23193 25.876 157.213)"
            />
            {/* hosts box */}
            <path
                d="m81.966 81.46-40.05 23.324L1.694 81.763l-.175-46.346 40.049-23.324L81.79 35.114Z"
                className={`kraken-attacks-hex ${
                    activeAttackCategory === "hosts" ? "kraken-attacks-hex-box-selected" : ""
                } ${disabled.host_alive && disabled.whois ? "kraken-attacks-hex-unavailable" : ""}`}
                transform="matrix(.46387 0 0 .46387 41.129 64.555)"
            />
            {/* whois */}
            <path
                d="m81.966 81.46-40.05 23.324L1.694 81.763l-.175-46.346 40.049-23.324L81.79 35.114Z"
                {...mouseHandler(AttackType.Whois)}
                className={`kraken-attacks-hex ${
                    activeAttack === AttackType.Whois ? "kraken-attacks-hex-selected" : ""
                } ${!disabled[AttackType.Whois] ? "kraken-attacks-clickable" : "kraken-attacks-hex-unavailable"}`}
                transform="matrix(.23193 0 0 .23193 19.926 77.996)"
            />
            {/* host alive */}
            <path
                d="m81.966 81.46-40.05 23.324L1.694 81.763l-.175-46.346 40.049-23.324L81.79 35.114Z"
                {...mouseHandler(AttackType.HostAlive)}
                className={`kraken-attacks-hex ${
                    activeAttack === AttackType.HostAlive ? "kraken-attacks-hex-selected" : ""
                } ${!disabled[AttackType.HostAlive] ? "kraken-attacks-clickable" : "kraken-attacks-hex-unavailable"}`}
                transform="matrix(.23193 0 0 .23193 -.12 77.808)"
            />
            {/* ports box */}
            <path
                d="m81.966 81.46-40.05 23.324L1.694 81.763l-.175-46.346 40.049-23.324L81.79 35.114Z"
                className={`kraken-attacks-hex ${
                    activeAttackCategory === "ports" ? "kraken-attacks-hex-box-selected" : ""
                } ${disabled.tcp_con ? "kraken-attacks-hex-unavailable" : ""}`}
                transform="matrix(.46387 0 0 .46387 61.48 28.148)"
            />
            {/* tcp port scan */}
            <path
                d="m81.966 81.46-40.05 23.324L1.694 81.763l-.175-46.346 40.049-23.324L81.79 35.114Z"
                {...mouseHandler(AttackType.TcpCon)}
                className={`kraken-attacks-hex ${
                    activeAttack === AttackType.TcpCon ? "kraken-attacks-hex-selected" : ""
                } ${!disabled[AttackType.TcpCon] ? "kraken-attacks-clickable" : "kraken-attacks-hex-unavailable"}`}
                transform="matrix(.23193 0 0 .23193 55.05 14.824)"
            />
            {/* testssl */}
            <path
                d="m81.966 81.46-40.05 23.324L1.694 81.763l-.175-46.346 40.049-23.324L81.79 35.114Z"
                {...mouseHandler(AttackType.TestSSL)}
                className={`kraken-attacks-hex ${
                    activeAttack === AttackType.TestSSL ? "kraken-attacks-hex-selected" : ""
                } ${!disabled[AttackType.TestSSL] ? "kraken-attacks-clickable" : "kraken-attacks-hex-unavailable"}`}
                transform="matrix(.23193 0 0 .23193 44.887 -2.266)"
            />
            {/* ports 3 */}
            <path
                d="m81.966 81.46-40.05 23.324L1.694 81.763l-.175-46.346 40.049-23.324L81.79 35.114Z"
                className={"kraken-attacks-hex-unavailable kraken-attacks-hex"}
                transform="matrix(.23193 0 0 .23193 24.863 -2.376)"
            />
            {/* others box */}
            <path
                d="m81.966 81.46-40.05 23.324L1.694 81.763l-.175-46.346 40.049-23.324L81.79 35.114Z"
                className={`kraken-attacks-hex ${
                    activeAttackCategory === "other" ? "kraken-attacks-hex-box-selected" : ""
                } ${disabled.dehashed ? "kraken-attacks-hex-unavailable" : ""}`}
                transform="matrix(.46387 0 0 .46387 124.843 64.555)"
            />
            {/* dehashed */}
            <path
                d="m81.966 81.46-40.05 23.324L1.694 81.763l-.175-46.346 40.049-23.324L81.79 35.114Z"
                {...mouseHandler(AttackType.Dehashed)}
                className={`kraken-attacks-hex ${
                    activeAttack === AttackType.Dehashed ? "kraken-attacks-hex-selected" : ""
                } ${!disabled[AttackType.Dehashed] ? "kraken-attacks-clickable" : "kraken-attacks-hex-unavailable"}`}
                transform="matrix(.23193 0 0 .23193 165.876 77.747)"
            />
            {/* unused box */}
            <path
                d="m81.966 81.46-40.05 23.324L1.694 81.763l-.175-46.346 40.049-23.324L81.79 35.114Z"
                className={"kraken-attacks-hex kraken-attacks-hex-unavailable"}
                transform="matrix(.46387 0 0 .46387 104.294 100.346)"
            />
            <text xmlSpace="preserve" x={71.5} y={58} className={"kraken-attacks-hex-box-text"}>
                {"Ports"}
            </text>
            <text xmlSpace="preserve" x={66} y={130} className={"kraken-attacks-hex-box-text"}>
                {"Domains"}
            </text>
            <text xmlSpace="preserve" x={108.5} y={58} className={"kraken-attacks-hex-box-text"}>
                {"Services"}
            </text>
            <text xmlSpace="preserve" x={135} y={94} className={"kraken-attacks-hex-box-text"}>
                {"Other"}
            </text>
            <text xmlSpace="preserve" x={49.5} y={94} className={"kraken-attacks-hex-box-text"}>
                {"Hosts"}
            </text>
            <text xmlSpace="preserve" x={59.5} y={155.5} className={"kraken-attacks-hex-text"}>
                {"BSd"}
            </text>
            <text xmlSpace="preserve" x={51.5} y={173} className={"kraken-attacks-hex-text"}>
                {"CT"}
            </text>
            <text xmlSpace="preserve" x={31} y={173} className={"kraken-attacks-hex-text"}>
                {"DR"}
            </text>
            <text xmlSpace="preserve" x={171.25} y={94} className={"kraken-attacks-hex-text"}>
                {"Dh"}
            </text>
            <text xmlSpace="preserve" x={21.5} y={94} className={"kraken-attacks-hex-text"}>
                {"WHO"}
            </text>
            <text xmlSpace="preserve" x={59} y={31} className={"kraken-attacks-hex-text"}>
                {"PsT"}
            </text>
            <text xmlSpace="preserve" x={133} y={31} className={"kraken-attacks-hex-text"}>
                {"SvD"}
            </text>
            <text xmlSpace="preserve" x={143.5} y={14} className={"kraken-attacks-hex-text"}>
                {"UDP"}
            </text>
            <text xmlSpace="preserve" x={5} y={94} className={"kraken-attacks-hex-text"}>
                {"HA"}
            </text>
            <text xmlSpace="preserve" x={50} y={14} className={"kraken-attacks-hex-text"}>
                {"TS"}
            </text>
        </svg>
    );
}
