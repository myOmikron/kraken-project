import { Cursor } from "./cursor";
import { parseDate, parseOsType, parsePort, parsePortProtocol, parseString, wrapMaybeRange, wrapRange } from "./parser";

export type ASTField = { [key: string]: { columns: string[]; parse: (cursor: Cursor) => any } };

export const ASTFields = {
    global: {
        tags: {
            columns: ["tags", "tag"],
            parse: parseString,
        },
        createdAt: {
            columns: ["createdAt"],
            parse: wrapRange(parseDate),
        },
    },
    domain: {
        tags: {
            columns: ["tags", "tag"],
            parse: parseString,
        },
        createdAt: {
            columns: ["createdAt"],
            parse: wrapRange(parseDate),
        },
        domains: {
            columns: ["domains", "domain"],
            parse: parseString,
        },
        sourceOf: {
            columns: ["sourceOf"],
            parse: parseString,
        },
        sourceOfTags: {
            columns: ["sourceOf.tag", "sourceOf.tags"],
            parse: parseString,
        },
        sourceOfCreatedAt: {
            columns: ["sourceOf.createdAt"],
            parse: wrapRange(parseDate),
        },
        targetOf: {
            columns: ["targetOf"],
            parse: parseString,
        },
        targetOfTags: {
            columns: ["targetOf.tag", "targetOf.tags"],
            parse: parseString,
        },
        targetOfCreatedAt: {
            columns: ["targetOf.createdAt"],
            parse: wrapRange(parseDate),
        },
        ips: {
            columns: ["ips", "ip"],
            parse: parseString,
        },
        ipsCreatedAt: {
            columns: ["ips.createdAt", "ip.createdAt"],
            parse: wrapRange(parseDate),
        },
        ipsTags: {
            columns: ["ips.tags", "ips.tag", "ip.tags", "ip.tag"],
            parse: parseString,
        },
        ipsOs: {
            columns: ["ips.os", "ip.os"],
            parse: parseOsType,
        },
    },
    host: {
        tags: {
            columns: ["tags", "tag"],
            parse: parseString,
        },
        createdAt: {
            columns: ["createdAt"],
            parse: wrapRange(parseDate),
        },
        ips: {
            columns: ["ips", "ip"],
            parse: parseString,
        },
        os: {
            columns: ["os"],
            parse: parseOsType,
        },
        ports: {
            columns: ["ports", "port"],
            parse: wrapMaybeRange(parsePort),
        },
        portsProtocols: {
            columns: ["ports.protocols", "ports.protocol", "port.protocols", "port.protocol"],
            parse: parsePortProtocol,
        },
        portsTags: {
            columns: ["ports.tags", "ports.tag", "port.tags", "port.tag"],
            parse: parseString,
        },
        portsCreatedAt: {
            columns: ["ports.createdAt", "port.createdAt"],
            parse: wrapRange(parseDate),
        },
        services: {
            columns: ["services", "service"],
            parse: parseString,
        },
        servicesPorts: {
            columns: ["services.ports", "services.port", "service.ports", "service.port"],
            parse: wrapMaybeRange(parsePort),
        },
        servicesProtocols: {
            columns: ["services.protocols", "services.protocol", "service.protocols", "service.protocol"],
            parse: parsePortProtocol,
        },
        servicesTags: {
            columns: ["services.tags", "services.tag", "service.tags", "service.tag"],
            parse: parseString,
        },
        servicesCreatedAt: {
            columns: ["services.createdAt", "service.createdAt"],
            parse: wrapRange(parseDate),
        },
        domains: {
            columns: ["domains", "domain"],
            parse: parseString,
        },
        domainsTags: {
            columns: ["domains.tags", "domains.tag", "domain.tags", "domain.tag"],
            parse: parseString,
        },
        domainsCreatedAt: {
            columns: ["domains.createdAt", "domain.createdAt"],
            parse: wrapRange(parseDate),
        },
    },
    port: {
        tags: {
            columns: ["tags", "tag"],
            parse: parseString,
        },
        createdAt: {
            columns: ["createdAt"],
            parse: wrapRange(parseDate),
        },
        ports: {
            columns: ["ports", "port"],
            parse: wrapMaybeRange(parsePort),
        },
        ips: {
            columns: ["ips", "ip"],
            parse: parseString,
        },
        ipsCreatedAt: {
            columns: ["ips.createdAt", "ip.createdAt"],
            parse: wrapRange(parseDate),
        },
        ipsTags: {
            columns: ["ips.tags", "ips.tag", "ip.tags", "ip.tag"],
            parse: parseString,
        },
        ipsOs: {
            columns: ["ips.os", "ip.os"],
            parse: parseOsType,
        },
        protocols: {
            columns: ["protocols", "protocol"],
            parse: parsePortProtocol,
        },
        services: {
            columns: ["services", "service"],
            parse: parseString,
        },
        servicesTags: {
            columns: ["services.tags", "services.tag", "service.tags", "service.tag"],
            parse: parseString,
        },
        servicesCreatedAt: {
            columns: ["services.createdAt", "service.createdAt"],
            parse: wrapRange(parseDate),
        },
    },
    service: {
        tags: {
            columns: ["tags", "tag"],
            parse: parseString,
        },
        createdAt: {
            columns: ["createdAt"],
            parse: wrapRange(parseDate),
        },
        ips: {
            columns: ["ips", "ip"],
            parse: parseString,
        },
        ipsCreatedAt: {
            columns: ["ips.createdAt", "ip.createdAt"],
            parse: wrapRange(parseDate),
        },
        ipsTags: {
            columns: ["ips.tags", "ips.tag", "ip.tags", "ip.tag"],
            parse: parseString,
        },
        ipsOs: {
            columns: ["ips.os", "ip.os"],
            parse: parseOsType,
        },
        ports: {
            columns: ["ports", "port"],
            parse: wrapMaybeRange(parsePort),
        },
        portsTags: {
            columns: ["ports.tags", "ports.tag", "port.tags", "port.tag"],
            parse: parseString,
        },
        portsCreatedAt: {
            columns: ["ports.createdAt", "port.createdAt"],
            parse: wrapRange(parseDate),
        },
        protocols: {
            columns: ["protocols", "protocol"],
            parse: parsePortProtocol,
        },
        services: {
            columns: ["services", "service"],
            parse: parseString,
        },
    },
} satisfies { [ast: string]: ASTField };

export type ASTType<Fields extends ASTField> = {
    [key in keyof Fields]: Array<ReturnType<Fields[key]["parse"]>>;
};

export type GlobalAST = ASTType<(typeof ASTFields)["global"]>;
export type DomainAST = ASTType<(typeof ASTFields)["domain"]>;
export type HostAST = ASTType<(typeof ASTFields)["host"]>;
export type PortAST = ASTType<(typeof ASTFields)["port"]>;
export type ServiceAST = ASTType<(typeof ASTFields)["service"]>;

export type Exprs<T> = Array<Expr.Or<T>>;

export namespace Expr {
    /** An optional `or` */
    export type Or<T> = {
        /** A list of value joined by `,` */
        or: Array<And<T>>;
    };

    /** An optional `and` */
    export type And<T> = {
        /** A list of value joined by `&` */
        and: Array<Not<T>>;
    };

    /** An optional negation */
    export type Not<T> = {
        /** Should the value be negated */
        not: boolean;

        /** The leaf's value */
        value: Value<T>;
    };

    /** A single value */
    export type Value<T> = T;

    export type Range<T> = {
        start: T | null;
        end: T | null;
    };

    export type MaybeRange<T> = Value<T> | Range<T>;
}
