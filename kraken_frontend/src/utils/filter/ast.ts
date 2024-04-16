import { OsType, PortProtocol } from "../../api/generated";
import { Cursor } from "./cursor";
import {
    parseDate,
    parseOsType,
    parsePort,
    parsePortProtocol,
    parseServiceTransport,
    parseString,
    wrapMaybeRange,
    wrapRange,
} from "./parser";

/**
 * Generic type for each field inside the `ASTFields` variable. Unless you need
 * to work on the generic data without much type-safety, this isn't really used.
 *
 * Instead, use `typeof ASTFields["yourKey"]` as a type, since it contains more
 * specific types (e.g. for `parse`).
 */
export type ASTField = {
    [key: string]: {
        /**
         * Human-readable text, shown in filter editor UI
         */
        label: string;
        /**
         * All possible filter language AST names for this column that mean the same.
         */
        columns: string[];
        // we use any since this type is used for `satisfies` and not to specify
        // the exact type of ASTFields. The real type of ASTFields will contain
        // the proper types, which can be extracted with `FieldTypes` as well.
        // See GlobalAST, DomainAST, HostAST, etc.
        /**
         * Specifies the AST parsing function to be used for this field.
         */
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        parse: (cursor: Cursor) => any;
        /**
         * If set and true, hides this setting inside a collapsible menu in the
         * filter editor UI.
         */
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
        httpServices: {
            label: "HTTP Services",
            columns: ["httpServices", "httpService"],
            parse: parseString,
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
        httpServices: {
            label: "HTTP Services",
            columns: ["httpServices", "httpService"],
            parse: parseString,
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
        httpServices: {
            label: "HTTP Services",
            columns: ["httpServices", "httpService"],
            parse: parseString,
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
        transport: {
            label: "Transport Type",
            columns: ["transports", "transport"],
            parse: parseServiceTransport,
        },
    },
    httpService: {
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
        // TODO
    },
} satisfies { [ast: string]: ASTField };

/**
 * For a each AST field, this maps the column name to the type the parse
 * function returns.
 */
export type ASTType<Fields extends ASTField> = {
    [key in keyof Fields]: ReturnType<Fields[key]["parse"]>;
};

/**
 * This is the parsing result type, which is just an Array of each parse type
 * for each column, since columns may be specified multiple times or just use
 * or expressions in the top level.
 */
export type ASTResult<Fields extends ASTField> = {
    [key in keyof Fields]: Array<Expr.Or<ReturnType<Fields[key]["parse"]>>>;
};

/**
 * Result type for global AST
 */
export type GlobalAST = ASTResult<(typeof ASTFields)["global"]>;

/**
 * Result type for domain AST
 */
export type DomainAST = ASTResult<(typeof ASTFields)["domain"]>;

/**
 * Result type for host AST
 */
export type HostAST = ASTResult<(typeof ASTFields)["host"]>;

/**
 * Result type for port AST
 */
export type PortAST = ASTResult<(typeof ASTFields)["port"]>;

/**
 * Result type for service AST
 */
export type ServiceAST = ASTResult<(typeof ASTFields)["service"]>;

/**
 * Result type for http service AST
 */
export type HttpServiceAST = ASTResult<(typeof ASTFields)["httpService"]>;

// these types are defined to automatically check for full coverage of all keys:
/**
 * This is the type of `ASTFieldTypes` for each different AST type. The type
 * restricts the possible runtime types (e.g. a string can be a generic string
 * input, but also a UUID for domain/host/etc. or an enum value)
 *
 * Allows to auto-complete and auto-verify the `ASTFieldTypes` value.
 */
export type FieldTypes<T extends ASTType<ASTField>> = {
    [key in keyof T]: FieldTypeValue<key, T[key]>;
};

/** FieldTypes for global AST. Hover over this in the IDE to see possible types */
export type GlobalFieldTypes = FieldTypes<GlobalAST>;
/** FieldTypes for domain AST. Hover over this in the IDE to see possible types */
export type DomainFieldTypes = FieldTypes<DomainAST>;
/** FieldTypes for host AST. Hover over this in the IDE to see possible types */
export type HostFieldTypes = FieldTypes<HostAST>;
/** FieldTypes for port AST. Hover over this in the IDE to see possible types */
export type PortFieldTypes = FieldTypes<PortAST>;
/** FieldTypes for service AST. Hover over this in the IDE to see possible types */
export type ServiceFieldTypes = FieldTypes<ServiceAST>;
/** FieldTypes for http service AST. Hover over this in the IDE to see possible types */
export type HttpServiceFieldTypes = FieldTypes<HttpServiceAST>;

/** Lists the possible runtime types for the generic JS type */
type FieldTypeValue<key, T> = key extends "tags" | `${string}Tags`
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
              ? "string" | "domain" | "host" | "service" | "httpService" | "transport"
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
        httpServices: "httpService",
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
        httpServices: "httpService",
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
        httpServices: "httpService",
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
        transport: "transport",
    },
    httpService: {
        tags: "tags",
        createdAt: "mayberange.date",
        // TODO
    },
} satisfies { [K in keyof typeof ASTFields]: FieldTypes<ASTType<(typeof ASTFields)[K]>> };

/** Union of all actually used AST types */
export type UsedASTTypes =
    | (typeof ASTFieldTypes)["global"][keyof (typeof ASTFieldTypes)["global"]]
    | (typeof ASTFieldTypes)["domain"][keyof (typeof ASTFieldTypes)["domain"]]
    | (typeof ASTFieldTypes)["host"][keyof (typeof ASTFieldTypes)["host"]]
    | (typeof ASTFieldTypes)["service"][keyof (typeof ASTFieldTypes)["service"]]
    | (typeof ASTFieldTypes)["httpService"][keyof (typeof ASTFieldTypes)["httpService"]]
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
