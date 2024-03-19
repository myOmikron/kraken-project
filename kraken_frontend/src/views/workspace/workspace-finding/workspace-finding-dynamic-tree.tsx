import { useEffect, useRef, useState } from "react";
import { Api } from "../../../api/api";
import { ApiError } from "../../../api/error";
import {
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
} from "../../../api/generated";
import CollapseIcon from "../../../svg/collapse";
import ExpandIcon from "../../../svg/expand";
import { handleApiError } from "../../../utils/helper";
import { Result } from "../../../utils/result";
import TagList from "../components/tag-list";
import { ViewportProps } from "../components/viewport";
import { TreeGraph, TreeNode } from "./workspace-finding-tree";

export type DynamicTreeGraphProps = {
    maximizable?: boolean;
} & ({ api: DynamicTreeLookupFunctions } | { workspace: string; uuid: string }) &
    ViewportProps;

export type DynamicTreeWorkspaceFunctions = {
    resolveFinding: (findingDefinition: string) => Promise<Result<FullFindingDefinition, ApiError>>;
    findingsForDomain: (domainUuid: string) => Promise<Result<ListFindings, ApiError>>;
    findingsForHost: (hostUuid: string) => Promise<Result<ListFindings, ApiError>>;
    findingsForService: (serviceUuid: string) => Promise<Result<ListFindings, ApiError>>;
    findingsForPort: (portUuid: string) => Promise<Result<ListFindings, ApiError>>;
    relationsForDomain: (domainUuid: string) => Promise<Result<DomainRelations, ApiError>>;
    relationsForHost: (hostUuid: string) => Promise<Result<HostRelations, ApiError>>;
    relationsForService: (serviceUuid: string) => Promise<Result<ServiceRelations, ApiError>>;
    relationsForPort: (portUuid: string) => Promise<Result<PortRelations, ApiError>>;
    resolveDomain: (domainUuid: string) => Promise<Result<FullDomain, ApiError>>;
    resolveHost: (hostUuid: string) => Promise<Result<FullHost, ApiError>>;
    resolveService: (serviceUuid: string) => Promise<Result<FullService, ApiError>>;
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
      };

export type DynamicTreeLookupFunctions = {
    getRoot: () => Promise<FullFinding>;
    getAffected: (affected: string) => Promise<Result<{ affected: AffectedShallow }, ApiError>>;
} & DynamicTreeWorkspaceFunctions;

export function treeLookupFunctionsWorkspace(workspace: string): DynamicTreeWorkspaceFunctions {
    return {
        resolveFinding: Api.knowledgeBase.findingDefinitions.get,
        findingsForDomain: (d) => Api.workspaces.domains.findings(workspace, d),
        findingsForHost: (d) => Api.workspaces.hosts.findings(workspace, d),
        findingsForService: (d) => Api.workspaces.services.findings(workspace, d),
        findingsForPort: (d) => Api.workspaces.ports.findings(workspace, d),
        relationsForDomain: (d) => Api.workspaces.domains.relations(workspace, d),
        relationsForHost: (d) => Api.workspaces.hosts.relations(workspace, d),
        relationsForService: (d) => Api.workspaces.services.relations(workspace, d),
        relationsForPort: (d) => Api.workspaces.ports.relations(workspace, d),
        resolveDomain: (d) => Api.workspaces.domains.get(workspace, d),
        resolveHost: (d) => Api.workspaces.hosts.get(workspace, d),
        resolveService: (d) => Api.workspaces.services.get(workspace, d),
        resolvePort: (d) => Api.workspaces.ports.get(workspace, d),
    };
}

export function treeLookupFunctionsRoot(workspace: string, uuid: string): DynamicTreeLookupFunctions {
    return {
        getRoot: () =>
            new Promise((resolve) => Api.workspaces.findings.get(workspace, uuid).then(handleApiError(resolve))),
        getAffected: (affected: string) => Api.workspaces.findings.getAffected(workspace, uuid, affected),
        ...treeLookupFunctionsWorkspace(workspace),
    };
}

