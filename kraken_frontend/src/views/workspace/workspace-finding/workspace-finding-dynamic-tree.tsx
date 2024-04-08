import React, { forwardRef, useEffect, useImperativeHandle, useRef, useState } from "react";
import Popup from "reactjs-popup";
import { Api, UUID } from "../../../api/api";
import { ApiError } from "../../../api/error";
import {
    AggregationType,
    DomainRelations,
    FullDomain,
    FullFinding,
    FullFindingDefinition,
    FullHost,
    FullPort,
    FullService,
    HostRelations,
    ListFindings,
    PortRelations,
    ServiceRelations,
    SimpleDomain,
    SimpleFindingAffected,
    SimpleHost,
    SimplePort,
    SimpleService,
    SimpleTag,
    TagType,
} from "../../../api/generated";
import { FullHttpService } from "../../../api/generated/models/FullHttpService";
import { HttpServiceRelations } from "../../../api/generated/models/HttpServiceRelations";
import { SimpleHttpService } from "../../../api/generated/models/SimpleHttpService";
import Checkbox from "../../../components/checkbox";
import Input from "../../../components/input";
import { ROUTES } from "../../../routes";
import CollapseIcon from "../../../svg/collapse";
import ExpandIcon from "../../../svg/expand";
import InformationIcon from "../../../svg/information";
import LinkIcon from "../../../svg/link";
import SettingsIcon from "../../../svg/settings";
import TagIcon from "../../../svg/tag";
import { handleApiError } from "../../../utils/helper";
import { Result } from "../../../utils/result";
import CollapsibleSection from "../components/collapsible-section";
import { ContextMenuEntry } from "../components/context-menu";
import Domain from "../components/domain";
import EditableTags from "../components/editable-tags";
import IpAddr from "../components/host";
import HttpServiceName from "../components/http-service";
import PortNumber from "../components/port";
import ServiceName from "../components/service";
import TagList from "../components/tag-list";
import { ViewportProps } from "../components/viewport";
import { WORKSPACE_CONTEXT } from "../workspace";
import WorkspaceFindingsQuickAttach from "../workspace-findings-quick-attach";
import { CreateFindingObject, getCreateAffectedData, getCreateAffectedType } from "./workspace-create-finding";
import { TreeGraph, TreeNode } from "./workspace-finding-tree";

export type DynamicTreeGraphProps = {
    maximizable?: boolean;
} & ({ api: DynamicTreeLookupFunctions } | { workspace: string; uuids: string[] }) &
    ViewportProps;

export type DynamicTreeWorkspaceFunctions = {
    resolveFinding: (findingDefinition: string) => Promise<Result<FullFindingDefinition, ApiError>>;
    findingsForDomain: (domainUuid: string) => Promise<Result<ListFindings, ApiError>>;
    findingsForHost: (hostUuid: string) => Promise<Result<ListFindings, ApiError>>;
    findingsForService: (serviceUuid: string) => Promise<Result<ListFindings, ApiError>>;
    findingsForHttpService: (httpServiceUuid: string) => Promise<Result<ListFindings, ApiError>>;
    findingsForPort: (portUuid: string) => Promise<Result<ListFindings, ApiError>>;
    relationsForDomain: (domainUuid: string) => Promise<Result<DomainRelations, ApiError>>;
    relationsForHost: (hostUuid: string) => Promise<Result<HostRelations, ApiError>>;
    relationsForService: (serviceUuid: string) => Promise<Result<ServiceRelations, ApiError>>;
    relationsForHttpService: (httpServiceUuid: string) => Promise<Result<HttpServiceRelations, ApiError>>;
    relationsForPort: (portUuid: string) => Promise<Result<PortRelations, ApiError>>;
    resolveDomain: (domainUuid: string) => Promise<Result<FullDomain, ApiError>>;
    resolveHost: (hostUuid: string) => Promise<Result<FullHost, ApiError>>;
    resolveService: (serviceUuid: string) => Promise<Result<FullService, ApiError>>;
    resolveHttpService: (httpServiceUuid: string) => Promise<Result<FullHttpService, ApiError>>;
    resolvePort: (portUuid: string) => Promise<Result<FullPort, ApiError>>;
};

