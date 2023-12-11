import React from "react";
import "../styling/running-attacks.css";
import RunningAttackIcon from "../svg/running-attack";
import Popup from "reactjs-popup";
import SuccessIcon from "../svg/success";
import FailedIcon from "../svg/failed";
import WS from "../api/websocket";
import { Api, UUID } from "../api/api";
import { AttackType } from "../api/generated";
import { toast } from "react-toastify";
import { ATTACKS } from "../utils/attack-resolver";

type RunningAttacksProps = {};
type RunningAttacksState = {
    runningAttacks: AttackDictionary;
};

interface AttackDictionary {
    [Key: UUID]: Array<Attack>;
}

type Attack = {
    attack: UUID;
    attack_type: AttackType;
    finishedSuccessful: boolean | null;
    error: null | string;
};

export default class RunningAttacks extends React.Component<RunningAttacksProps, RunningAttacksState> {
    constructor(props: RunningAttacksProps) {
        super(props);

        this.state = {
            runningAttacks: {},
        };
    }

    componentDidMount() {
        Api.attacks.all().then((x) =>
            x.match(
                (attacks) => {
                    let runningAttacks: AttackDictionary = {};
                    for (const attack of attacks.attacks) {
                        const a = {
                            attack: attack.uuid,
                            attack_type: attack.attackType,
                            error: attack.error !== undefined ? attack.error : null,
                            finishedSuccessful:
                                attack.finishedAt !== undefined && attack.finishedAt !== null ? true : null,
                        };
                        if (runningAttacks[attack.workspaceUuid] !== null) {
                            runningAttacks[attack.workspaceUuid] = [a];
                        } else {
                            runningAttacks[attack.workspaceUuid].push(a);
                        }
                    }
                    this.setState({ runningAttacks });
                },
                (err) => toast.error(err.message)
            )
        );
        WS.addEventListener("message.AttackStarted", (attack) => {
            const a = {
                attack: attack.attackUuid,
                attack_type: attack.attackType,
                error: null,
                finishedSuccessful: null,
            };

            let runningAttacks = this.state.runningAttacks;
            if (runningAttacks[attack.workspaceUuid] === null) {
                runningAttacks[attack.workspaceUuid] = [a];
            } else {
                runningAttacks[attack.workspaceUuid] = [a, ...runningAttacks[attack.workspaceUuid]];
            }

            this.setState({ runningAttacks });
        });
        WS.addEventListener("message.AttackFinished", (attack) => {
            let runningAttacks = this.state.runningAttacks;
            if (runningAttacks[attack.workspaceUuid] === null) {
            } else {
                let workspaceAttacks = runningAttacks[attack.workspaceUuid];
                for (let workspaceAttack of workspaceAttacks) {
                    if (workspaceAttack.attack === attack.attackUuid) {
                        workspaceAttack.error = attack.error === undefined ? null : attack.error;
                        workspaceAttack.finishedSuccessful = attack.finishedSuccessful;
                    }
                }
            }

            this.setState({ runningAttacks });
        });
    }

    render() {
        return (
            <div className={"running-attacks-container"}>
                {Object.entries(this.state.runningAttacks).map(([key, value]) => {
                    return (
                        <>
                            <div>
                                Workspace
                                <br />
                            </div>
                            {value.map((attack) => (
                                <Popup
                                    trigger={
                                        <div className={"running-attacks-attack"}>
                                            <RunningAttackIcon />
                                            {attack.finishedSuccessful === null ? (
                                                <span className={"running-attacks-inner neon"}>
                                                    {ATTACKS[attack.attack_type].abbreviation}
                                                </span>
                                            ) : (
                                                <span className={"running-attacks-inner stopped neon"}>
                                                    <span>{ATTACKS[attack.attack_type].abbreviation}</span>
                                                    {attack.finishedSuccessful ? <SuccessIcon /> : <FailedIcon />}
                                                </span>
                                            )}
                                        </div>
                                    }
                                    position={"bottom center"}
                                    on={"hover"}
                                    arrow={true}
                                >
                                    <div className={"pane-thin"}>
                                        <h2 className={"sub-heading"}>{ATTACKS[attack.attack_type].long}</h2>
                                        {attack.error !== null ? <span>Error: {attack.error}</span> : undefined}
                                        <span>Workspace: pst-test</span>
                                        <span>Started by: Omikron</span>
                                        <span>Started at: 2023-11-15 17:30:00 +1:00</span>
                                    </div>
                                </Popup>
                            ))}
                            <div className={"running-attacks-seperator"}></div>
                        </>
                    );
                })}
            </div>
        );
    }
}
