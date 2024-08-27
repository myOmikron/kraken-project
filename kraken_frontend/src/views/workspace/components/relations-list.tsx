import { ReactNode } from "react";
import { DomainRelations, HostRelations, PortRelations, ServiceRelations } from "../../../api/generated";
import { HttpServiceRelations } from "../../../api/generated/models/HttpServiceRelations";
import RelationIndirectIcon from "../../../svg/relation-indirect";
import RelationLeftIcon from "../../../svg/relation-left";
import RelationRightIcon from "../../../svg/relation-right";

/**
 * The relation type, used for the icon (arrow direction) and tooltip.
 */
export type RelationConnectionType = "direct-out" | "direct-source" | "direct-in" | "direct-target" | "indirect";

/**
 * A single relation, corresponds to exactly one row in the <RelationsList> component.
 */
export type Relation = {
    /** The type to render the icon for and choose the tooltip based on */
    connection: RelationConnectionType;
    /** UI node what to render in the type column */
    type: ReactNode;
    /** UI node what to render in the to column */
    to: ReactNode;
};

/**
 * A component showing a list of relations in a <div>, which can be customized
 * using regular div props as well.
 *
 * Usually you would use one of the more specific relation list types that are
 * specifically for domains, hosts, ports, etc. instead.
 */
export function RelationsList(
    props: {
        /**
         * List of relations to render
         */
        relations: undefined | Relation[];
    } & React.HTMLProps<HTMLDivElement>,
) {
    const { relations, ...divProps } = props;

    return (
        <div className="workspace-data-details-relations-container" {...divProps}>
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

/**
 * Renders an icon with a tooltip for the relation direction.
 */
function RelationConnection(props: {
    /**
     * The relation connection type
     */
    type: RelationConnectionType;
}) {
    const { type } = props;
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

/**
 * Relation list for domains.
 */
export function DomainRelationsList(
    props: {
        /**
         * domain relations
         */
        relations: DomainRelations | undefined | null;
    } & React.HTMLProps<HTMLDivElement>,
) {
    const { relations, ...divProps } = props;
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
            {...divProps}
        />
    );
}

/**
 * Relation list for hosts.
 */
export function HostRelationsList(
    props: {
        /**
         * host relations
         */
        relations: HostRelations | undefined | null;
    } & React.HTMLProps<HTMLDivElement>,
) {
    const { relations, ...divProps } = props;
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
            {...divProps}
        />
    );
}

/**
 * Relation list for ports
 */
export function PortRelationsList(
    props: {
        /**
         * port relations
         */
        relations: PortRelations | undefined | null;
    } & React.HTMLProps<HTMLDivElement>,
) {
    const { relations, ...divProps } = props;
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
            {...divProps}
        />
    );
}

/**
 * Relation list for services
 */
export function ServiceRelationsList(
    props: {
        /**
         * service relations
         */
        relations: ServiceRelations | undefined | null;
    } & React.HTMLProps<HTMLDivElement>,
) {
    const { relations, ...divProps } = props;
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
            {...divProps}
        />
    );
}

/**
 * Relation list for HTTP services
 */
export function HttpServiceRelationsList(
    props: {
        /**
         * HTTP service relations
         */
        relations: HttpServiceRelations | undefined | null;
    } & React.HTMLProps<HTMLDivElement>,
) {
    const { relations, ...divProps } = props;
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
            {...divProps}
        />
    );
}