export type AffectedShallow =
    | {
          domain: { uuid: string };
      }
    | {
          port: { uuid: string };
      }
    | {
          host: { uuid: string };
      }
    | {
          service: { uuid: string };
      }
    | {
          httpService: { uuid: string };
      };

export type DynamicTreeLookupFunctions = {
    getRoots: () => Promise<FullFinding[]>;
    getAffected: (
        finding: FullFinding,
        affected: SimpleFindingAffected,
    ) => Promise<Result<{ affected: AffectedShallow }, ApiError>>;
} & DynamicTreeWorkspaceFunctions;

export function treeLookupFunctionsWorkspace(workspace: string): DynamicTreeWorkspaceFunctions {
    return {
        resolveFinding: Api.knowledgeBase.findingDefinitions.get,
        findingsForDomain: (d) => Api.workspaces.domains.findings(workspace, d),
        findingsForHost: (d) => Api.workspaces.hosts.findings(workspace, d),
        findingsForService: (d) => Api.workspaces.services.findings(workspace, d),
        findingsForHttpService: (d) => Api.workspaces.httpServices.findings(workspace, d),
        findingsForPort: (d) => Api.workspaces.ports.findings(workspace, d),
        relationsForDomain: (d) => Api.workspaces.domains.relations(workspace, d),
        relationsForHost: (d) => Api.workspaces.hosts.relations(workspace, d),
        relationsForService: (d) => Api.workspaces.services.relations(workspace, d),
        relationsForHttpService: (d) => Api.workspaces.httpServices.relations(workspace, d),
        relationsForPort: (d) => Api.workspaces.ports.relations(workspace, d),
        resolveDomain: (d) => Api.workspaces.domains.get(workspace, d),
        resolveHost: (d) => Api.workspaces.hosts.get(workspace, d),
        resolveService: (d) => Api.workspaces.services.get(workspace, d),
        resolveHttpService: (d) => Api.workspaces.httpServices.get(workspace, d),
        resolvePort: (d) => Api.workspaces.ports.get(workspace, d),
    };
}

export function treeLookupFunctionsRoot(workspace: string, uuids: string[]): DynamicTreeLookupFunctions {
    return {
        getRoots: () =>
            Promise.all(
                uuids.map(
                    (uuid) =>
                        new Promise<FullFinding>((resolve) =>
                            Api.workspaces.findings.get(workspace, uuid).then(handleApiError(resolve)),
                        ),
                ),
            ),
        getAffected: ({ uuid }: FullFinding, { affectedUuid: affected }: SimpleFindingAffected) =>
            Api.workspaces.findings.getAffected(workspace, uuid, affected),
        ...treeLookupFunctionsWorkspace(workspace),
    };
}

export type DynamicTreeGraphRef = {
    reloadAffected(): void;
    reloadRoot(): void;
    addTag(value: SimpleTag, negate: boolean): void;
};

type DynamicTreeNode = {
    _searchIndex: string;
    children?: Array<DynamicTreeNode>;
} & TreeNode;

