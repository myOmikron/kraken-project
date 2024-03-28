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

interface AttackDictionary {
    [Key: UUID]: Array<SimpleAttack>;
}

export default function RunningAttacks() {
    const [runningAttacks, setRunningAttacks] = React.useState<AttackDictionary>({});

    React.useEffect(() => {
        Api.attacks.all().then((x) =>
            x.match(
                (attacks) => {
                    const runningAttacks: AttackDictionary = {};
                    for (const attack of attacks.attacks) {
                        if (runningAttacks[attack.workspace.uuid] !== null) {
                            runningAttacks[attack.workspace.uuid] = [attack];
                        } else {
                            runningAttacks[attack.workspace.uuid].push(attack);
                        }
                    }
                    setRunningAttacks(runningAttacks);
                },
                (err) => toast.error(err.message),
            ),
        );
        WS.addEventListener("message.AttackStarted", (msg) => {
            const r = runningAttacks;
            if (r[msg.attack.workspace.uuid] === null) {
                r[msg.attack.workspace.uuid] = [msg.attack];
            } else {
                r[msg.attack.workspace.uuid] = [msg.attack, ...r[msg.attack.workspace.uuid]];
            }
            setRunningAttacks(r);
        });
        WS.addEventListener("message.AttackFinished", (msg) => {
            const r = runningAttacks;
            if (r[msg.attack.workspace.uuid] !== null) {
                const workspaceAttacks = r[msg.attack.workspace.uuid];
                for (const workspaceAttack of workspaceAttacks) {
                    if (workspaceAttack.uuid === msg.attack.uuid) {
                        workspaceAttack.error = msg.attack.error === undefined ? null : msg.attack.error;
                        workspaceAttack.finishedAt = msg.attack.finishedAt;
                    }
                }
            }

            setRunningAttacks(r);
        });
    }, []);

    return (
        <div className={"running-attacks-container"}>
            {Object.entries(runningAttacks).map(([key, value]) => {
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
