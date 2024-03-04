import React from "react";
import { toast } from "react-toastify";
import Popup from "reactjs-popup";
import { Api, UUID } from "../api/api";
import { SimpleAttack } from "../api/generated";
import WS from "../api/websocket";
import "../styling/running-attacks.css";
import FailedIcon from "../svg/failed";
import RunningAttackIcon from "../svg/running-attack";
import SuccessIcon from "../svg/success";
import { ATTACKS } from "../utils/attack-resolver";

type RunningAttacksProps = {};
type RunningAttacksState = {
    runningAttacks: AttackDictionary;
};

interface AttackDictionary {
    [Key: UUID]: Array<SimpleAttack>;
}

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
                        if (runningAttacks[attack.workspace.uuid] !== null) {
                            runningAttacks[attack.workspace.uuid] = [attack];
                        } else {
                            runningAttacks[attack.workspace.uuid].push(attack);
                        }
                    }
                    this.setState({ runningAttacks });
                },
                (err) => toast.error(err.message),
            ),
        );
        WS.addEventListener("message.AttackStarted", (msg) => {
            let runningAttacks = this.state.runningAttacks;
            if (runningAttacks[msg.attack.workspace.uuid] === null) {
                runningAttacks[msg.attack.workspace.uuid] = [msg.attack];
            } else {
                runningAttacks[msg.attack.workspace.uuid] = [msg.attack, ...runningAttacks[msg.attack.workspace.uuid]];
            }

            this.setState({ runningAttacks });
        });
        WS.addEventListener("message.AttackFinished", (msg) => {
            let runningAttacks = this.state.runningAttacks;
            if (runningAttacks[msg.attack.workspace.uuid] === null) {
            } else {
                let workspaceAttacks = runningAttacks[msg.attack.workspace.uuid];
                for (let workspaceAttack of workspaceAttacks) {
                    if (workspaceAttack.uuid === msg.attack.uuid) {
                        workspaceAttack.error = msg.attack.error === undefined ? null : msg.attack.error;
                        workspaceAttack.finishedAt = msg.attack.finishedAt;
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
                        <React.Fragment key={key}>
                            <div>
                                Workspace
                                <br />
                            </div>
                            {value.map((attack) => (
                                <Popup
                                    key={attack.uuid}
                                    trigger={
                                        <div key={attack.uuid + "_trigger"} className={"running-attacks-attack"}>
                                            <RunningAttackIcon />
                                            {attack.finishedAt === null ? (
                                                <span className={"running-attacks-inner neon"}>
                                                    {ATTACKS[attack.attackType].abbreviation}
                                                </span>
                                            ) : (
                                                <span className={"running-attacks-inner stopped neon"}>
                                                    <span>{ATTACKS[attack.attackType].abbreviation}</span>
                                                    {attack.error === null || attack.error === undefined ? (
                                                        <SuccessIcon />
                                                    ) : (
                                                        <FailedIcon />
                                                    )}
                                                </span>
                                            )}
                                        </div>
                                    }
                                    position={"bottom left"}
                                    on={"hover"}
                                    arrow={true}
                                >
                                    <div className={"pane-thin"}>
                                        <h2 className={"sub-heading"}>{ATTACKS[attack.attackType].long}</h2>
                                        {attack.error !== null && attack.error !== undefined ? (
                                            <span>Error: {attack.error}</span>
                                        ) : undefined}
                                        <span>Workspace: {attack.workspace.name}</span>
                                        <span>Started by: {attack.startedBy.displayName}</span>
                                        <span>Started at: {attack.finishedAt?.toLocaleString()}</span>
                                    </div>
                                </Popup>
                            ))}
                            <div className={"running-attacks-seperator"}></div>
                        </React.Fragment>
                    );
                })}
            </div>
        );
    }
}
