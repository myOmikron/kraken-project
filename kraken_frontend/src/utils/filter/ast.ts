import { OsType, PortProtocol } from "../../api/generated";
import { Cursor } from "./cursor";
import { parseDate, parseOsType, parsePort, parsePortProtocol, parseString, wrapMaybeRange, wrapRange } from "./parser";

export type ASTField = {
    [key: string]: {
        label: string;
        columns: string[];
        parse: (cursor: Cursor) => any;
        advanced?: boolean;
    };
};

export const ASTFields = {
    global: {
        tags: {
            label: "Tags",
            columns: ["tags", "tag"],
            parse: parseString,
        },
        createdAt: {
            label: "Creation date",
            columns: ["createdAt"],
            parse: wrapRange(parseDate),
        },
    },
    domain: {
        tags: {
            label: "Tags",
            columns: ["tags", "tag"],
            parse: parseString,
        },
        createdAt: {
            label: "Creation date",
            columns: ["createdAt"],
            parse: wrapRange(parseDate),
        },
        domains: {
            label: "Domain",
            columns: ["domains", "domain"],
            parse: parseString,
        },
        sourceOf: {
            label: "Source of domain",
            columns: ["sourceOf"],
            parse: parseString,
            advanced: true,
        },
        sourceOfTags: {
            label: "Source of domain tags",
            columns: ["sourceOf.tag", "sourceOf.tags"],
            parse: parseString,
            advanced: true,
        },
        sourceOfCreatedAt: {
            label: "Source of creation date",
            columns: ["sourceOf.createdAt"],
            parse: wrapRange(parseDate),
            advanced: true,
        },
        targetOf: {
            label: "Target of domain",
            columns: ["targetOf"],
            parse: parseString,
            advanced: true,
        },
        targetOfTags: {
            label: "Target of domain tags",
            columns: ["targetOf.tag", "targetOf.tags"],
            parse: parseString,
            advanced: true,
        },
        targetOfCreatedAt: {
            label: "Target of creation date",
            columns: ["targetOf.createdAt"],
            parse: wrapRange(parseDate),
            advanced: true,
        },
        ips: {
            label: "IPs",
            columns: ["ips", "ip"],
            parse: parseString,
        },
        ipsCreatedAt: {
            label: "IP creation date",
            columns: ["ips.createdAt", "ip.createdAt"],
            parse: wrapRange(parseDate),
            advanced: true,
        },
        ipsTags: {
            label: "IP tags",
            columns: ["ips.tags", "ips.tag", "ip.tags", "ip.tag"],
            parse: parseString,
            advanced: true,
        },
        ipsOs: {
            label: "Host OS",
            columns: ["ips.os", "ip.os"],
            parse: parseOsType,
            advanced: true,
        },
    },
    host: {
        tags: {
            label: "Tags",
            columns: ["tags", "tag"],
            parse: parseString,
        },
        createdAt: {
            label: "Creation date",
            columns: ["createdAt"],
            parse: wrapRange(parseDate),
        },
        ips: {
            label: "IPs",
            columns: ["ips", "ip"],
            parse: parseString,
        },
        os: {
            label: "Operating System",
            columns: ["os"],
            parse: parseOsType,
        },
        ports: {
            label: "Ports",
            columns: ["ports", "port"],
            parse: wrapMaybeRange(parsePort),
        },
        portsProtocols: {
            label: "Port protocols",
            columns: ["ports.protocols", "ports.protocol", "port.protocols", "port.protocol"],
            parse: parsePortProtocol,
            advanced: true,
        },
        portsTags: {
            label: "Port tags",
            columns: ["ports.tags", "ports.tag", "port.tags", "port.tag"],
            parse: parseString,
            advanced: true,
        },
        portsCreatedAt: {
            label: "Port creation date",
            columns: ["ports.createdAt", "port.createdAt"],
            parse: wrapRange(parseDate),
            advanced: true,
        },
        services: {
            label: "Services",
            columns: ["services", "service"],
            parse: parseString,
        },
        servicesPorts: {
            label: "Service ports",
            columns: ["services.ports", "services.port", "service.ports", "service.port"],
            parse: wrapMaybeRange(parsePort),
            advanced: true,
        },
        servicesProtocols: {
            label: "Service protocols",
            columns: ["services.protocols", "services.protocol", "service.protocols", "service.protocol"],
            parse: parsePortProtocol,
            advanced: true,
        },
        servicesTags: {
            label: "Service tags",
            columns: ["services.tags", "services.tag", "service.tags", "service.tag"],
            parse: parseString,
            advanced: true,
        },
        servicesCreatedAt: {
            label: "Service creation date",
            columns: ["services.createdAt", "service.createdAt"],
            parse: wrapRange(parseDate),
            advanced: true,
        },
        domains: {
            label: "Domains",
            columns: ["domains", "domain"],
            parse: parseString,
        },
        domainsTags: {
            label: "Domain tags",
            columns: ["domains.tags", "domains.tag", "domain.tags", "domain.tag"],
            parse: parseString,
            advanced: true,
        },
        domainsCreatedAt: {
            label: "Domain creation date",
            columns: ["domains.createdAt", "domain.createdAt"],
            parse: wrapRange(parseDate),
            advanced: true,
        },
    },
    port: {
        tags: {
            label: "Tags",
            columns: ["tags", "tag"],
            parse: parseString,
        },
        createdAt: {
            label: "Creation date",
            columns: ["createdAt"],
            parse: wrapRange(parseDate),
        },
        ports: {
            label: "Ports",
            columns: ["ports", "port"],
            parse: wrapMaybeRange(parsePort),
        },
        ips: {
            label: "IPs",
            columns: ["ips", "ip"],
            parse: parseString,
        },
        ipsCreatedAt: {
            label: "IP creation date",
            columns: ["ips.createdAt", "ip.createdAt"],
            parse: wrapRange(parseDate),
            advanced: true,
        },
        ipsTags: {
            label: "IP tags",
            columns: ["ips.tags", "ips.tag", "ip.tags", "ip.tag"],
            parse: parseString,
            advanced: true,
        },
        ipsOs: {
            label: "Host OS",
            columns: ["ips.os", "ip.os"],
            parse: parseOsType,
            advanced: true,
        },
        protocols: {
            label: "Protocols",
            columns: ["protocols", "protocol"],
            parse: parsePortProtocol,
        },
        services: {
            label: "Services",
            columns: ["services", "service"],
            parse: parseString,
        },
        servicesTags: {
            label: "Service tags",
            columns: ["services.tags", "services.tag", "service.tags", "service.tag"],
            parse: parseString,
            advanced: true,
        },
        servicesCreatedAt: {
            label: "Service creation date",
            columns: ["services.createdAt", "service.createdAt"],
            parse: wrapRange(parseDate),
            advanced: true,
        },
    },
    service: {
        tags: {
            label: "Tags",
            columns: ["tags", "tag"],
            parse: parseString,
        },
        createdAt: {
            label: "Creation date",
            columns: ["createdAt"],
            parse: wrapRange(parseDate),
        },
        ips: {
            label: "IPs",
            columns: ["ips", "ip"],
            parse: parseString,
        },
        ipsCreatedAt: {
            label: "IP creation date",
            columns: ["ips.createdAt", "ip.createdAt"],
            parse: wrapRange(parseDate),
            advanced: true,
        },
        ipsTags: {
            label: "IP tags",
            columns: ["ips.tags", "ips.tag", "ip.tags", "ip.tag"],
            parse: parseString,
            advanced: true,
        },
        ipsOs: {
            label: "Host OS",
            columns: ["ips.os", "ip.os"],
            parse: parseOsType,
            advanced: true,
        },
        ports: {
            label: "Port",
            columns: ["ports", "port"],
            parse: wrapMaybeRange(parsePort),
        },
        portsTags: {
            label: "Port tags",
            columns: ["ports.tags", "ports.tag", "port.tags", "port.tag"],
            parse: parseString,
            advanced: true,
        },
        portsCreatedAt: {
            label: "Port creation date",
            columns: ["ports.createdAt", "port.createdAt"],
            parse: wrapRange(parseDate),
            advanced: true,
        },
        protocols: {
            label: "Protocols",
            columns: ["protocols", "protocol"],
            parse: parsePortProtocol,
        },
        services: {
            label: "Services",
            columns: ["services", "service"],
            parse: parseString,
        },
    },
} satisfies { [ast: string]: ASTField };

