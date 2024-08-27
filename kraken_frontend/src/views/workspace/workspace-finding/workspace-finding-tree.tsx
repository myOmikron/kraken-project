import * as d3 from "d3";
import { useEffect, useRef, useState } from "react";
import { UUID } from "../../../api/api";
import {
    FindingSeverity,
    FullDomain,
    FullHost,
    FullPort,
    FullService,
    SimpleFindingDefinition,
    SimpleTag,
} from "../../../api/generated";
import { FullHttpService } from "../../../api/generated/models/FullHttpService";
import ContextMenu, { ContextMenuEntry } from "../components/context-menu";
import SeverityIcon from "../components/severity-icon";
import TagList, { TagClickCallback } from "../components/tag-list";
import { Viewport, ViewportProps, ViewportRef } from "../components/viewport";

export type TreeNode = {
    uuid: string;
    children?: Array<TreeNode>;
} & (
    | {
          definition: SimpleFindingDefinition;
          type: "Finding";
          severity: FindingSeverity;
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
          type: "HttpService";
          httpService: FullHttpService;
      }
    | {
          type: "Domain";
          domain: FullDomain;
      }
);

export function TreeGraph({
    roots,
    showTags,
    fastSimulation,
    onClickTag,
    children,
    getMenu,
    ...props
}: {
    roots: TreeNode[];
    showTags?: boolean;
    fastSimulation?: boolean;
    onClickTag?: TagClickCallback;
    getMenu?: (t: TreeNode) => ContextMenuEntry[] | undefined;
} & ViewportProps) {
    const verticalMargin = 16;
    const horizontalMargin = 64;

    type NodeT = d3.SimulationNodeDatum & {
        uuid: string;
        root: boolean;
        column: number;
        radius?: number;
        children?: { uuid: string }[];
    };
    type LinkT = d3.SimulationLinkDatum<NodeT> & { sourceUuid: string; targetUuid: string };
    type ConnectionT = {
        from: [number, number];
        to: [number, number];
        fromUuid: string;
        toUuid: string;
        className: string;
        reversed: boolean;
        finding: boolean;
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

    const unsetHovered = (uuid?: string) => {
        clearTimeout(hoverDebounceTimer.current);
        hoverDebounceTimer.current = setTimeout(function () {
            setHoveredUuid((prev) => {
                if (uuid === undefined || prev == uuid) {
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

    const centeringForce = d3.forceY<NodeT>(0).strength((n) => (n.root ? 0.05 : 0.01));

    useEffect(() => {
        simulation.current = d3
            .forceSimulation<NodeT, LinkT>()
            // .force("charge", d3.forceManyBody().strength(10))
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
                    .strength((l) => (roots.some((r) => r.uuid == l.source || l.target) ? 0.5 : 0.1)),
            )
            .force("centering", centeringForce)
            .on("tick", () => {
                const sim = simulation.current;
                if (!sim) return;
                for (const node of sim.nodes()) {
                    const elem = getElement(node.uuid);
                    if (elem) {
                        if (node.x !== undefined) elem.style.left = `${Math.round(node.x)}px`;
                        if (node.y !== undefined) elem.style.top = `${Math.round(node.y)}px`;
                    }
                }
                const linkForce = sim.force("link")! as d3.ForceLink<NodeT, LinkT>;
                const treeNodeWidth = window.innerWidth < 2000 ? 240 : 310;
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
                            reversed: false,
                            finding:
                                (typeof l.source == "object" ? l.source.root : false) ||
                                (typeof l.target == "object" ? l.target.root : false),
                            className: "",
                        };
                        // XXX: swapping connection direction here for reverse connections
                        if (res.from[0] > res.to[0])
                            res = {
                                ...res,
                                from: [res.to[0] + treeNodeWidth, res.to[1]],
                                to: [res.from[0] - treeNodeWidth, res.from[1]],
                                fromUuid: res.toUuid,
                                toUuid: res.fromUuid,
                                reversed: true,
                            };
                        res.className = generateConnectionClass(res);
                        return res;
                    }),
                );
            });

        return () => {
            simulation.current?.stop();
            simulation.current = undefined;
        };
    }, []);

    const rootUuids = roots.map((r) => r.uuid).join("\n");
    useEffect(() => {
        const sim = simulation.current;
        if (!sim) throw new Error("simulation not ready?!");

        // recycle old nodes to preserve position and velocity.
        const old = new Map(simulationState.current.nodes.map((n) => [n.uuid, n]));
        const state = new Map(sim.nodes().map((n) => [n.uuid, n]));
        const inserted: { [index: UUID]: NodeT } = {};
        const treeNodeWidth = window.innerWidth < 2000 ? 240 : 300;
        const columnW = treeNodeWidth + horizontalMargin;
        let nextY = 0;
        const forceRecalcX = simulationState.current.rootUuids != rootUuids;
        if (forceRecalcX) {
            old.forEach((v) => {
                v.column = 0;
                v.fx = undefined;
            });
        }
        const nodes = roots.flatMap((root) =>
            flatMapTree<NodeT>(root, (n, d, ci, parent) => {
                if (n.uuid in inserted) return [];
                let startColumn = 0;
                if (n.type == "Finding" && parent === undefined && n.children?.length) {
                    const c = n.children.map((c) => inserted[c.uuid]).find((a) => a);
                    if (c?.column) startColumn = c.column - 1;
                }
                const column = Math.max(
                    0,
                    startColumn,
                    (parent ? inserted[parent.uuid]?.column ?? 0 : 0) +
                        // try to align findings always left of nodes, so we add finding children of nodes left of them:
                        (n.type == "Finding" ? -1 : 1),
                );
                let overrideY: number | undefined = undefined;
                if (parent) {
                    const parentNode = state.get(parent.uuid);
                    overrideY = inserted[parent.uuid]?.y ?? parentNode?.y;
                    if (parentNode?.children) {
                        for (const child of parentNode.children) {
                            const siblingNode = state.get(child.uuid);
                            if (siblingNode && siblingNode.uuid != n.uuid) {
                                overrideY = siblingNode.y;
                            }
                        }
                    }
                }
                const res = {
                    root: d == 0,
                    y: (overrideY ?? (nextY += 20)) + (ci / (parent?.children?.length || 1)) * 19,
                    ...old.get(n.uuid),
                    column: column,
                    fx: column * columnW,
                    ...n,
                };
                inserted[n.uuid] = res;
                return [res];
            }),
        );
        const links = roots.flatMap((root) =>
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

        {
            const contentColumns = new Set(nodes.filter((n) => !n.root).map((n) => n.column));
            const rootColumns = new Set(nodes.filter((n) => n.root).map((n) => n.column));
            const mixedColumns: number[] = [];
            for (const c of contentColumns.values()) {
                if (rootColumns.has(c)) mixedColumns.push(c);
            }
            mixedColumns.sort((a, b) => b - a);
            let maxColumn = 0;
            for (const c of mixedColumns) {
                for (const node of nodes) {
                    if (node.column > c) {
                        node.column += 2;
                        node.fx = node.column * columnW;
                    } else if (node.column == c) {
                        node.column += node.root ? 1 : 2;
                        node.fx = node.column * columnW;
                    }
                    maxColumn = Math.max(maxColumn, node.column);
                }
            }
            const usedColumns = new Set(nodes.map((n) => n.column));
            const emptyColumns: number[] = [];
            for (let i = maxColumn; i >= 0; i--) {
                if (!usedColumns.has(i)) emptyColumns.push(i);
            }
            for (const c of emptyColumns) {
                for (const node of nodes) {
                    if (node.column > c) {
                        node.column--;
                        node.fx = node.column * columnW;
                    }
                }
            }
        }
        {
            const rootColumns = new Set(nodes.filter((n) => n.root).map((n) => n.column));
            for (const c of Array.from(rootColumns.values()).sort((a, b) => b - a)) {
                if (rootColumns.has(c - 1)) {
                    // neighboring root columns, merge
                    for (const node of nodes) {
                        if (node.column >= c) {
                            node.column--;
                            node.fx = node.column * columnW;
                        }
                    }
                }
            }
        }

        sim.nodes(nodes);
        linkForce.links(links);
        simulationState.current.nodes = nodes;
        simulationState.current.links = links;
        simulationState.current.rootUuids = rootUuids;
        sim.force("centering", null);
        const collisionForce = sim.force("collide")!;
        sim.force("collide", null);
        sim.alpha(0.4).restart().tick();
        if (fastSimulation ?? true) {
            for (let i = 0; i < 400; i++) {
                if (sim.alpha() < sim.alphaMin()) break;
                sim.tick();
            }
        }
        sim.force("collide", collisionForce);
        sim.force("centering", centeringForce);
        sim.alpha(0.4).restart().tick();
        if (fastSimulation ?? true) {
            for (let i = 0; i < 1000; i++) {
                if (sim.alpha() < sim.alphaMin()) break;
                sim.tick();
            }
        }
        for (const node of nodes) {
            if (node.root && node.children?.length) {
                node.y =
                    node.children.map((c) => inserted[c.uuid].y ?? 0).reduce((a, b) => a + b) / node.children.length;
                node.vy = 0;
            }
        }
        sim.alpha(0.01).restart().tick();

        if (forceRecalcX) {
            const viewportDiv = viewportRef.current?.getRootElement();
            if (viewportDiv) {
                for (const anim of viewportDiv.getAnimations()) {
                    anim.currentTime = 0;
                }
            }
        }
    }, [roots, rootUuids, simulation, showTags, fastSimulation]);

    const generateConnectionClass = (c: ConnectionT): string => `
        ${highlighted.includes(c.fromUuid) && highlighted.includes(c.toUuid) ? highlightConnectionClass : defaultConnectionClass}
        ${c.reversed ? "reversed" : ""}
        ${c.finding ? "finding" : ""}
    `;

    useEffect(() => {
        if (connections.some((c) => c.className != generateConnectionClass(c))) {
            setConnections((conns) =>
                conns.map((c) => ({
                    ...c,
                    className: generateConnectionClass(c),
                })),
            );
        }
    }, [connections, highlighted]);

    const rendered: { [index: UUID]: React.ReactNode[] } = {};
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
            onMouseMoveBackground={() => {
                unsetHovered();
            }}
        >
            {roots.flatMap((root) =>
                flatMapTree(root, (n) =>
                    n.uuid in rendered
                        ? []
                        : (rendered[n.uuid] = [
                              <TreeNode
                                  showTags={showTags ?? true}
                                  key={n.uuid}
                                  menu={getMenu?.(n)}
                                  node={n}
                                  className={`${highlighted.includes(n.uuid) ? "highlighted" : ""}`}
                                  onClickTag={onClickTag}
                                  onPointerEnter={(e) => (e.altKey ? null : setHovered(n.uuid))}
                                  onPointerLeave={() => unsetHovered(n.uuid)}
                              />,
                          ]),
                ),
            )}
            {children}
        </Viewport>
    );
}

function TreeNode({
    node,
    showTags,
    menu,
    onClickTag,
    className,
    onPointerEnter,
    onPointerLeave,
    onClick,
}: {
    node: TreeNode;
    showTags: boolean;
    menu?: ContextMenuEntry[];
    className?: string;
    onClickTag?: TagClickCallback;
    onPointerEnter?: React.PointerEventHandler<HTMLDivElement>;
    onPointerLeave?: React.PointerEventHandler<HTMLDivElement>;
    onClick?: React.MouseEventHandler<HTMLDivElement>;
}) {
    let name: string;
    let cve: string | null | undefined;
    let tags: SimpleTag[] | undefined;
    switch (node.type) {
        case "Finding":
            name = node.definition.name;
            cve = node.definition.cve;
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
        case "HttpService":
            name = "HTTP Service " + node.httpService.name;
            tags = node.httpService.tags;
            break;
    }

    return (
        <ContextMenu
            data-uuid={node.uuid}
            className={`
                tree-node
                node-${node.type}
                ${node.type === "Finding" ? "severity-" + node.severity : ""}
                ${className}
            `}
            onPointerEnter={onPointerEnter}
            onPointerLeave={onPointerLeave}
            onClick={onClick}
            menu={menu}
        >
            <div className="tree-node-content">
                <div className={`tree-node-heading ${cve ? "with-cve" : ""}`}>
                    <span>{name}</span>
                    {cve && <span>{cve}</span>}
                </div>
                {node.type == "Finding" ? (
                    <div className="tree-node-body">
                        <SeverityIcon severity={node.severity} tooltip={false} />
                        Severity: <b>{node.severity}</b>
                    </div>
                ) : (
                    <div className={`tree-node-body ${!showTags || tags!.length == 0 ? "empty" : ""}`}>
                        {showTags && <TagList tags={tags!} onClickTag={onClickTag} />}
                    </div>
                )}
            </div>
        </ContextMenu>
    );
}

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
