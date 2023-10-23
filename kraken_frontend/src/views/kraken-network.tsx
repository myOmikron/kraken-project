import React from "react";
import { Api } from "../api/api";
import { toast } from "react-toastify";
import { handleApiError, sleep } from "../utils/helper";
import Input from "../components/input";
import Popup from "reactjs-popup";
import { SimpleLeech } from "../api/generated";
import "../styling/kraken-network.css";

type KrakenNetworkProps = {};
type KrakenNetworkState = {
    leeches?: Array<SimpleLeech>;
    showPopup: boolean;
    name: string;
    address: string;
};

export default class KrakenNetwork extends React.Component<KrakenNetworkProps, KrakenNetworkState> {
    pane: HTMLElement | null | undefined;
    kraken: HTMLElement | null | undefined;
    leftItems: Array<HTMLElement | null | undefined>;
    rightItems: Array<HTMLElement | null | undefined>;
    leftTable: HTMLElement | null | undefined;
    rightTable: HTMLElement | null | undefined;
    leftTop: SVGElement | null | undefined;
    leftMid: SVGElement | null | undefined;
    leftBottom: SVGElement | null | undefined;
    rightTop: SVGElement | null | undefined;
    rightMid: SVGElement | null | undefined;
    rightBottom: SVGElement | null | undefined;
    constructor(props: KrakenNetworkProps) {
        super(props);

        this.state = {
            showPopup: false,
            name: "",
            address: "",
        };

        this.leftItems = [];
        this.rightItems = [];

        this.retrieveLeeches = this.retrieveLeeches.bind(this);
        this.createLeech = this.createLeech.bind(this);

        // Trigger rerender on resize
        window.addEventListener("resize", () => {
            this.setState({});
        });
    }

    componentDidMount() {
        this.retrieveLeeches();
    }

    retrieveLeeches() {
        Api.admin.leeches.all().then(
            handleApiError(async ({ leeches }) => {
                this.setState({ leeches });
                await sleep(10);
                this.setState({});
            }),
        );
    }

    createLeech(e: React.FormEvent<HTMLFormElement>) {
        e.preventDefault();
        const { name, address } = this.state;

        Api.admin.leeches.create({ name, address }).then(
            handleApiError(() => {
                toast.success("Created leech");
                this.setState({ showPopup: false, address: "", name: "" });
                this.retrieveLeeches();
            }),
        );
    }

