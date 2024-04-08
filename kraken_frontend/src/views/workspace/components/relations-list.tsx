import { ReactNode } from "react";
import { DomainRelations, HostRelations, PortRelations, ServiceRelations } from "../../../api/generated";
import { HttpServiceRelations } from "../../../api/generated/models/HttpServiceRelations";
import RelationIndirectIcon from "../../../svg/relation-indirect";
import RelationLeftIcon from "../../../svg/relation-left";
import RelationRightIcon from "../../../svg/relation-right";

export type RelationConnectionType = "direct-out" | "direct-source" | "direct-in" | "direct-target" | "indirect";
export type Relation = { connection: RelationConnectionType; type: ReactNode; to: ReactNode };

export function RelationsList({
    relations,
    ...props
}: {
    relations: undefined | Relation[];
} & React.HTMLProps<HTMLDivElement>) {
    return (
        <div className="workspace-data-details-relations-container" {...props}>
            <div className="workspace-data-details-relations-header">
                <div className="workspace-data-details-relations-heading">Connection</div>
                <div className="workspace-data-details-relations-heading">Type</div>
                <div className="workspace-data-details-relations-heading">To</div>
            </div>
            {relations ? (
                <div className="workspace-data-details-relations-body">
                    {relations.map((r, i) => {
                        return (
                            <div key={i} className="workspace-data-details-relations-entry">
                                <RelationConnection type={r.connection} />
                                <span>{r.type}</span>
                                <span>{r.to}</span>
                            </div>
                        );
                    })}
                </div>
            ) : (
                <p>Loading...</p>
            )}
        </div>
    );
}

function RelationConnection({ type }: { type: RelationConnectionType }) {
    switch (type) {
        case "direct-in":
            return (
                <div title={"Direct"}>
                    <RelationLeftIcon />
                </div>
            );
        case "direct-out":
            return (
                <div title={"Direct"}>
                    <RelationRightIcon />
                </div>
            );
        case "direct-source":
            return (
                <div title={"Direct source"}>
                    <RelationLeftIcon />
                </div>
            );
        case "direct-target":
            return (
                <div title={"Direct target"}>
                    <RelationRightIcon />
                </div>
            );
        case "indirect":
            return (
                <div className="indirect" title={"Indirect"}>
                    <RelationIndirectIcon />
                </div>
            );
        default:
            return <div></div>;
    }
}

export function DomainRelationsList({
    relations,
    ...props
}: { relations: DomainRelations | undefined | null } & React.HTMLProps<HTMLDivElement>) {
    return (
        <RelationsList
            relations={
                relations
                    ? [
                          ...relations.sourceDomains.map<Relation>((d) => ({
                              connection: "direct-source",
                              type: "Domain",
                              to: d.domain,
                          })),
                          ...relations.targetDomains.map<Relation>((d) => ({
                              connection: "direct-target",
                              type: "Domain",
                              to: d.domain,
                          })),
                          ...relations.directHosts.map<Relation>((h) => ({
                              connection: "direct-out",
                              type: "Host",
                              to: h.ipAddr,
                          })),
                          ...relations.indirectHosts.map<Relation>((h) => ({
                              connection: "indirect",
                              type: "Host",
                              to: h.ipAddr,
                          })),
                          ...relations.httpServices.map<Relation>((h) => ({
                              connection: "direct-out",
                              type: "HTTP Service",
                              to: h.name,
                          })),
                      ]
                    : undefined
            }
            {...props}
        />
    );
}

export function HostRelationsList({
    relations,
    ...props
}: { relations: HostRelations | undefined | null } & React.HTMLProps<HTMLDivElement>) {
    return (
        <RelationsList
            relations={
                relations
                    ? [
                          ...relations.directDomains.map<Relation>((d) => ({
                              connection: "direct-in",
                              type: "Domain",
                              to: d.domain,
                          })),
                          ...relations.indirectDomains.map<Relation>((d) => ({
                              connection: "indirect",
                              type: "Domain",
                              to: d.domain,
                          })),
                          ...relations.ports.map<Relation>((p) => ({
                              connection: "direct-out",
                              type: "Port",
                              to: p.port,
                          })),
                          ...relations.services.map<Relation>((s) => ({
                              connection: "direct-out",
                              type: "Service",
                              to: s.name,
                          })),
                          ...relations.httpServices.map<Relation>((s) => ({
                              connection: "direct-out",
                              type: "HTTP Service",
                              to: s.name,
                          })),
                      ]
                    : undefined
            }
            {...props}
        />
    );
}

export function PortRelationsList({
    relations,
    ...props
}: { relations: PortRelations | undefined | null } & React.HTMLProps<HTMLDivElement>) {
    return (
        <RelationsList
            relations={
                relations
                    ? [
                          ...(relations.host !== null && relations.host !== undefined
                              ? [
                                    {
                                        connection: "direct-in",
                                        type: "Host",
                                        to: relations.host.ipAddr,
                                    } as Relation,
                                ]
                              : []),
                          ...relations.services.map<Relation>((s) => ({
                              connection: "direct-out",
                              type: "Service",
                              to: s.name,
                          })),
                          ...relations.httpServices.map<Relation>((s) => ({
                              connection: "direct-out",
                              type: "HTTP Service",
                              to: s.name,
                          })),
                      ]
                    : undefined
            }
            {...props}
        />
    );
}

export function ServiceRelationsList({
    relations,
    ...props
}: { relations: ServiceRelations | undefined | null } & React.HTMLProps<HTMLDivElement>) {
    return (
        <RelationsList
            relations={
                relations
                    ? [
                          ...(relations.host !== null && relations.host !== undefined
                              ? [
                                    {
                                        connection: "direct-in",
                                        type: "Host",
                                        to: relations.host.ipAddr,
                                    } as Relation,
                                ]
                              : []),
                          ...(relations.port !== null && relations.port !== undefined
                              ? [
                                    {
                                        connection: "direct-in",
                                        type: "Port",
                                        to: relations.port.port,
                                    } as Relation,
                                ]
                              : []),
                      ]
                    : undefined
            }
            {...props}
        />
    );
}

export function HttpServiceRelationsList({
    relations,
    ...props
}: { relations: HttpServiceRelations | undefined | null } & React.HTMLProps<HTMLDivElement>) {
    return (
        <RelationsList
            relations={
                relations
                    ? [
                          ...(relations.domain !== null && relations.domain !== undefined
                              ? [
                                    {
                                        connection: "direct-in",
                                        type: "Domain",
                                        to: relations.domain.domain,
                                    } as Relation,
                                ]
                              : []),
                          {
                              connection: "direct-in",
                              type: "Host",
                              to: relations.host.ipAddr,
                          },
                          {
                              connection: "direct-in",
                              type: "Port",
                              to: relations.port.port,
                          },
                      ]
                    : undefined
            }
            {...props}
        />
    );
}
