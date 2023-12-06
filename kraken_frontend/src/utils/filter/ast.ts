import { PortProtocol } from "../../api/generated";

export type GlobalAST = {
    tags: Array<Expr.Or<string>>;
    createdAt: Array<Expr.Or<Expr.Range<Date>>>;
};

export type DomainAST = {
    tags: Array<Expr.Or<string>>;
    createdAt: Array<Expr.Or<Expr.Range<Date>>>;
    domains: Array<Expr.Or<string>>;
};

export type HostAST = {
    tags: Array<Expr.Or<string>>;
    createdAt: Array<Expr.Or<Expr.Range<Date>>>;
    ips: Array<Expr.Or<string>>;
};

export type PortAST = {
    tags: Array<Expr.Or<string>>;
    createdAt: Array<Expr.Or<Expr.Range<Date>>>;
    ports: Array<Expr.Or<Expr.MaybeRange<number>>>;
    ips: Array<Expr.Or<string>>;
    protocols: Array<Expr.Or<PortProtocol>>;
};

export type ServiceAST = {
    tags: Array<Expr.Or<string>>;
    createdAt: Array<Expr.Or<Expr.Range<Date>>>;
    ips: Array<Expr.Or<string>>;
    ports: Array<Expr.Or<Expr.MaybeRange<number>>>;
    services: Array<Expr.Or<string>>;
};

export type Expr<T> = Expr.Or<T>;

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