export const DynamicTreeGraph = forwardRef<DynamicTreeGraphRef, DynamicTreeGraphProps>(
    ({ maximizable, ...props }, ref) => {
        const apiTimeout = 100;

        const api = "api" in props ? props.api : treeLookupFunctionsRoot(props.workspace, props.uuids);

        const [maximized, setMaximized] = useState(false);
        const [manualAffected, setManualAffected] = useState(0);
        const [roots, setRoots] = useState<DynamicTreeNode[]>([]);
        const [shownRoots, setShownRoots] = useState<DynamicTreeNode[]>([]);
        const [wantRerender, setWantRerender] = React.useState(false);
        const [filterTags, setFilterTags] = useState<SimpleTag[]>([]);
        const addedUuids = useRef<{ [index: string]: null }>({});
        const [filterText, setFilterText] = useState("");
        const [attaching, setAttaching] = useState<CreateFindingObject>();
        const [tagging, setTagging] = useState<CreateFindingObject>();
        const [showTags, setShowTags] = React.useState(true);
        const [fastSimulation, setFastSimulation] = React.useState(true);
        const {
            workspace: { uuid: contextWorkspace },
        } = React.useContext(WORKSPACE_CONTEXT);

        const getMutator = () => {
            const srcUuid = apiOrUuid;
            const srcWorkspace = apiOrWorkspace;
            const srcManualAffected = manualAffected;

            const mutator = {
                shouldAbort() {
                    return (
                        srcUuid != apiOrUuid || srcWorkspace != apiOrWorkspace || srcManualAffected != manualAffected
                    );
                },
                /// inserts the child to parent, returns true if the child hasn't been added to the tree yet
                insertChild(parent: DynamicTreeNode, child: DynamicTreeNode) {
                    const isNew = !(child.uuid in addedUuids.current);
                    addedUuids.current[child.uuid] = null;
                    mutator.updateNode(parent, (n) => ({
                        ...n,
                        children: [...(n.children ?? []), child],
                    }));
                    return isNew;
                },
                updateNode(search: string | DynamicTreeNode, update: (node: DynamicTreeNode) => DynamicTreeNode) {
                    const uuid = typeof search == "string" ? search : search.uuid;
                    let found = false;
                    let copied: { [uuid: string]: DynamicTreeNode } = {};

                    function mutate(n: DynamicTreeNode): DynamicTreeNode {
                        if (copied[n.uuid]) return copied[n.uuid];
                        if (n.uuid == uuid) {
                            found = true;
                            return (copied[n.uuid] = update(n));
                        } else {
                            const copy: DynamicTreeNode = {
                                ...n,
                            };
                            copied[n.uuid] = copy;
                            copy.children = n.children?.map(mutate);
                            return copy;
                        }
                    }

                    setRoots((r) => {
                        const res = r.map((r) => {
                            copied = {};
                            return mutate(r);
                        });
                        if (!found) console.warn("couldn't find ", search, " in roots for update!");
                        return res;
                    });
                },
                insertFindings(parent: DynamicTreeNode, node: DynamicTreeNode, findings: ListFindings) {
                    if (mutator.shouldAbort()) return;
                    for (const f of findings.findings) {
                        if (f.uuid == parent.uuid) continue;
                        api.resolveFinding(f.definition).then(
                            handleApiError((definition) => {
                                if (mutator.shouldAbort()) return;
                                const insert: DynamicTreeNode = {
                                    uuid: f.uuid,
                                    type: "Finding",
                                    definition: definition,
                                    severity: f.severity,
                                    _searchIndex:
                                        "finding " +
                                        (
                                            definition.name + (definition.cve ? ` [${definition.cve}]` : "")
                                        ).toLowerCase(),
                                };
                                if (!mutator.insertChild(node, insert)) return;
                                // TODO: recurse here
                            }),
                        );
                    }
                },
                populateDomain(parent: DynamicTreeNode, node: DynamicTreeNode, domain: SimpleDomain | FullDomain) {
                    api.findingsForDomain(domain.uuid).then(
                        handleApiError((findings) => mutator.insertFindings(parent, node, findings)),
                    );
                    api.relationsForDomain(domain.uuid).then(
                        handleApiError((relations) => {
                            [...relations.directHosts, ...relations.indirectHosts].forEach((c) =>
                                mutator.insertHostSimple(node, c),
                            );
                            [...relations.sourceDomains, ...relations.targetDomains].forEach((c) =>
                                mutator.insertDomainSimple(node, c),
                            );
                            relations.httpServices.forEach((c) => mutator.insertHttpServiceSimple(node, c));
                        }),
                    );
                },
                populateHost(parent: DynamicTreeNode, node: DynamicTreeNode, host: SimpleHost | FullHost) {
                    api.findingsForHost(host.uuid).then(
                        handleApiError((findings) => mutator.insertFindings(parent, node, findings)),
                    );
                    api.relationsForHost(host.uuid).then(
                        handleApiError((relations) => {
                            [...relations.directDomains, ...relations.indirectDomains].forEach((c) =>
                                mutator.insertDomainSimple(node, c),
                            );
                            relations.ports.forEach((c) => mutator.insertPortSimple(node, c));
                            relations.services
                                .filter((s) => !s.port || !relations.ports.some((rp) => rp.uuid == s.port))
                                .forEach((c) => mutator.insertServiceSimple(node, c));
                            relations.httpServices
                                .filter((s) => !s.port || !relations.ports.some((rp) => rp.uuid == s.port))
                                .forEach((c) => mutator.insertHttpServiceSimple(node, c));
                        }),
                    );
                },
                populatePort(parent: DynamicTreeNode, node: DynamicTreeNode, port: SimplePort | FullPort) {
                    api.findingsForPort(port.uuid).then(
                        handleApiError((findings) => mutator.insertFindings(parent, node, findings)),
                    );
                    api.relationsForPort(port.uuid).then(
                        handleApiError((relations) => {
                            mutator.insertHostSimple(node, relations.host);
                            relations.services.forEach((c) => mutator.insertServiceSimple(node, c));
                            relations.httpServices.forEach((c) => mutator.insertHttpServiceSimple(node, c));
                        }),
                    );
                },
                populateService(parent: DynamicTreeNode, node: DynamicTreeNode, service: SimpleService | FullService) {
                    api.findingsForService(service.uuid).then(
                        handleApiError((findings) => mutator.insertFindings(parent, node, findings)),
                    );
                    api.relationsForService(service.uuid).then(
                        handleApiError((relations) => {
                            if (relations.port && relations.port.host == relations.host.uuid) {
                                // don't make graph cyclic here
                            } else mutator.insertHostSimple(node, relations.host);
                            if (relations.port) mutator.insertPortSimple(node, relations.port);
                        }),
                    );
                },
                populateHttpService(
                    parent: DynamicTreeNode,
                    node: DynamicTreeNode,
                    httpService: SimpleHttpService | FullHttpService,
                ) {
                    api.findingsForHttpService(httpService.uuid).then(
                        handleApiError((findings) => mutator.insertFindings(parent, node, findings)),
                    );
                    api.relationsForHttpService(httpService.uuid).then(
                        handleApiError((relations) => {
                            mutator.insertPortSimple(node, relations.port);
                            mutator.insertHostSimple(node, relations.host);
                            // TODO: probably need to add an ignore condition w.r.t. the domain, so we don't get extra cyclic graphs
                            if (relations.domain) mutator.insertDomainSimple(node, relations.domain);
                        }),
                    );
                },
                insertDomain(node: DynamicTreeNode, child: FullDomain) {
                    if (mutator.shouldAbort()) return;
                    const insert: DynamicTreeNode = {
                        type: "Domain",
                        domain: child,
                        uuid: child.uuid,
                        _searchIndex: child.domain.toLowerCase(),
                    };
                    if (!mutator.insertChild(node, insert)) return;
                    setTimeout(function () {
                        mutator.populateDomain(node, insert, child);
                    }, apiTimeout);
                },
                insertHost(node: DynamicTreeNode, child: FullHost) {
                    if (mutator.shouldAbort()) return;
                    const insert: DynamicTreeNode = {
                        type: "Host",
                        host: child,
                        uuid: child.uuid,
                        _searchIndex: child.ipAddr.toLowerCase(),
                    };
                    if (!mutator.insertChild(node, insert)) return;
                    setTimeout(function () {
                        mutator.populateHost(node, insert, child);
                    }, apiTimeout);
                },
                insertPort(node: DynamicTreeNode, child: FullPort) {
                    if (mutator.shouldAbort()) return;
                    const insert: DynamicTreeNode = {
                        type: "Port",
                        port: child,
                        uuid: child.uuid,
                        _searchIndex: "port " + child.port,
                    };
                    if (!mutator.insertChild(node, insert)) return;
                    setTimeout(function () {
                        mutator.populatePort(node, insert, child);
                    }, apiTimeout);
                },
                insertService(node: DynamicTreeNode, child: FullService) {
                    if (mutator.shouldAbort()) return;
                    const insert: DynamicTreeNode = {
                        type: "Service",
                        service: child,
                        uuid: child.uuid,
                        _searchIndex: "service " + child.name.toLowerCase(),
                    };
                    if (!mutator.insertChild(node, insert)) return;
                    setTimeout(function () {
                        mutator.populateService(node, insert, child);
                    }, apiTimeout);
                },
                insertHttpService(node: DynamicTreeNode, child: FullHttpService) {
                    if (mutator.shouldAbort()) return;
                    const insert: DynamicTreeNode = {
                        type: "HttpService",
                        httpService: child,
                        uuid: child.uuid,
                        _searchIndex: "service " + child.name.toLowerCase(),
                    };
                    if (!mutator.insertChild(node, insert)) return;
                    setTimeout(function () {
                        mutator.populateHttpService(node, insert, child);
                    }, apiTimeout);
                },
                insertDomainSimple(node: DynamicTreeNode, child: { uuid: string }) {
                    api.resolveDomain(child.uuid).then(handleApiError((full) => mutator.insertDomain(node, full)));
                },
                insertHostSimple(node: DynamicTreeNode, child: { uuid: string }) {
                    api.resolveHost(child.uuid).then(handleApiError((full) => mutator.insertHost(node, full)));
                },
                insertPortSimple(node: DynamicTreeNode, child: { uuid: string }) {
                    api.resolvePort(child.uuid).then(handleApiError((full) => mutator.insertPort(node, full)));
                },
                insertServiceSimple(node: DynamicTreeNode, child: { uuid: string }) {
                    api.resolveService(child.uuid).then(handleApiError((full) => mutator.insertService(node, full)));
                },
                insertHttpServiceSimple(node: DynamicTreeNode, child: { uuid: string }) {
                    api.resolveHttpService(child.uuid).then(
                        handleApiError((full) => mutator.insertHttpService(node, full)),
                    );
                },
                populateAffectedRoot(root: DynamicTreeNode, finding: FullFinding) {
                    if (mutator.shouldAbort()) return;
                    for (const a of finding.affected) {
                        api.getAffected(finding, a).then(
                            handleApiError((affected) => {
                                if (mutator.shouldAbort()) return;
                                if ("domain" in affected.affected && affected.affected.domain) {
                                    mutator.insertDomainSimple(root, affected.affected.domain);
                                } else if ("host" in affected.affected && affected.affected.host) {
                                    mutator.insertHostSimple(root, affected.affected.host);
                                } else if ("port" in affected.affected && affected.affected.port) {
                                    mutator.insertPortSimple(root, affected.affected.port);
                                } else if ("service" in affected.affected && affected.affected.service) {
                                    mutator.insertServiceSimple(root, affected.affected.service);
                                } else if ("httpService" in affected.affected && affected.affected.httpService) {
                                    mutator.insertServiceSimple(root, affected.affected.httpService);
                                }
                            }),
                        );
                    }
                },
            };
            return mutator;
        };

        useImperativeHandle(ref, () => ({
            reloadAffected() {
                setManualAffected((v) => v + 1);
                api.getRoots().then((findingsList) => {
                    const findings = Object.fromEntries(findingsList.map((f) => [f.uuid, f]));
                    setRoots((roots) =>
                        roots.map((root) => ({
                            ...root,
                            children: root.children
                                ?.filter((a) => findings[root.uuid].affected.some((f) => f.affectedUuid == a.uuid))
                                .map((c) => ({
                                    ...c,
                                })),
                        })),
                    );
                });
            },
            reloadRoot() {
                api.getRoots().then((findingsList) => {
                    setRoots((roots) =>
                        roots.map<DynamicTreeNode>((root) => {
                            const finding = findingsList.find((f) => f.uuid == root.uuid);
                            if (!finding) return root;
                            return {
                                uuid: root.uuid,
                                type: "Finding",
                                _searchIndex:
                                    "finding " +
                                    (
                                        finding.definition.name +
                                        (finding.definition.cve ? ` [${finding.definition.cve}]` : "")
                                    ).toLowerCase(),
                                definition: finding.definition,
                                severity: finding.severity,
                                children: root.children,
                            };
                        }),
                    );
                });
            },
            addTag(value, negate) {
                const existing = filterTags.findIndex((t) => t.uuid == value.uuid);
                if (negate && existing != -1)
                    setFilterTags((tags) => {
                        const existing = tags.findIndex((t) => t.uuid == value.uuid);
                        if (existing == -1) return tags;
                        const copy = [...tags];
                        copy.splice(existing, 1);
                        return copy;
                    });
                else if (!negate && existing == -1) {
                    setFilterTags((tags) => {
                        const existing = tags.findIndex((t) => t.uuid == value.uuid);
                        if (existing != -1) return tags;
                        return [...tags, value];
                    });
                }
            },
        }));

        const toggleMax = () => setMaximized((m) => !m);

        const apiOrWorkspace = "api" in props ? props.api : props.workspace;
        const apiOrUuid = "api" in props ? props.api : props.uuids.join("+"); // dependency cache index (just join strings)

        useEffect(() => {
            setRoots([]);
            addedUuids.current = {};

            const mutator = getMutator();

            api.getRoots().then((findings): void => {
                if (mutator.shouldAbort()) return;
                const roots = [];
                for (const finding of findings) {
                    const root: DynamicTreeNode = {
                        uuid: finding.uuid,
                        type: "Finding",
                        _searchIndex: "",
                        definition: finding.definition,
                        severity: finding.severity,
                    };
                    roots.push(root);
                    setTimeout(function () {
                        mutator.populateAffectedRoot(root, finding);
                    }, apiTimeout);
                }
                setRoots(roots);
            });
        }, [apiOrUuid, apiOrWorkspace]);

        function checkTag(node: TreeNode) {
            switch (node.type) {
                case "Domain":
                    return filterTags.every((expect) => node.domain.tags.some((t) => expect.uuid == t.uuid));
                case "Host":
                    return filterTags.every((expect) => node.host.tags.some((t) => expect.uuid == t.uuid));
                case "Port":
                    return filterTags.every((expect) => node.port.tags.some((t) => expect.uuid == t.uuid));
                case "Service":
                    return filterTags.every((expect) => node.service.tags.some((t) => expect.uuid == t.uuid));
                case "HttpService":
                    return filterTags.every((expect) => node.httpService.tags.some((t) => expect.uuid == t.uuid));
                case "Finding":
                    return true;
            }
        }

        useEffect(() => {
            const lowerCaseFilterText = filterText?.toLowerCase() ?? "";
            const toShow =
                filterTags.length || lowerCaseFilterText != ""
                    ? roots.map((root) => {
                          // TODO: performance optimize
                          type T = DynamicTreeNode & { _hit: boolean };
                          const visited: { [uuid: string]: T } = {};

                          function markTagged(n: DynamicTreeNode, parents: T[]): T {
                              if (visited[n.uuid]) return visited[n.uuid];
                              const v: T = (visited[n.uuid] = {
                                  ...n,
                                  _hit:
                                      (filterTags.length && checkTag(n)) ||
                                      (lowerCaseFilterText != "" && n._searchIndex.includes(lowerCaseFilterText)),
                              });
                              if (v._hit) for (const p of parents) p._hit = true;
                              v.children = v.children?.map((c) => markTagged(c, [...parents, v]));
                              return v;
                          }

                          const tagged = markTagged(root, []);
                          const visited2: { [uuid: string]: DynamicTreeNode } = {};

                          function filterTagged(n: T): DynamicTreeNode {
                              if (visited2[n.uuid]) return visited2[n.uuid];
                              if (!("_hit" in n)) return n;
                              const { _hit, ...node } = n;
                              visited2[n.uuid] = node;
                              node.children = node.children
                                  ?.filter((v) => (v as T)._hit)
                                  .map((v) => filterTagged(v as T));
                              return node;
                          }

                          return filterTagged(tagged);
                      })
                    : roots;

            function isIdentical(
                shown: DynamicTreeNode,
                check: DynamicTreeNode,
                checked: Record<string, true>,
            ): boolean {
                if (shown == check) return true;
                if (typeof shown != typeof check) return false;
                if (checked[check.uuid]) return true;
                checked[check.uuid] = true;
                return (
                    shown.uuid == check.uuid &&
                    shown.children?.length == check.children?.length &&
                    (shown.children?.every(
                        (c, i) => check.children?.[i] && isIdentical(c, check.children[i], checked),
                    ) ??
                        true)
                );
            }

            if (wantRerender || toShow.some((r, i) => !isIdentical(r, shownRoots[i], {}))) {
                setShownRoots(toShow);
                setWantRerender(false);
            }
        }, [roots, shownRoots, filterText, filterTags, wantRerender]);

        return (
            <>
                <TreeGraph
                    className={`${maximized ? "maximized" : ""}`}
                    onClickTag={(e, tag) => {
                        if (e.altKey) setFilterTags((tags) => tags.filter((t) => t.uuid != tag.uuid));
                        else setFilterTags((tags) => (tags.some((t) => t.uuid == tag.uuid) ? tags : [...tags, tag]));
                    }}
                    showTags={showTags}
                    fastSimulation={fastSimulation}
                    roots={shownRoots}
                    decoration={
                        <>
                            <div className="toolbar">
                                <Popup
                                    trigger={
                                        <button
                                            className="icon-button sidebar"
                                            aria-label={"Show Graph Options"}
                                            title={"Show Graph Options"}
                                        >
                                            <SettingsIcon />
                                        </button>
                                    }
                                    on={["click"]}
                                    position={["bottom left", "right center"]}
                                >
                                    <div className="sidebar-popup pane-thin">
                                        <CollapsibleSection summary={"Display"} defaultVisible>
                                            <label>
                                                <Checkbox value={showTags} onChange={(v) => setShowTags(v)} />
                                                Show tags
                                            </label>
                                        </CollapsibleSection>
                                        <CollapsibleSection summary={"Physics"} defaultVisible>
                                            <label>
                                                <Checkbox
                                                    value={fastSimulation}
                                                    onChange={(v) => setFastSimulation(v)}
                                                />
                                                Speed up simulation
                                            </label>
                                        </CollapsibleSection>
                                    </div>
                                </Popup>
                                <TagList
                                    tags={filterTags}
                                    onClickTag={(e, tag) => {
                                        setFilterTags((tags) => tags.filter((t) => t.uuid != tag.uuid));
                                    }}
                                />
                                <Input
                                    className=""
                                    placeholder="Search"
                                    value={filterText}
                                    onChange={(v) => setFilterText(v)}
                                />
                                <div className="pad"></div>
                                {maximizable && (
                                    <button
                                        className="icon-button maximize"
                                        onClick={toggleMax}
                                        aria-label={maximized ? "Minimize" : "Maximize"}
                                        title={maximized ? "Minimize" : "Maximize"}
                                    >
                                        {maximized ? <CollapseIcon /> : <ExpandIcon />}
                                    </button>
                                )}
                            </div>
                        </>
                    }
                    getMenu={(item) => [
                        (async (): Promise<ContextMenuEntry[]> => {
                            switch (item.type) {
                                case "Finding":
                                    return [
                                        [
                                            <>
                                                <InformationIcon />
                                                {item.definition.name}
                                                {" - "}
                                                {item.definition.cve}
                                            </>,
                                            undefined,
                                        ],
                                    ];
                                case "Domain":
                                    return [
                                        [
                                            <>
                                                <InformationIcon />
                                                <Domain domain={item.domain} pretty />
                                            </>,
                                            undefined,
                                        ],
                                    ];
                                case "Host":
                                    return [
                                        [
                                            <>
                                                <InformationIcon />
                                                <IpAddr host={item.host} pretty />
                                            </>,
                                            undefined,
                                        ],
                                    ];
                                case "Port":
                                    return [
                                        [
                                            <>
                                                <InformationIcon />
                                                <PortNumber port={item.port} pretty />
                                            </>,
                                            undefined,
                                        ],
                                    ];
                                case "Service":
                                    return [
                                        [
                                            <>
                                                <InformationIcon />
                                                <ServiceName service={item.service} pretty />
                                            </>,
                                            undefined,
                                        ],
                                    ];
                                case "HttpService":
                                    return [
                                        [
                                            <>
                                                <InformationIcon />
                                                <HttpServiceName httpService={item.httpService} pretty />
                                            </>,
                                            undefined,
                                        ],
                                    ];
                                default:
                                    const exhaustiveCheck: never = item;
                                    throw new Error(`Unhandled node type: ${(exhaustiveCheck as TreeNode).type}`);
                            }
                        }) satisfies ContextMenuEntry,
                        ...(item.type === "Finding"
                            ? []
                            : ([
                                  [
                                      <>
                                          <LinkIcon />
                                          <span>Attach to finding...</span>
                                      </>,
                                      async (e) => {
                                          if (e.ctrlKey)
                                              ROUTES.WORKSPACE_FINDINGS_QUICK_ATTACH.open({
                                                  workspace: contextWorkspace,
                                                  type: item.type,
                                                  uuid: item.uuid,
                                              });
                                          else setAttaching(await resolveItem(contextWorkspace, item.type, item.uuid));
                                      },
                                  ],
                                  [
                                      <>
                                          <TagIcon />
                                          <span>Manage tags...</span>
                                      </>,
                                      async () => {
                                          setTagging(await resolveItem(contextWorkspace, item.type, item.uuid));
                                      },
                                  ],
                              ] satisfies ContextMenuEntry[])),
                    ]}
                    {...props}
                />
                {attaching && (
                    <Popup nested modal open onClose={() => setAttaching(undefined)}>
                        <div className="pane-thin">
                            <WorkspaceFindingsQuickAttach
                                type={getCreateAffectedType(attaching)}
                                data={getCreateAffectedData(attaching)}
                                onAttached={(f, wantMore) => {
                                    if (!wantMore) setAttaching(undefined);
                                }}
                            />
                        </div>
                    </Popup>
                )}
                {tagging && (
                    <Popup nested modal open onClose={() => setTagging(undefined)}>
                        <div className="popup-thin">
                            <TagEditorPopup
                                data={tagging}
                                workspace={contextWorkspace}
                                onChange={(newTags) => {
                                    const mutator = getMutator();
                                    mutator.updateNode(getCreateAffectedData(tagging).uuid, (n) => {
                                        switch (n.type) {
                                            case "Finding":
                                                return n;
                                            case "Domain":
                                                return {
                                                    ...n,
                                                    domain: {
                                                        ...n.domain,
                                                        tags: newTags,
                                                    },
                                                };
                                            case "Host":
                                                return {
                                                    ...n,
                                                    host: {
                                                        ...n.host,
                                                        tags: newTags,
                                                    },
                                                };
                                            case "Port":
                                                return {
                                                    ...n,
                                                    port: {
                                                        ...n.port,
                                                        tags: newTags,
                                                    },
                                                };
                                            case "Service":
                                                return {
                                                    ...n,
                                                    service: {
                                                        ...n.service,
                                                        tags: newTags,
                                                    },
                                                };
                                            case "HttpService":
                                                return {
                                                    ...n,
                                                    httpService: {
                                                        ...n.httpService,
                                                        tags: newTags,
                                                    },
                                                };
                                        }
                                    });
                                    setWantRerender(true);
                                }}
                            />
                        </div>
                    </Popup>
                )}
            </>
        );
    },
);

