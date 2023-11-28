import React from "react";
import "../styling/running-attacks.css";
import RunningAttackIcon from "../svg/running-attack";
import Popup from "reactjs-popup";
import SuccessIcon from "../svg/success";
import FailedIcon from "../svg/failed";
import WS from "../api/websocket";
import { UUID } from "../api/api";
import { AttackType } from "../api/generated";
import { toast } from "react-toastify";

type RunningAttacksProps = {};
type RunningAttacksState = {
    runningAttacks: {
        // key is the workspace uuid
        [key: UUID]: [
            {
                attack: UUID;
                attack_type: AttackType;
                finished: boolean;
                error: null | string;
            }
        ];
    };
};

export default class RunningAttacks extends React.Component<RunningAttacksProps, RunningAttacksState> {
    constructor(props: RunningAttacksProps) {
        super(props);

        this.state = {
            runningAttacks: {},
        };
    }

    componentDidMount() {
        WS.addEventListener("message.AttackStarted", () => {
            toast.success("Attack started through ws");
        });
    }

    render() {
        return (
            <div className={"running-attacks-container"}>
                <div>
                    Workspace
                    <br /> pst-test
                </div>
                <Popup
                    trigger={
                        <div className={"running-attacks-attack"}>
                            <RunningAttackIcon />
                            <span className={"running-attacks-inner neon"}>SvD</span>
                        </div>
                    }
                    position={"bottom center"}
                    on={"hover"}
                    arrow={true}
                >
                    <div className={"pane-thin"}>
                        <h2 className={"sub-heading"}>Service Detection</h2>
                        <span>Workspace: pst-test</span>
                        <span>Started by: Omikron</span>
                        <span>Started at: 2023-11-15 17:30:00 +1:00</span>
                    </div>
                </Popup>
                <Popup
                    trigger={
                        <div className={"running-attacks-attack"}>
                            <RunningAttackIcon />
                            <span className={"running-attacks-inner neon"}>HA</span>
                        </div>
                    }
                    position={"bottom center"}
                    on={"hover"}
                    arrow={true}
                >
                    <div className={"pane-thin"}>
                        <h2 className={"sub-heading"}>SvC</h2>
                        <span>Workspace: pst-test</span>
                        <span>Started by: Omikron</span>
                        <span>Started at: 2023-11-15 17:30:00 +1:00</span>
                    </div>
                </Popup>
                <Popup
                    trigger={
                        <div className={"running-attacks-attack"}>
                            <RunningAttackIcon />
                            <span className={"running-attacks-inner stopped neon"}>
                                <span>HA</span>
                                <FailedIcon />
                            </span>
                        </div>
                    }
                    position={"bottom center"}
                    on={"hover"}
                    arrow={true}
                >
                    <div className={"pane-thin"}>
                        <h2 className={"sub-heading"}>SvC</h2>
                        <span>Workspace: pst-test</span>
                        <span>Started by: Omikron</span>
                        <span>Started at: 2023-11-15 17:30:00 +1:00</span>
                    </div>
                </Popup>
                <Popup
                    trigger={
                        <div className={"running-attacks-attack"}>
                            <RunningAttackIcon />
                            <span className={"running-attacks-inner stopped neon"}>
                                <span>SvD</span>
                                <SuccessIcon />
                            </span>
                        </div>
                    }
                    position={"bottom center"}
                    on={"hover"}
                    arrow={true}
                >
                    <div className={"pane-thin"}>
                        <h2 className={"sub-heading"}>SvC</h2>
                        <span>Workspace: pst-test</span>
                        <span>Started by: Omikron</span>
                        <span>Started at: 2023-11-15 17:30:00 +1:00</span>
                    </div>
                </Popup>

                <div className={"running-attacks-seperator"}></div>
                <div className={"neon"}>
                    Workspace
                    <br /> pst-test
                </div>
                <Popup
                    trigger={
                        <div className={"running-attacks-attack"}>
                            <RunningAttackIcon />
                            <span className={"running-attacks-inner neon"}>SvD</span>
                        </div>
                    }
                    position={"bottom center"}
                    on={"hover"}
                    arrow={true}
                >
                    <div className={"pane-thin"}>
                        <h2 className={"sub-heading"}>Service Detection</h2>
                        <span>Workspace: pst-test</span>
                        <span>Started by: Omikron</span>
                        <span>Started at: 2023-11-15 17:30:00 +1:00</span>
                    </div>
                </Popup>
                <Popup
                    trigger={
                        <div className={"running-attacks-attack"}>
                            <RunningAttackIcon />
                            <span className={"running-attacks-inner neon"}>HA</span>
                        </div>
                    }
                    position={"bottom center"}
                    on={"hover"}
                    arrow={true}
                >
                    <div className={"pane-thin"}>
                        <h2 className={"sub-heading"}>SvC</h2>
                        <span>Workspace: pst-test</span>
                        <span>Started by: Omikron</span>
                        <span>Started at: 2023-11-15 17:30:00 +1:00</span>
                    </div>
                </Popup>
                <Popup
                    trigger={
                        <div className={"running-attacks-attack"}>
                            <RunningAttackIcon />
                            <span className={"running-attacks-inner stopped neon"}>
                                <span>HA</span>
                                <FailedIcon />
                            </span>
                        </div>
                    }
                    position={"bottom center"}
                    on={"hover"}
                    arrow={true}
                >
                    <div className={"pane-thin"}>
                        <h2 className={"sub-heading"}>SvC</h2>
                        <span>Workspace: pst-test</span>
                        <span>Started by: Omikron</span>
                        <span>Started at: 2023-11-15 17:30:00 +1:00</span>
                    </div>
                </Popup>
                <Popup
                    trigger={
                        <div className={"running-attacks-attack"}>
                            <RunningAttackIcon />
                            <span className={"running-attacks-inner stopped neon"}>
                                <span>SvD</span>
                                <SuccessIcon />
                            </span>
                        </div>
                    }
                    position={"bottom center"}
                    on={"hover"}
                    arrow={true}
                >
                    <div className={"pane-thin"}>
                        <h2 className={"sub-heading"}>SvC</h2>
                        <span>Workspace: pst-test</span>
                        <span>Started by: Omikron</span>
                        <span>Started at: 2023-11-15 17:30:00 +1:00</span>
                    </div>
                </Popup>
            </div>
        );
    }
}
