import * as d3 from "d3";
import { forwardRef, useEffect, useRef, useState } from "react";
import {
    FullDomain,
    FullHost,
    FullPort,
    FullService,
    SimpleFindingDefinition,
    SimpleTag,
} from "../../../api/generated";
import TagList, { TagClickCallback } from "../components/tag-list";
import { Viewport, ViewportProps, ViewportRef } from "../components/viewport";

export type TreeNode = {
    uuid: string;
    children?: Array<TreeNode>;
} & (
    | {
          definition: SimpleFindingDefinition;
          type: "Finding";
      }
    | {
          type: "Host";
          host: FullHost;
      }
    | {
          type: "Port";
          port: FullPort;
      }
    | {
          type: "Service";
          service: FullService;
      }
    | {
          type: "Domain";
          domain: FullDomain;
      }
);

export function TreeGraph({
    roots,
    onClickTag,
    children,
    ...props
}: { roots: TreeNode[]; onClickTag?: TagClickCallback } & ViewportProps) {
    const verticalMargin = 16;
    const horizontalMargin = 64;
    const treeNodeWidth = 200; // from CSS: .tree-node width in px

    type NodeT = d3.SimulationNodeDatum & { uuid: string; radius?: number; children?: { uuid: string }[] };
    type LinkT = d3.SimulationLinkDatum<NodeT> & { sourceUuid: string; targetUuid: string };
    type ConnectionT = {
        from: [number, number];
        to: [number, number];
        fromUuid: string;
        toUuid: string;
        className: string;
    };
    const defaultConnectionClass = "";
    const highlightConnectionClass = "highlighted";

    const [hoveredUuid, setHoveredUuid] = useState<string | undefined>(undefined);
    const [highlighted, setHighlighted] = useState<string[]>([]);
    const [connections, setConnections] = useState<ConnectionT[]>([]);
    const viewportRef = useRef<ViewportRef>(null);
    const simulation = useRef<d3.Simulation<NodeT, LinkT>>();
    const simulationState = useRef<{
        nodes: NodeT[];
        links: LinkT[];
        rootUuids: string;
        heights: { [uuid: string]: number };
    }>({
        nodes: [],
        links: [],
        rootUuids: "",
        heights: {},
    });

    const hoverDebounceTimer = useRef<number>();

    const setHovered = (uuid: string) => {
        clearTimeout(hoverDebounceTimer.current);
        hoverDebounceTimer.current = setTimeout(
            function () {
                // use callback syntax to call setter, since we also do that in the unset
                // if we didn't do that here, the state might get unsynced with the events,
                // e.g. the user might hover over something but the class is not getting applied
                setHoveredUuid(() => {
                    setHighlighted(
                        Array.from(
                            new Set([
                                uuid,
                                ...simulationState.current.links
                                    .filter((l) => l.sourceUuid == uuid || l.targetUuid == uuid)
                                    .flatMap((l) => [l.sourceUuid, l.targetUuid]),
                            ]).values(),
                        ),
                    );
                    return uuid;
                });
            },
            hoveredUuid != undefined ? 0 : 100,
        );
    };

    const unsetHovered = (uuid: string) => {
        clearTimeout(hoverDebounceTimer.current);
        hoverDebounceTimer.current = setTimeout(function () {
            setHoveredUuid((prev) => {
                if (prev == uuid) {
                    setHighlighted([]);
                    return undefined;
                }
                return prev;
            });
        }, 50);
    };

    const getElement = (uuid: string): HTMLElement | null =>
        viewportRef.current?.getRootElement()?.querySelector<HTMLElement>(`[data-uuid="${uuid}"]`) ?? null;

    const getHeight = (uuid: string) => {
        return (
            simulationState.current.heights[uuid] ||
            (function () {
                const e = getElement(uuid);
                if (!e) return 100;
                return (simulationState.current.heights[uuid] = e.getBoundingClientRect().height);
            })()
        );
    };

    const centeringForce = d3.forceY<NodeT>(0).strength((n) => (roots.some((r) => r.uuid == n.uuid) ? 1 : 0.01));

    useEffect(() => {
        console.log("start simulation");
        simulation.current = d3
            .forceSimulation<NodeT, LinkT>()
            .force("charge", d3.forceManyBody())
            .force(
                "collide",
                d3
                    .forceCollide<NodeT>()
                    .radius((d) => (getHeight(d.uuid) + verticalMargin) / 2)
                    .strength(1)
                    .iterations(5),
            )
            .force(
                "link",
                d3
                    .forceLink<NodeT, LinkT>()
                    .id((d) => d.uuid)
                    .strength((l) => (roots.some((r) => r.uuid == l.source || l.target) ? 0 : 0.2)),
            )
            .force("centering", centeringForce)
            .on("tick", () => {
                const sim = simulation.current;
                if (!sim) return;
                for (const node of sim.nodes()) {
                    let elem = getElement(node.uuid);
                    if (elem) {
                        if (node.x !== undefined) elem.style.left = `${Math.round(node.x)}px`;
                        if (node.y !== undefined) elem.style.top = `${Math.round(node.y)}px`;
                    }
                }
                const linkForce = sim.force("link")! as d3.ForceLink<NodeT, LinkT>;
                setConnections(
                    linkForce.links().map((l) => {
                        let res: ConnectionT = {
                            from:
                                typeof l.source == "object"
                                    ? [(l.source.x ?? NaN) + treeNodeWidth / 2, l.source.y ?? NaN]
                                    : [NaN, NaN],
                            to:
                                typeof l.target == "object"
                                    ? [(l.target.x ?? NaN) - treeNodeWidth / 2, l.target.y ?? NaN]
                                    : [NaN, NaN],
                            fromUuid: typeof l.source == "object" ? l.source.uuid : (l.source as string),
                            toUuid: typeof l.target == "object" ? l.target.uuid : (l.target as string),
                            className: defaultConnectionClass,
                        };
                        // XXX: swapping connection direction here for reverse connections
                        if (res.from[0] > res.to[0])
                            res = {
                                ...res,
                                from: [res.to[0] + treeNodeWidth, res.to[1]],
                                to: [res.from[0] - treeNodeWidth, res.from[1]],
                                fromUuid: res.toUuid,
                                toUuid: res.fromUuid,
                            };
                        return res;
                    }),
                );
            });

        return () => {
            console.log("end simulation");
            simulation.current?.stop();
            simulation.current = undefined;
        };
    }, []);

    useEffect(() => {
        const sim = simulation.current;
        if (!sim) throw new Error("simulation not ready?!");

        // recycle old nodes to preserve position and velocity.
        const old = new Map(simulationState.current.nodes.map((n) => [n.uuid, n]));
        const state = new Map(sim.nodes().map((n) => [n.uuid, n]));
        const uuids = roots.map((r) => r.uuid).join("\n");
        let nextY = 0;
        let forceRecalcX = simulationState.current.rootUuids != uuids;
        let nodes = roots.flatMap((root) =>
            flatMapTree<NodeT>(root, (n, d, ci, parent) => {
                const fx = d * (treeNodeWidth + horizontalMargin);
                let overrideY: number | undefined = undefined;
                if (parent) {
                    let parentNode = state.get(parent.uuid);
                    overrideY = parentNode?.y;
                    if (parentNode?.children) {
                        for (const child of parentNode.children) {
                            let siblingNode = state.get(child.uuid);
                            if (siblingNode && siblingNode.uuid != n.uuid) {
                                overrideY = siblingNode.y;
                            }
                        }
                    }
                }
                let res = {
                    fx,
                    y: (overrideY ?? (nextY += 20)) + Math.random() * 10,
                    ...old.get(n.uuid),
                    ...n,
                };
                if (forceRecalcX) res.fx = fx;
                return [res];
            }),
        );
        let links = roots.flatMap((root) =>
            flatMapTree<LinkT>(
                root,
                (n) =>
                    n.children?.flatMap<LinkT>((c) => ({
                        source: n.uuid,
                        target: c.uuid,
                        // source and target get replaced with actual objects by d3,
                        // so we add extra fields here to avoid needing to check
                        // if their type is object or string
                        sourceUuid: n.uuid,
                        targetUuid: c.uuid,
                    })) ?? [],
            ),
        );

        const linkForce = sim.force("link")! as d3.ForceLink<NodeT, LinkT>;

        sim.nodes(nodes);
        linkForce.links(links);
        simulationState.current.nodes = nodes;
        simulationState.current.links = links;
        simulationState.current.rootUuids = uuids;
        sim.force("centering", centeringForce);
        sim.alpha(0.4).restart().tick();

        if (forceRecalcX) {
            let viewportDiv = viewportRef.current?.getRootElement();
            if (viewportDiv) {
                for (let anim of viewportDiv.getAnimations()) {
                    anim.currentTime = 0;
                }
            }
        }
    }, [roots, simulation]);

    useEffect(() => {
        if (
            connections.some(
                (c) =>
                    (c.className == highlightConnectionClass) !=
                    (highlighted.includes(c.fromUuid) && highlighted.includes(c.toUuid)),
            )
        ) {
            setConnections((conns) =>
                conns.map((c) => ({
                    ...c,
                    className:
                        highlighted.includes(c.fromUuid) && highlighted.includes(c.toUuid)
                            ? highlightConnectionClass
                            : defaultConnectionClass,
                })),
            );
        }
    }, [connections, highlighted]);

    return (
        <Viewport
            {...props}
            originX={0.15}
            className={`
                workspace-findings-graph
                ${props.className ?? ""}
                ${highlighted.length > 0 ? "has-highlighted" : ""}
            `}
            ref={viewportRef}
            connections={connections}
        >
            {roots.flatMap((root) =>
                flatMapTree(root, (n) => [
                    <TreeNode
                        key={n.uuid}
                        node={n}
                        className={`${highlighted.includes(n.uuid) ? "highlighted" : ""}`}
                        onClickTag={onClickTag}
                        onPointerEnter={(e) => (e.altKey ? null : setHovered(n.uuid))}
                        onPointerLeave={() => unsetHovered(n.uuid)}
                    />,
                ]),
            )}
            {children}
        </Viewport>
    );
}