    render() {
        let leechesLeft = [],
            leechesRight = [];
        if (this.state.leeches) {
            let half = Math.ceil(this.state.leeches.length / 2);
            for (let i = 0; i < this.state.leeches.length; i++) {
                const l = this.state.leeches[i];

                let leech = (
                    <tr
                        ref={(e) => {
                            if (i < half) {
                                this.leftItems.push(e);
                            } else {
                                this.rightItems.push(e);
                            }
                        }}
                    >
                        <th>{l.uuid}</th>
                        <th>{l.name}</th>
                        <th>{l.address}</th>
                        <th>
                            <button
                                className="button"
                                type="button"
                                onClick={async () => {
                                    const result = await Api.admin.leeches.genCert(l.uuid);

                                    let config = "";
                                    result.match(
                                        ({ ca, cert, key, sni }) => {
                                            config = `KrakenSni = "${sni}"\nKrakenCa = """\n${ca}"""\nLeechCert = """\n${cert}"""\nLeechKey="""\n${key}"""`;
                                        },
                                        (err) => {
                                            toast.error(err.message);
                                        },
                                    );
                                    if (config.length == 0) return;

                                    await navigator.clipboard.writeText(config);
                                    toast.success("Copied client tls config to clipboard", { autoClose: 1500 });
                                }}
                            >
                                Gen tls config
                            </button>
                        </th>
                    </tr>
                );

                if (i < half) {
                    leechesLeft.push(leech);
                } else {
                    leechesRight.push(leech);
                }
            }
        }

        const lines = [];
        if (this.pane && this.kraken && this.leftTop && this.rightTop) {
            const leftTopStartLeft =
                this.kraken.offsetLeft +
                this.leftTop.getBoundingClientRect().left -
                this.kraken.getBoundingClientRect().left;
            const rightTopStartLeft =
                this.kraken.offsetLeft +
                this.rightTop.getBoundingClientRect().left -
                this.kraken.getBoundingClientRect().left;
            const leftTopStartTop =
                this.kraken.offsetTop +
                this.leftTop.getBoundingClientRect().top -
                this.kraken.getBoundingClientRect().top;
            const rightTopStartTop = leftTopStartTop;

            for (const item of this.leftItems) {
                if (item && this.leftTable) {
                    let left = item.offsetWidth + item.offsetLeft + this.leftTable.offsetLeft;
                    let top = item.offsetTop + Math.round(item.offsetHeight / 2) + this.leftTable.offsetTop;
                    lines.push(curve(leftTopStartLeft, leftTopStartTop, left, top));
                }
            }
            for (const item of this.rightItems) {
                if (item && this.rightTable) {
                    let left = item.offsetLeft + this.rightTable.offsetLeft;
                    let top = item.offsetTop + Math.round(item.offsetHeight / 2) + this.rightTable.offsetTop;
                    lines.push(curve(rightTopStartLeft, rightTopStartTop, left, top));
                }
            }
        }

        return (
            <>
                <div
                    className="pane kraken-network"
                    ref={(e) => {
                        this.pane = e;
                    }}
                >
                    <h1 className="heading neon">Kraken network</h1>
                    <div className="kraken-network-grid">
                        <div
                            className="kraken-network-left"
                            ref={(e) => {
                                this.leftTable = e;
                            }}
                        >
                            {leechesLeft.length === 0 ? undefined : (
                                <table>
                                    <thead>
                                        <tr>
                                            <th>ID</th>
                                            <th>Name</th>
                                            <th>Address</th>
                                            <th>Token</th>
                                        </tr>
                                    </thead>
                                    <tbody>{leechesLeft}</tbody>
                                </table>
                            )}
                        </div>
                        <div
                            className="kraken-network-mid"
                            ref={(e) => {
                                this.kraken = e;
                            }}
                        >
                            <svg
                                className="neon kraken-svg"
                                viewBox="0 0 128 128"
                                enableBackground="new 0 0 128 128"
                                version="1.1"
                                xmlSpace="preserve"
                                xmlns="http://www.w3.org/2000/svg"
                            >
                                <path d="m64.266 92.241s0.593 3.587 6.382 7.804c0 0 1.322 12.712 7.048 12.838 0 0 1.51-0.063 1.322-1.385-0.16-1.123-0.503-2.769 0.629-3.272 1.519-0.675 4.216-0.063 4.279 2.517s0 6.167-5.16 7.111c0 0-4.594 0.629-6.923-2.643s-6.697-13.656-7.578-15.293c-0.881 1.636-5.25 12.02-7.578 15.293s-6.923 2.643-6.923 2.643c-5.16-0.944-5.223-4.531-5.16-7.111s2.76-3.193 4.279-2.517c1.133 0.503 0.79 2.149 0.629 3.272-0.189 1.322 1.322 1.385 1.322 1.385 5.727-0.126 7.048-12.838 7.048-12.838 5.792-4.217 6.384-7.804 6.384-7.804zm9.041-70.44l-0.101-3.707c-4.6-5.16-8.952-7.997-8.952-7.997s-19.572 12.76-19.572 33.371c0 15.379 6.964 21.183 6.964 21.183s1.451 2.902-2.176 4.933c0 0 0.58 4.498 2.902 7.254 0 0-1.161 1.886-3.917 1.596-2.757-0.29-10.592-2.176-9.866-11.027s3.192-13.929 2.321-20.893-4.789-14.074-11.027-13.784c-6.239 0.29-10.31 5.039-10.446 9.141-0.145 4.353 3.047 7.69 6.384 7.69s4.208-1.741 4.062-2.757c-0.145-1.016-0.435-1.886-2.467-2.031s-2.902-0.871-2.902-2.467 1.016-3.627 4.643-3.482 5.949 6.819 5.513 10.011c-0.435 3.192-3.627 14.654-3.047 18.716 0.58 4.063 1.596 13.348 14.074 16.685 0 0 6.529 1.596 8.996-1.161 0 0-1.016 2.902 0.58 5.368 0 0 0.913 5.208-5.513 7.545-4.788 1.741-12.913 2.321-13.783-2.321 0 0-0.419-4.597 2.612-4.063 2.467 0.435 3.047 2.467 2.176 4.498 0 0 2.612 2.612 5.368 0.145 0 0 1.628-9.192-7.98-9.576-7.254-0.29-7.69 7.254-7.69 8.56s-0.145 8.27 12.333 9.286 18.267-8.27 21.458-14.074c3.192 5.804 8.981 15.089 21.458 14.074s12.333-7.98 12.333-9.286-0.435-8.85-7.69-8.56c-9.608 0.384-7.98 9.576-7.98 9.576 2.757 2.467 5.368-0.145 5.368-0.145-0.871-2.031-0.29-4.063 2.176-4.498 3.031-0.535 2.612 4.063 2.612 4.063-0.871 4.643-8.996 4.062-13.783 2.321-6.426-2.337-5.513-7.545-5.513-7.545 1.596-2.467 0.58-5.368 0.58-5.368 2.467 2.757 8.996 1.161 8.996 1.161 12.477-3.336 13.493-12.622 14.073-16.684s-2.612-15.525-3.047-18.716c-0.435-3.192 1.886-9.866 5.513-10.011s4.643 1.886 4.643 3.482-0.871 2.321-2.902 2.467-2.321 1.016-2.467 2.031c-0.145 1.016 0.725 2.757 4.062 2.757s6.529-3.337 6.384-7.69c-0.137-4.101-4.208-8.85-10.446-9.141-6.239-0.29-10.156 6.819-11.027 13.783s1.596 12.042 2.321 20.893c0.725 8.85-7.109 10.737-9.866 11.027s-3.917-1.596-3.917-1.596c2.321-2.757 2.902-7.254 2.902-7.254-3.627-2.031-2.176-4.933-2.176-4.933s6.964-5.804 6.964-21.183c0-8.702-3.489-16.004-7.521-21.538l-2.997-0.129zm-12.339 52.064l-1.888 3.021s-4.909-2.895-4.657-5.286 1.007-2.266 1.385-2.266 5.16 4.531 5.16 4.531zm12.199-4.531c0.378 0 1.133-0.126 1.385 2.266 0.252 2.391-4.657 5.286-4.657 5.286l-1.888-3.021s4.782-4.531 5.16-4.531zm-53.546 21.775c-0.378-4.405 6.356-9.566 7.174-10.321s0.532-1.569 0.126-2.266c-0.441-0.755-1.762-1.007-3.021-0.755-1.441 0.288-10.069 4.909-9.377 14.223s8.685 14.537 15.607 14.412c6.923-0.126 8.37-2.266 8.37-2.266-2.014-0.441-5.853-2.958-5.853-2.958-8.306 0.377-12.649-5.664-13.026-10.069zm88.758 0c0.378-4.405-6.356-9.566-7.174-10.321s-0.532-1.569-0.126-2.266c0.441-0.755 1.762-1.007 3.021-0.755 1.441 0.288 10.069 4.909 9.377 14.223s-8.685 14.537-15.607 14.412c-6.923-0.126-9.086-1.825-9.086-1.825 2.014-0.441 6.545-2.769 6.545-2.769 8.307 0.377 12.673-6.294 13.05-10.699z" />
                                <circle
                                    cx="28.25573"
                                    cy="46.884533"
                                    r="1"
                                    ref={(e) => {
                                        this.leftTop = e;
                                    }}
                                />
                                <circle
                                    cx="25.380424"
                                    cy="79.233101"
                                    r="1"
                                    ref={(e) => {
                                        this.leftMid = e;
                                    }}
                                />
                                <circle
                                    cx="47.241169"
                                    cy="109.65893"
                                    r="1"
                                    ref={(e) => {
                                        this.leftBottom = e;
                                    }}
                                />
                                <circle
                                    cx="81.291626"
                                    cy="109.67508"
                                    r="1"
                                    ref={(e) => {
                                        this.rightBottom = e;
                                    }}
                                />
                                <circle
                                    cx="102.78103"
                                    cy="79.241173"
                                    r="1"
                                    ref={(e) => {
                                        this.rightMid = e;
                                    }}
                                />
                                <circle
                                    cx="100.81873"
                                    cy="46.898804"
                                    r="1"
                                    ref={(e) => {
                                        this.rightTop = e;
                                    }}
                                />
                            </svg>
                        </div>
                        <div
                            className="kraken-network-right"
                            ref={(e) => {
                                this.rightTable = e;
                            }}
                        >
                            {leechesRight.length === 0 ? undefined : (
                                <table>
                                    <thead>
                                        <tr>
                                            <th>ID</th>
                                            <th>Name</th>
                                            <th>Address</th>
                                            <th>Token</th>
                                        </tr>
                                    </thead>
                                    <tbody>{leechesRight}</tbody>
                                </table>
                            )}
                        </div>
                        <div className="kraken-network-add">
                            <div
                                onClick={() => {
                                    this.setState({ showPopup: true });
                                }}
                            >
                                <svg
                                    className="neon"
                                    viewBox="0 0 1024 1024"
                                    version="1.1"
                                    xmlns="http://www.w3.org/2000/svg"
                                >
                                    <path
                                        d="M512 288a32 32 0 0 0 32-32V192a32 32 0 0 0-64 0v64a32 32 0 0 0 32 32zM494.08 90.56a24.32 24.32 0 0 0 5.76 2.88l5.76 2.56h12.48a19.84 19.84 0 0 0 6.08-1.92 17.92 17.92 0 0 0 5.44-2.88l4.8-3.84A32 32 0 0 0 544 64a32 32 0 0 0-2.56-12.16 29.12 29.12 0 0 0-7.04-10.56l-4.8-3.84a17.92 17.92 0 0 0-5.44-2.88 19.52 19.52 0 0 0-6.08-2.56 27.2 27.2 0 0 0-12.48 0 20.16 20.16 0 0 0-5.76 1.92 24.32 24.32 0 0 0-5.76 2.88l-4.8 3.84a36.8 36.8 0 0 0-6.72 10.56 26.56 26.56 0 0 0-2.56 12.8 32 32 0 0 0 9.28 22.72zM960 480H544v-96a32 32 0 0 0-64 0v96H64a32 32 0 0 0 0 64h416v416a32 32 0 0 0 64 0V544h416a32 32 0 0 0 0-64z"
                                        fill="#0cf"
                                    />
                                </svg>
                                <h2 className="heading neon">Add leech</h2>
                            </div>
                        </div>
                    </div>

                    <svg className="neon kraken-network-data">
                        <g stroke="#0cf">{lines}</g>
                    </svg>
                </div>
                <Popup
                    modal={true}
                    nested={true}
                    open={this.state.showPopup}
                    onClose={() => {
                        this.setState({ showPopup: false });
                    }}
                >
                    <form
                        method="post"
                        onSubmit={this.createLeech}
                        className="popup-content pane kraken-network-create"
                    >
                        <h1 className="heading neon">Create leech</h1>
                        <label htmlFor="name">Name</label>
                        <Input
                            className="input"
                            autoFocus={true}
                            value={this.state.name}
                            onChange={(v: string) => {
                                this.setState({ name: v });
                            }}
                        />
                        <label htmlFor="name">Address</label>
                        <Input
                            className="input"
                            value={this.state.address}
                            onChange={(v: string) => {
                                this.setState({ address: v });
                            }}
                        />
                        <button className="button">Create</button>
                    </form>
                </Popup>
            </>
        );
    }
}

function curve(fromX: number, fromY: number, toX: number, toY: number) {
    const stepX = (toX - fromX) / 3;
    return (
        <path
            fill="none"
            strokeWidth="2"
            d={`M${fromX},${fromY} C${fromX + stepX},${fromY} ${toX - stepX},${toY} ${toX},${toY}`}
        />
    );
}
