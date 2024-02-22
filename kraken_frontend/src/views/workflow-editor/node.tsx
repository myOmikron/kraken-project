import { useEffect } from "react";
import { useDrag, useDrop } from "react-dnd";
import { getEmptyImage } from "react-dnd-html5-backend";
import { EditorNodeCSS } from "../workflow-editor";
import { ATTACKS, AttackType } from "../workspace/workspace-attacks";
import {
    DesignWorkflowNode,
    DragType,
    DragTypeConnector,
    DragTypeNode,
    generateFill,
    getInputY,
    getInputs,
    getOutputY,
} from "./common";

export type DesignWorkflowNodeProps<T extends AttackType> =
    | ({
          preview: true;
      } & Omit<DesignWorkflowNode<T>, "x" | "y">)
    | ({
          preview?: false;
          id: string;
          connectedInputs: { [field: string]: { from: string; field: string } };
          connectInput: (intoField: string, from: string, fromField: string) => any;
          connectOutput: (fromField: string, to: string, toField: string) => any;
      } & DesignWorkflowNode<T>);

export function DesignWorkflowNodeEditor<T extends AttackType>(node: DesignWorkflowNodeProps<T>) {
    let inputs = getInputs(node.attack);

    const [{ isDragging }, drag, preview] = useDrag(
        () => ({
            type: DragType.node,
            item: node as DragTypeNode,
            collect: (monitor) => ({
                isDragging: monitor.isDragging(),
            }),
        }),
        [node],
    );

    useEffect(() => {
        preview(getEmptyImage());
    }, []);

    return (
        <div
            id={"node-" + (node.preview ? "preview" : node.id)}
            ref={drag}
            className={`workflow-editor-node pane-thin ${node.preview ? "dragging" : ""}`}
            tabIndex={0}
            style={
                isDragging
                    ? {
                          opacity: 0,
                          pointerEvents: "none",
                      }
                    : node.preview
                      ? {
                            width: node.width + "px",
                        }
                      : {
                            left: node.x + "px",
                            top: node.y + "px",
                            width: node.width + "px",
                        }
            }
        >
            <label className={`category-${ATTACKS[node.attack].category}`}>{ATTACKS[node.attack].name}</label>
            <div className="inputs">
                {Object.keys(inputs).map((field) => (
                    <div
                        className={`${inputs[field].multi ? "list" : ""}`}
                        style={
                            {
                                "--fill": generateFill(inputs[field]),
                            } as EditorNodeCSS
                        }
                    >
                        {node.preview ? (
                            <Connector preview side="input" id={field} />
                        ) : (
                            <Connector
                                preview={node.preview}
                                node={node.id}
                                side="input"
                                connectedFromField={node.connectedInputs[field]?.field}
                                connectedFromNode={node.connectedInputs[field]?.from}
                                id={field}
                                x={node.x}
                                y={node.y + getInputY(node, field)}
                                onConnect={node.connectInput}
                            />
                        )}
                        {inputs[field].label}
                    </div>
                ))}
            </div>
            <div className="outputs">
                {Object.keys(node.outputs).map((output) => (
                    <div
                        className="domain stream"
                        style={
                            {
                                "--fill": "var(--type-domain)",
                            } as EditorNodeCSS
                        }
                    >
                        {node.preview ? (
                            <Connector preview side="output" id={output} />
                        ) : (
                            <Connector
                                side="output"
                                id={output}
                                node={node.id}
                                x={node.x + node.width}
                                y={node.y + getOutputY(node, output)}
                                onConnect={node.connectOutput}
                            />
                        )}
                        {output}
                    </div>
                ))}
            </div>
        </div>
    );
}

type ConnectorProps =
    | {
          preview: true;
          side: "input" | "output";
          id: string;
      }
    | {
          preview?: false;
          node: string;
          side: "input";
          connectedFromNode: string | undefined;
          connectedFromField: string | undefined;
          id: string;
          x: number;
          y: number;
          onConnect: (thisField: string, node: string, field: string) => any;
      }
    | {
          preview?: false;
          node: string;
          side: "output";
          id: string;
          x: number;
          y: number;
          onConnect: (thisField: string, node: string, field: string) => any;
      };

function Connector(props: ConnectorProps) {
    const [{ isOver }, drop] = props.preview
        ? [{ isOver: false }, undefined]
        : useDrop<DragTypeConnector, unknown, { isOver: boolean }>(
              {
                  accept: DragType.connector,
                  drop(item, monitor) {
                      if (props.preview) return;
                      if (item.side == props.side) return;
                      props.onConnect(props.id, item.sourceId, item.field);
                  },
                  collect: (monitor) => ({
                      isOver: monitor.getItem()?.side != props.side && monitor.isOver(),
                  }),
              },
              [props],
          );
    const [, drag, preview] = props.preview
        ? [undefined, undefined, undefined]
        : useDrag(
              () => ({
                  type: DragType.connector,
                  item: () => {
                      if (props.side == "input" && props.connectedFromField && props.connectedFromNode) {
                          return {
                              field: props.connectedFromField,
                              side: "output",
                              sourceId: props.connectedFromNode,
                              x: props.x,
                              y: props.y,
                              disconnectField: props.id,
                              disconnectId: props.node,
                          } as DragTypeConnector;
                      } else {
                          return {
                              field: props.id,
                              side: props.side,
                              sourceId: props.node,
                              x: props.x,
                              y: props.y,
                          } as DragTypeConnector;
                      }
                  },
              }),
              [props],
          );

    useEffect(() => {
        if (!props.preview) preview!(getEmptyImage());
    }, []);

    return (
        <>
            {!props.preview && (
                <div
                    className={`connector-drag connector-side-${props.side} ${isOver ? "drop-preview" : ""}`}
                    ref={(v) => {
                        drop!(v);
                        drag!(v);
                    }}
                ></div>
            )}
            <div
                key={`connector-${props.id}`}
                className={`connector connector-side-${props.side} ${isOver ? "drop-preview" : ""}`}
            ></div>
        </>
    );
}
