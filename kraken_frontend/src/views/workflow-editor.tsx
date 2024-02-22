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

    let connections = nodes.flatMap(([id, node]) =>
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
        let from = [
            c.side == "input" ? node.x : node.x + node.width,
            c.side == "input" ? node.y + getInputY(node, c.field) : node.y + getOutputY(node, c.field),
        ];
        let to = [c.x, c.y];
        if (c.side == "input") [from, to] = [to, from];
        connections.push({
            from,
            to,
        });
    }
    const padding = 300;
    let cx = {
        minX: Math.min(...connections.map((v) => Math.min(v.from[0], v.to[0]))) - padding,
        minY: Math.min(...connections.map((v) => Math.min(v.from[1], v.to[1]))) - padding,
        maxX: Math.max(...connections.map((v) => Math.max(v.from[0], v.to[0]))) + padding,
        maxY: Math.max(...connections.map((v) => Math.max(v.from[1], v.to[1]))) + padding,
        width: 0,
        height: 0,
    };
    cx.width = cx.maxX - cx.minX;
    cx.height = cx.maxY - cx.minY;
    // when adjusting the workflow to update it with setState afterwards,
    // operate on this value instead so that multiple updates in the same frame
    // are combined and not discard the previous one.
    let newWorkflow = workflow;

    const moveBox = useCallback(
        (id: string, x: number, y: number) => {
            setWorkflow(
                (newWorkflow = update(newWorkflow, {
                    nodes: {
                        [id]: {
                            $merge: { x, y },
                        },
                    },
                })),
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
                newWorkflow = update(newWorkflow, {
                    nodes: {
                        [existing.id]: {
                            outputs: {
                                [existing.field]: {
                                    $splice: [[existing.index, 1]],
                                },
                            },
                        },
                    },
                });
            }
            // now add a new output:
            if (from && fromField) {
                newWorkflow = update(newWorkflow, {
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
                });
            }
            setWorkflow(newWorkflow);
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
        <div
            className="workflow-editor"
            onContextMenu={(e) => {
                if (!e.ctrlKey && addAttack === undefined) {
                    setAddAttack({ x: e.pageX, y: e.pageY });
                    e.preventDefault();
                }
            }}
        >
            <WorkflowAttackSelector
                x={addAttack?.x ?? 0}
                y={addAttack?.y ?? 0}
                open={addAttack !== undefined}
                onClose={() => setAddAttack(undefined)}
            />
            <div className="pane" ref={drop}>
                <div className="nodes">
                    <svg
                        className="connections"
                        style={{
                            left: cx.minX + "px",
                            top: cx.minY + "px",
                            width: cx.width + "px",
                            height: cx.height + "px",
                        }}
                        viewBox={`${cx.minX} ${cx.minY} ${cx.width} ${cx.height}`}
                    >
                        {connections.map((c, i) => {
                            const curviness = Math.abs(c.from[0] - c.to[0] + 40) / 2;
                            const padding = 10;
                            const path =
                                `M${c.from[0]},${c.from[1]}` +
                                `h${padding}` +
                                `C${c.from[0] + padding + curviness},${c.from[1]},${c.to[0] - curviness - padding},${
                                    c.to[1]
                                },${c.to[0] - padding},${c.to[1]}` +
                                `h${padding}`;
                            return (
                                <>
                                    <path key={"connection-" + i} d={path} stroke="white" strokeWidth={2} fill="none" />
                                    <path
                                        key={"select-connection-" + i}
                                        d={path}
                                        strokeWidth={12}
                                        stroke="transparent"
                                        fill="none"
                                    />
                                </>
                            );
                        })}
                    </svg>
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
                </div>
            </div>
            <WorkflowEditorDragLayer />
        </div>
    );
}
