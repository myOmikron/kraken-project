import { PortProtocol } from "../../api/generated";

export type GlobalAST = {
    tags: Exprs<string>;
    createdAt: Exprs<Expr.Range<Date>>;
};

export type DomainAST = {
    tags: Exprs<string>;
    createdAt: Exprs<Expr.Range<Date>>;
    domains: Exprs<string>;
};

export type HostAST = {
    tags: Exprs<string>;
    createdAt: Exprs<Expr.Range<Date>>;
    ips: Exprs<string>;

    ports: Exprs<Expr.MaybeRange<number>>;
    portsProtocols: Exprs<PortProtocol>;
    portsTags: Exprs<string>;
    portsCreatedAt: Exprs<Expr.Range<Date>>;

    services: Exprs<string>;
    servicesPorts: Exprs<Expr.MaybeRange<number>>;
    servicesProtocols: Exprs<PortProtocol>;
    servicesTags: Exprs<string>;
    servicesCreatedAt: Exprs<Expr.Range<Date>>;
};

export type PortAST = {
    tags: Exprs<string>;
    createdAt: Exprs<Expr.Range<Date>>;
    ports: Exprs<Expr.MaybeRange<number>>;
    ips: Exprs<string>;
    ipsCreatedAt: Exprs<Expr.Range<Date>>;
    ipsTags: Exprs<string>;
    protocols: Exprs<PortProtocol>;

    services: Exprs<string>;
    servicesTags: Exprs<string>;
    servicesCreatedAt: Exprs<Expr.Range<Date>>;
};

export type ServiceAST = {
    tags: Exprs<string>;
    createdAt: Exprs<Expr.Range<Date>>;
    ips: Exprs<string>;
    ipsCreatedAt: Exprs<Expr.Range<Date>>;
    ipsTags: Exprs<string>;
    ports: Exprs<Expr.MaybeRange<number>>;
    portsTags: Exprs<string>;
    portsCreatedAt: Exprs<Expr.Range<Date>>;
    protocols: Exprs<PortProtocol>;
    services: Exprs<string>;
};

export type Expr<T> = Expr.Or<T>;
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