export type ASTType<Fields extends ASTField> = {
    [key in keyof Fields]: ReturnType<Fields[key]["parse"]>;
};
export type ASTResult<Fields extends ASTField> = {
    [key in keyof Fields]: Array<ReturnType<Fields[key]["parse"]>>;
};

export type GlobalAST = ASTResult<(typeof ASTFields)["global"]>;
export type DomainAST = ASTResult<(typeof ASTFields)["domain"]>;
export type HostAST = ASTResult<(typeof ASTFields)["host"]>;
export type PortAST = ASTResult<(typeof ASTFields)["port"]>;
export type ServiceAST = ASTResult<(typeof ASTFields)["service"]>;

// these types are defined to automatically check for full coverage of all keys:
export type FieldTypes<T extends ASTType<any>> = {
    [key in keyof T]: FieldTypeValue<key, T[key]>;
};
export type GlobalFieldTypes = FieldTypes<GlobalAST>;
export type DomainFieldTypes = FieldTypes<DomainAST>;
export type HostFieldTypes = FieldTypes<HostAST>;
export type PortFieldTypes = FieldTypes<PortAST>;
export type ServiceFieldTypes = FieldTypes<ServiceAST>;
type FieldTypeValue<key, T> = key extends "tags" | `${any}Tags`
    ? "tags"
    : T extends Expr.Range<infer U>
      ? T extends Expr.MaybeRange<U>
          ? `mayberange.${FieldTypeValue<key, U>}`
          : `range.${FieldTypeValue<key, U>}`
      : T extends PortProtocol
        ? "protocol"
        : T extends OsType
          ? "ostype"
          : T extends number
            ? "number" | "port"
            : T extends string
              ? "string" | "domain" | "host" | "service"
              : T extends Date
                ? "date"
                : never;