function TagEditorPopup({
    data,
    workspace,
    onChange,
}: {
    data: CreateFindingObject;
    workspace: string;
    onChange?: (newTags: SimpleTag[]) => void;
}) {
    const obj = getCreateAffectedData(data);
    const [tags, setTags] = React.useState(obj.tags);

    return (
        <form className="pane-thin">
            <EditableTags
                tags={tags}
                workspace={workspace}
                onChange={(newTags) => {
                    onChange?.(newTags);
                    const args = {
                        globalTags: newTags.filter((t) => t.tagType == TagType.Global).map((t) => t.uuid),
                        workspaceTags: newTags.filter((t) => t.tagType == TagType.Workspace).map((t) => t.uuid),
                    };
                    switch (getCreateAffectedType(data)) {
                        case "Domain":
                            Api.workspaces.domains.update(workspace, obj.uuid, args);
                            break;
                        case "Host":
                            Api.workspaces.hosts.update(workspace, obj.uuid, args);
                            break;
                        case "Port":
                            Api.workspaces.ports.update(workspace, obj.uuid, args);
                            break;
                        case "Service":
                            Api.workspaces.services.update(workspace, obj.uuid, args);
                            break;
                        case "HttpService":
                            Api.workspaces.httpServices.update(workspace, obj.uuid, args);
                            break;
                    }
                    setTags(newTags);
                }}
            />
        </form>
    );
}

async function resolveItem(workspace: string, type: AggregationType, uuid: UUID): Promise<CreateFindingObject> {
    switch (type) {
        case "Domain":
            return {
                domain: await Api.workspaces.domains.get(workspace, uuid).then((a) => a.unwrap()),
            };
        case "Host":
            return {
                host: await Api.workspaces.hosts.get(workspace, uuid).then((a) => a.unwrap()),
            };
        case "Port":
            return {
                port: await Api.workspaces.ports.get(workspace, uuid).then((a) => a.unwrap()),
            };
        case "Service":
            return {
                service: await Api.workspaces.services.get(workspace, uuid).then((a) => a.unwrap()),
            };
        case "HttpService":
            return {
                httpService: await Api.workspaces.httpServices.get(workspace, uuid).then((a) => a.unwrap()),
            };
        default:
            const exhaustiveCheck: never = type;
            throw new Error("Invalid item type: " + exhaustiveCheck);
    }
}
