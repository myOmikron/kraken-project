import {
    BooleanAttackInput,
    DurationAttackInput,
    NumberAttackInput,
    PortListInput,
    StringAttackInput,
    WordlistAttackInput,
} from "../workspace/attacks/attack-input";
import { ATTACKS, AttackType, IAttackInput, PrefilledAttackParams } from "../workspace/workspace-attacks";

export enum DragType {
    node = "workflow.node",
    connector = "workflow.connector",
}

export type DragTypeNode = DesignWorkflowNode<any> & { id: string };
export type DragTypeConnector = {
    x: number;
    y: number;
    sourceId: string;
    side: "input" | "output";
    field: string;
    disconnectId: string | undefined;
    disconnectField: string | undefined;
};

export type DesignWorkflow = {
    nodes: { [key: string]: DesignWorkflowNode<keyof typeof ATTACKS> };
};

export type DesignWorkflowNode<T extends keyof typeof ATTACKS> = {
    x: number;
    y: number;
    width: number;
    attack: T;
    fixed: {
        [Key in keyof (typeof ATTACKS)[T]["inputs"]["inputs"]]?: (typeof ATTACKS)[T]["inputs"]["inputs"][Key];
    };
    outputs: {
        [key: string]: {
            into: string;
            field: string;
        }[];
    };
};

export const getInputs = (type: AttackType): { [index: string]: IAttackInput } => ATTACKS[type].inputs.inputs as any;

export const prefillColors: { [K in keyof PrefilledAttackParams]: string } = {
    domain: "var(--type-domain)",
    ipAddr: "var(--type-ip-addr)",
    port: "var(--type-port)",
};

export const generateFill = (input: IAttackInput, multi: boolean = true) => {
    if (Array.isArray(input.prefill) && input.prefill.length > 1 && multi) {
        let d = 360 / input.prefill.length;
        return (
            "conic-gradient(from 45deg, " +
            input.prefill
                .map((p, i) => `${prefillColors[p]} ${i * d}deg, ${prefillColors[p]} ${i * d + d}deg`)
                .join(", ") +
            ")"
        );
    } else if (Array.isArray(input.prefill) && input.prefill.length > 0) return prefillColors[input.prefill[0]];
    else if (typeof input.prefill == "string") return prefillColors[input.prefill];
    else if (input.type == NumberAttackInput) return "var(--type-number)";
    else if (input.type == DurationAttackInput) return "var(--type-duration)";
    else if (input.type == BooleanAttackInput) return "var(--type-boolean)";
    else if (input.type == PortListInput) return "var(--type-port)";
    else if (input.type == StringAttackInput) return "var(--type-string)";
    else if (input.type == WordlistAttackInput) return "var(--type-wordlist)";
    else return "white";
};

const rem = 16; //parseFloat(getComputedStyle(document.body).fontSize);
export const getInputY = (node: DesignWorkflowNode<keyof typeof ATTACKS>, key: string) =>
    Math.ceil(2.5 * rem) + Math.ceil(2 * rem) * Object.keys(getInputs(node.attack)).indexOf(key) + 1 * rem + 2;
export const getOutputY = (node: DesignWorkflowNode<keyof typeof ATTACKS>, key: string) =>
    Math.ceil(2.5 * rem) +
    Math.ceil(2 * rem) * Object.keys(getInputs(node.attack)).length +
    Math.ceil(2 * rem) * Object.keys(node.outputs).indexOf(key) +
    1 * rem +
    2;

export function createSampleWorkflow(): DesignWorkflow {
    return {
        nodes: {
            "1": {
                x: -200,
                y: -200,
                width: 160,
                attack: AttackType.DnsTxtScan,
                fixed: {},
                outputs: {
                    domain: [
                        {
                            into: "4",
                            field: "targets",
                        },
                        {
                            into: "3",
                            field: "targets",
                        },
                    ],
                },
            },
            "2": {
                x: 80,
                y: 140,
                width: 0,
                attack: AttackType.Dehashed,
                fixed: {},
                outputs: {},
            },
            "3": {
                x: -300,
                y: 60,
                width: 200,
                attack: AttackType.HostAlive,
                fixed: {},
                outputs: {},
            },
            "4": {
                x: 300,
                y: -120,
                width: 200,
                attack: AttackType.ServiceDetection,
                fixed: {},
                outputs: {},
            },
        },
    };
}
