import update from "immutability-helper";
import { CSSProperties, useCallback, useState } from "react";
import { useDrop } from "react-dnd";
import "../styling/workflow-editor.css";
import { WorkflowAttackSelector } from "./workflow-editor/attack-selector";
import {
    DesignWorkflowNode,
    DragType,
    DragTypeConnector,
    DragTypeNode,
    createSampleWorkflow,
    getInputY,
    getInputs,
    getOutputY,
} from "./workflow-editor/common";
import { WorkflowEditorDragLayer } from "./workflow-editor/drag-layer";
import { DesignWorkflowNodeEditor } from "./workflow-editor/node";
import { Viewport, ViewportProps } from "./workspace/components/viewport";
import { AttackType } from "./workspace/workspace-attacks";

export interface EditorNodeCSS extends CSSProperties {
    "--fill"?: string;
}

type WorkflowEditorProps = {};

export default function WorkflowEditor(props: WorkflowEditorProps) {
    const [workflow, setWorkflow] = useState(createSampleWorkflow());
    const [previewConnection, setPreviewConnection] = useState<DragTypeConnector | undefined>(undefined);
    let [addAttack, setAddAttack] = useState<{ x: number; y: number } | undefined>(undefined);

    let nodes = Object.keys(workflow.nodes).map(
        (key) => [key, workflow.nodes[key]] as [string, DesignWorkflowNode<AttackType>],
    );

    let connections: ViewportProps["connections"] = nodes.flatMap(([id, node]) =>
        Object.keys(node.outputs).flatMap((k) =>
            node.outputs[k].flatMap((output) => {
                let target = workflow.nodes[output.into];
                if (
                    previewConnection &&
                    previewConnection.disconnectId == output.into &&
                    previewConnection.disconnectField == output.field
                )
                    return [];
                return [
                    {
                        from: [node.x + node.width, node.y + getOutputY(node, k)],
                        to: [target.x, target.y + getInputY(target, output.field)],
                    },
                ];
            }),
        ),
    );
    if (previewConnection) {
        const c = previewConnection;
        const node = workflow.nodes[c.sourceId];
        let from: [number, number] = [
            c.side == "input" ? node.x : node.x + node.width,
            c.side == "input" ? node.y + getInputY(node, c.field) : node.y + getOutputY(node, c.field),
        ];
        let to: [number, number] = [c.x, c.y];
        if (c.side == "input") [from, to] = [to, from];
        connections.push({
            from,
            to,
        });
    }

    const moveBox = useCallback(
        (id: string, x: number, y: number) => {
            setWorkflow((workflow) =>
                update(workflow, {
                    nodes: {
                        [id]: {
                            $merge: { x, y },
                        },
                    },
                }),
            );
        },
        [workflow, setWorkflow],
    );

    const getInput = (id: string, field: string): { id: string; field: string; index: number } | undefined => {
        for (const node of nodes) {
            for (const f of Object.keys(node[1].outputs)) {
                let outputs = node[1].outputs[f];
                for (let i = 0; i < outputs.length; i++) {
                    let output = outputs[i];
                    if (output.into == id && output.field == field) {
                        return {
                            id: node[0],
                            field: f,
                            index: i,
                        };
                    }
                }
            }
        }
    };

    const setInput = useCallback(
        (id: string, field: string, from: string | undefined, fromField: string | undefined) => {
            // remove existing outgoing connections (just iterate over the whole nodes right now)
            let existing = getInput(id, field);
            if (existing) {
                setWorkflow((workflow) =>
                    update(workflow, {
                        nodes: {
                            [existing!.id]: {
                                outputs: {
                                    [existing!.field]: {
                                        $splice: [[existing!.index, 1]],
                                    },
                                },
                            },
                        },
                    }),
                );
            }
            // now add a new output:
            if (from && fromField) {
                setWorkflow((workflow) =>
                    update(workflow, {
                        nodes: {
                            [from]: {
                                outputs: {
                                    [fromField]: {
                                        $push: [
                                            {
                                                into: id,
                                                field: field,
                                            },
                                        ],
                                    },
                                },
                            },
                        },
                    }),
                );
            }
        },
        [workflow, setWorkflow],
    );

    const [, drop] = useDrop<DragTypeNode | DragTypeConnector>(
        {
            accept: [DragType.node, DragType.connector],
            hover(item, monitor) {
                const delta = monitor.getDifferenceFromInitialOffset();
                const x = delta ? Math.round(item.x + delta.x) : item.x;
                const y = delta ? Math.round(item.y + delta.y) : item.y;
                if ("id" in item) {
                    moveBox(item.id, x, y);
                } else {
                    setPreviewConnection({
                        sourceId: item.sourceId,
                        side: item.side,
                        field: item.field,
                        x,
                        y,
                        disconnectField: item.disconnectField,
                        disconnectId: item.disconnectId,
                    });
                }
            },
            drop(item, monitor) {
                const delta = monitor.getDifferenceFromInitialOffset();
                const x = delta ? Math.round(item.x + delta.x) : item.x;
                const y = delta ? Math.round(item.y + delta.y) : item.y;
                if ("id" in item) {
                    moveBox(item.id, x, y);
                } else {
                    if (item.disconnectField && item.disconnectId) {
                        setInput(item.disconnectId, item.disconnectField, undefined, undefined);
                    }
                    setPreviewConnection(undefined);
                }
            },
        },
        [moveBox, setPreviewConnection],
    );

    return (
        <div className="workflow-editor-root">
            <WorkflowAttackSelector
                x={addAttack?.x ?? 0}
                y={addAttack?.y ?? 0}
                open={addAttack !== undefined}
                onClose={() => setAddAttack(undefined)}
            />
            <Viewport
                className="workflow-editor"
                onContextMenu={(e) => {
                    if (!e.ctrlKey && addAttack === undefined) {
                        setAddAttack({ x: e.pageX, y: e.pageY });
                        e.preventDefault();
                    }
                }}
                ref={drop}
                connections={connections}
            >
                {nodes.map(([key, node]) => (
                    <DesignWorkflowNodeEditor
                        connectInput={(dstField, srcId, srcField) => {
                            setInput(key, dstField, srcId, srcField);
                        }}
                        connectOutput={(srcField, dstId, dstField) => {
                            setInput(dstId, dstField, key, srcField);
                        }}
                        connectedInputs={Object.fromEntries(
                            Object.keys(getInputs(node.attack))
                                .map((k) => [k, getInput(key, k)] as [string, ReturnType<typeof getInput>])
                                .filter((v) => v[1] !== undefined)
                                .map((v) => [
                                    v[0],
                                    {
                                        from: v[1]!.id,
                                        field: v[1]!.field,
                                    },
                                ]),
                        )}
                        id={key}
                        key={"node-" + key}
                        {...node}
                    />
                ))}
            </Viewport>

            <WorkflowEditorDragLayer />
        </div>
    );
}