const TreeNode = forwardRef<
    HTMLDivElement,
    {
        node: TreeNode;
        className?: string;
        onClickTag?: TagClickCallback;
        onPointerEnter?: React.PointerEventHandler<HTMLDivElement>;
        onPointerLeave?: React.PointerEventHandler<HTMLDivElement>;
        onClick?: React.MouseEventHandler<HTMLDivElement>;
    }
>(({ node, onClickTag, className, onPointerEnter, onPointerLeave, onClick }, ref) => {
    function getSeverityColor(node: TreeNode): { color: string; backgroundColor: string } {
        if (node.type != "Finding") return { backgroundColor: "#00ccff40", color: "white" };
        switch (node.definition.severity) {
            case "Okay":
                return { backgroundColor: "#00ff8840", color: "white" };
            case "Low":
                return { backgroundColor: "#ffcc0040", color: "white" };
            case "Medium":
                return { backgroundColor: "#ff6a0040", color: "white" };
            case "High":
                return { backgroundColor: "#ff000040", color: "white" };
            case "Critical":
                return { backgroundColor: "#9900ff40", color: "white" };
            default:
                const exhaustiveCheck: never = node.definition.severity;
                throw new Error(`Unhandled severity: ${exhaustiveCheck}`);
        }
    }

    let name: string;
    let tags: SimpleTag[] | undefined;
    switch (node.type) {
        case "Finding":
            name = node.definition.name;
            tags = undefined;
            break;
        case "Domain":
            name = node.domain.domain;
            tags = node.domain.tags;
            break;
        case "Host":
            name = node.host.ipAddr;
            tags = node.host.tags;
            break;
        case "Port":
            name = "Port " + node.port.port;
            tags = node.port.tags;
            break;
        case "Service":
            name = "Service " + node.service.name;
            tags = node.service.tags;
            break;
        default:
            const exhaustiveCheck: never = node;
            throw new Error(`Unhandled node type: ${(exhaustiveCheck as any).type}`);
    }

    return (
        <div
            data-uuid={node.uuid}
            className={`tree-node ` + className}
            ref={ref}
            onPointerEnter={onPointerEnter}
            onPointerLeave={onPointerLeave}
            onClick={onClick}
        >
            <div className="tree-node-content">
                <div className="tree-node-heading" style={getSeverityColor(node)}>
                    <span>{name}</span>
                </div>
                <span className="tree-node-type">{node.type}</span>
                <div className="tree-node-body">
                    {node.type != "Finding" && <TagList tags={tags!} onClickTag={onClickTag} />}
                </div>
            </div>
        </div>
    );
});

function flatMapTree<T>(
    node: TreeNode,
    mapFn: (node: TreeNode, depth: number, childIndex: number, parent: TreeNode | undefined) => T[],
): T[] {
    const visited: { [uuid: string]: null } = {};
    const impl = (node: TreeNode, depth: number, childIndex: number, parent: TreeNode | undefined): T[] => {
        if (node.uuid in visited) return [];
        visited[node.uuid] = null;
        return [
            ...mapFn(node, depth, childIndex, parent),
            ...(node.children?.flatMap((n, i) => impl(n, depth + 1, i, node)) ?? []),
        ];
    };
    return impl(node, 0, 0, undefined);
}