export function DynamicTreeGraph({ maximizable, ...props }: DynamicTreeGraphProps) {
    const apiTimeout = 100;

    const api = "api" in props ? props.api : treeLookupFunctionsRoot(props.workspace, props.uuid);

    const [maximized, setMaximized] = useState(false);
    const [roots, setRoots] = useState<TreeNode[]>([]);
    const [filterTags, setFilterTags] = useState<SimpleTag[]>([]);
    const addedUuids = useRef<{ [index: string]: null }>({});

    const toggleMax = () => setMaximized((m) => !m);

    const apiOrWorkspace = "api" in props ? props.api : props.workspace;
    const apiOrUuid = "api" in props ? props.api : props.uuid;

    useEffect(() => {
        const srcUuid = apiOrUuid;
        const srcWorkspace = apiOrWorkspace;
        const shouldAbort = () => srcUuid != apiOrUuid || srcWorkspace != apiOrWorkspace;

        setRoots([]);
        addedUuids.current = {};

        /// inserts the child to parent, returns true if the child hasn't been added to the tree yet
        const insertChild = (parent: TreeNode, child: TreeNode) => {
            const isNew = !(child.uuid in addedUuids.current);
            addedUuids.current[child.uuid] = null;
            let found = false;
            let copied: { [uuid: string]: TreeNode } = {};
            function mutate(n: TreeNode): TreeNode {
                if (copied[n.uuid]) return copied[n.uuid];
                if (n.uuid == parent.uuid) {
                    found = true;
                    return (copied[n.uuid] = {
                        ...n,
                        children: [...(n.children ?? []), child],
                    });
                } else {
                    let copy: TreeNode = {
                        ...n,
                    };
                    copied[n.uuid] = copy;
                    copy.children = n.children?.map(mutate);
                    return copy;
                }
            }

            setRoots((r) => {
                let res = r.map(mutate);
                if (!found) console.warn("couldn't find ", parent, " in roots to insert child into?!");
                return res;
            });
            return isNew;
        };

        const insertFindings = (parent: TreeNode, node: TreeNode, findings: ListFindings) => {
            if (shouldAbort()) return;
            for (const f of findings.findings) {
                if (f.uuid == parent.uuid) return;
                api.resolveFinding(f.definition).then(
                    handleApiError((definition) => {
                        if (shouldAbort()) return;
                        const insert: TreeNode = {
                            uuid: f.uuid,
                            type: "Finding",
                            definition: definition,
                        };
                        if (!insertChild(node, insert)) return;
                        // TODO: recurse here
                    }),
                );
            }
        };
        const populateDomain = (parent: TreeNode, node: TreeNode, domain: SimpleDomain | FullDomain) => {
            api.findingsForDomain(domain.uuid).then(
                handleApiError((findings) => insertFindings(parent, node, findings)),
            );
            api.relationsForDomain(domain.uuid).then(
                handleApiError((relations) => {
                    [...relations.directHosts, ...relations.indirectHosts].forEach((c) => insertHostSimple(node, c));
                    [...relations.sourceDomains, ...relations.targetDomains].forEach((c) =>
                        insertDomainSimple(node, c),
                    );
                }),
            );
        };
        const populateHost = (parent: TreeNode, node: TreeNode, host: SimpleHost | FullHost) => {
            api.findingsForHost(host.uuid).then(handleApiError((findings) => insertFindings(parent, node, findings)));
            api.relationsForHost(host.uuid).then(
                handleApiError((relations) => {
                    [...relations.directDomains, ...relations.indirectDomains].forEach((c) =>
                        insertDomainSimple(node, c),
                    );
                    relations.ports.forEach((c) => insertPortSimple(node, c));
                    relations.services.forEach((c) => insertServiceSimple(node, c));
                }),
            );
        };
        const populatePort = (parent: TreeNode, node: TreeNode, port: SimplePort | FullPort) => {
            api.findingsForPort(port.uuid).then(handleApiError((findings) => insertFindings(parent, node, findings)));
            api.relationsForPort(port.uuid).then(
                handleApiError((relations) => {
                    insertHostSimple(node, relations.host);
                    relations.services.forEach((c) => insertServiceSimple(node, c));
                }),
            );
        };
        const populateService = (parent: TreeNode, node: TreeNode, service: SimpleService | FullService) => {
            api.findingsForService(service.uuid).then(
                handleApiError((findings) => insertFindings(parent, node, findings)),
            );
            api.relationsForService(service.uuid).then(
                handleApiError((relations) => {
                    insertHostSimple(node, relations.host);
                    if (relations.port) insertPortSimple(node, relations.port);
                }),
            );
        };
        const insertDomain = (node: TreeNode, child: FullDomain) => {
            if (shouldAbort()) return;
            const insert: TreeNode = {
                type: "Domain",
                domain: child,
                uuid: child.uuid,
            };
            if (!insertChild(node, insert)) return;
            setTimeout(function () {
                populateDomain(node, insert, child);
            }, apiTimeout);
        };
        const insertHost = (node: TreeNode, child: FullHost) => {
            if (shouldAbort()) return;
            const insert: TreeNode = {
                type: "Host",
                host: child,
                uuid: child.uuid,
            };
            if (!insertChild(node, insert)) return;
            setTimeout(function () {
                populateHost(node, insert, child);
            }, apiTimeout);
        };
        const insertPort = (node: TreeNode, child: FullPort) => {
            if (shouldAbort()) return;
            const insert: TreeNode = {
                type: "Port",
                port: child,
                uuid: child.uuid,
            };
            if (!insertChild(node, insert)) return;
            setTimeout(function () {
                populatePort(node, insert, child);
            }, apiTimeout);
        };
        const insertService = (node: TreeNode, child: FullService) => {
            if (shouldAbort()) return;
            const insert: TreeNode = {
                type: "Service",
                service: child,
                uuid: child.uuid,
            };
            if (!insertChild(node, insert)) return;
            setTimeout(function () {
                populateService(node, insert, child);
            }, apiTimeout);
        };

        const insertDomainSimple = (node: TreeNode, child: { uuid: string }) => {
            api.resolveDomain(child.uuid).then(handleApiError((full) => insertDomain(node, full)));
        };
        const insertHostSimple = (node: TreeNode, child: { uuid: string }) => {
            api.resolveHost(child.uuid).then(handleApiError((full) => insertHost(node, full)));
        };
        const insertPortSimple = (node: TreeNode, child: { uuid: string }) => {
            api.resolvePort(child.uuid).then(handleApiError((full) => insertPort(node, full)));
        };
        const insertServiceSimple = (node: TreeNode, child: { uuid: string }) => {
            api.resolveService(child.uuid).then(handleApiError((full) => insertService(node, full)));
        };
        const populateAffectedRoot = (root: TreeNode, affected: SimpleFindingAffected[]) => {
            if (shouldAbort()) return;
            for (const a of affected) {
                api.getAffected(a.affectedUuid).then(
                    handleApiError((affected) => {
                        if (shouldAbort()) return;
                        if ("domain" in affected.affected && affected.affected.domain) {
                            insertDomainSimple(root, affected.affected.domain);
                        } else if ("host" in affected.affected && affected.affected.host) {
                            insertHostSimple(root, affected.affected.host);
                        } else if ("port" in affected.affected && affected.affected.port) {
                            insertPortSimple(root, affected.affected.port);
                        } else if ("service" in affected.affected && affected.affected.service) {
                            insertServiceSimple(root, affected.affected.service);
                        }
                    }),
                );
            }
        };

        api.getRoot().then((finding) => {
            if (shouldAbort()) return;
            const root: TreeNode = {
                uuid: finding.uuid,
                type: "Finding",
                definition: finding.definition,
            };
            setRoots([root]);
            setTimeout(function () {
                populateAffectedRoot(root, finding.affected);
            }, apiTimeout);
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
            case "Finding":
                return true;
        }
    }

    return (
        <TreeGraph
            className={`${maximized ? "maximized" : ""}`}
            onClickTag={(e, tag) => {
                if (e.altKey) setFilterTags((tags) => tags.filter((t) => t.uuid != tag.uuid));
                else setFilterTags((tags) => (tags.some((t) => t.uuid == tag.uuid) ? tags : [...tags, tag]));
            }}
            roots={
                filterTags.length
                    ? roots.map((root) => {
                          type T = TreeNode & { _hit: boolean };
                          let visited: { [uuid: string]: T } = {};
                          function markTagged(n: TreeNode, parents: T[]): T {
                              if (visited[n.uuid]) return visited[n.uuid];
                              const v: T = (visited[n.uuid] = {
                                  ...n,
                                  _hit: checkTag(n),
                              });
                              if (v._hit) for (const p of parents) p._hit = true;
                              v.children = v.children?.map((c) => markTagged(c, [...parents, v]));
                              return v;
                          }
                          let tagged = markTagged(root, []);
                          let visited2: { [uuid: string]: TreeNode } = {};
                          function filterTagged(n: T): TreeNode {
                              if (visited2[n.uuid]) return visited2[n.uuid];
                              if (!("_hit" in n)) return n;
                              let { _hit, ...node } = n;
                              visited2[n.uuid] = node;
                              node.children = node.children
                                  ?.filter((v) => (v as T)._hit)
                                  .map((v) => filterTagged(v as T));
                              return node;
                          }
                          return filterTagged(tagged);
                      })
                    : roots
            }
            decoration={
                <>
                    <TagList
                        tags={filterTags}
                        onClickTag={(e, tag) => {
                            setFilterTags((tags) => tags.filter((t) => t.uuid != tag.uuid));
                        }}
                    />
                    {maximizable && (
                        <button
                            className="maximize"
                            onClick={toggleMax}
                            aria-label={maximized ? "Minimize" : "Maximize"}
                            title={maximized ? "Minimize" : "Maximize"}
                        >
                            {maximized ? <CollapseIcon /> : <ExpandIcon />}
                        </button>
                    )}
                </>
            }
            {...props}
        />
    );
}