export const ASTFieldTypes = {
    global: {
        tags: "tags",
        createdAt: "mayberange.date",
    },
    domain: {
        tags: "tags",
        createdAt: "mayberange.date",
        domains: "domain",
        sourceOf: "domain",
        sourceOfTags: "tags",
        sourceOfCreatedAt: "mayberange.date",
        targetOf: "domain",
        targetOfTags: "tags",
        targetOfCreatedAt: "mayberange.date",
        ips: "host",
        ipsCreatedAt: "mayberange.date",
        ipsTags: "tags",
        ipsOs: "ostype",
    },
    host: {
        tags: "tags",
        createdAt: "mayberange.date",
        ips: "host",
        os: "ostype",
        ports: "mayberange.port",
        portsProtocols: "protocol",
        portsTags: "tags",
        portsCreatedAt: "mayberange.date",
        services: "service",
        servicesPorts: "mayberange.port",
        servicesProtocols: "protocol",
        servicesTags: "tags",
        servicesCreatedAt: "mayberange.date",
        domains: "domain",
        domainsTags: "tags",
        domainsCreatedAt: "mayberange.date",
    },
    port: {
        tags: "tags",
        createdAt: "mayberange.date",
        ports: "mayberange.port",
        ips: "host",
        ipsCreatedAt: "mayberange.date",
        ipsTags: "tags",
        ipsOs: "ostype",
        protocols: "protocol",
        services: "service",
        servicesTags: "tags",
        servicesCreatedAt: "mayberange.date",
    },
    service: {
        tags: "tags",
        createdAt: "mayberange.date",
        ips: "host",
        ipsCreatedAt: "mayberange.date",
        ipsTags: "tags",
        ipsOs: "ostype",
        ports: "mayberange.port",
        portsTags: "tags",
        portsCreatedAt: "mayberange.date",
        protocols: "protocol",
        services: "service",
    },
} satisfies { [K in keyof typeof ASTFields]: FieldTypes<ASTType<(typeof ASTFields)[K]>> };

/** Union of all actually used AST types */
export type UsedASTTypes =
    | (typeof ASTFieldTypes)["global"][keyof (typeof ASTFieldTypes)["global"]]
    | (typeof ASTFieldTypes)["domain"][keyof (typeof ASTFieldTypes)["domain"]]
    | (typeof ASTFieldTypes)["host"][keyof (typeof ASTFieldTypes)["host"]]
    | (typeof ASTFieldTypes)["service"][keyof (typeof ASTFieldTypes)["service"]]
    | (typeof ASTFieldTypes)["port"][keyof (typeof ASTFieldTypes)["port"]];

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
