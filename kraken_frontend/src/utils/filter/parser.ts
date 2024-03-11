import { OsType, PortProtocol } from "../../api/generated";
import { ASTField, ASTFields, ASTResult, DomainAST, Expr, GlobalAST, HostAST, PortAST, ServiceAST } from "./ast";
import { Cursor } from "./cursor";
import ParserError from "./error";
import { tokenize } from "./lexer";

/**
 * Parse a string into a generic AST defined in {@link ASTFields}
 *
 * @throws ParserError
 */
export function parseAstFields<Fields extends ASTField>(input: string, ast: Fields): ASTResult<Fields> {
    // create object like `{ tags: [], createdAt: [], ... }`
    let ret: {
        [Key in keyof Fields]: any[];
    } = Object.fromEntries(Object.keys(ast).map((k) => [k, []])) as any;

    parseAst(input, (column, cursor) => {
        let field = Object.keys(ast).find((field) => ast[field].columns.includes(column));
        if (!field) throw new ParserError({ type: "unknownColumn", column });
        ret[field].push(parseOr(cursor, ast[field].parse));
    });
    return ret;
}

/**
 * Parse a string into a {@link GlobalAST}
 *
 * @throws ParserError
 */
export function parseGlobalAST(input: string): GlobalAST {
    return parseAstFields(input, ASTFields.global);
}

/**
 * Parse a string into a {@link DomainAST}
 *
 * @throws ParserError
 */
export function parseDomainAST(input: string): DomainAST {
    return parseAstFields(input, ASTFields.domain);
}

/**
 * Parse a string into a {@link HostAST}
 *
 * @throws ParserError
 */
export function parseHostAST(input: string): HostAST {
    return parseAstFields(input, ASTFields.host);
}

/**
 * Parse a string into a {@link PortAST}
 *
 * @throws ParserError
 */
export function parsePortAST(input: string): PortAST {
    return parseAstFields(input, ASTFields.port);
}

/**
 * Parse a string into a {@link ServiceAST}
 *
 * @throws ParserError
 */
export function parseServiceAST(input: string): ServiceAST {
    return parseAstFields(input, ASTFields.service);
}

/**
 * Helper function to be called from `parse...AST`
 *
 * @param input the source string to parse
 * @param parseColumn is a callback which is invoked with each column which is encountered.
 *     Its arguments are the column's name and the cursor to parse the column's expression.
 */
function parseAst(input: string, parseColumn: (column: string, cursor: Cursor) => void) {
    const tokens = tokenize(input);
    const cursor = new Cursor(tokens);
    while (true) {
        const token = cursor.nextToken();
        if (token === null) break;

        if (token.type === "column") parseColumn(token.value, cursor);
        else throw new ParserError({ type: "unexpectedToken", exp: "column", got: token });
    }
}

/** Parse an {@link Expr.Or} expression using a `parseValue` to parse the leaves */
function parseOr<T>(tokens: Cursor, parseValue: (cursor: Cursor) => Expr.Value<T>): Expr.Or<T> {
    const list = [parseAnd(tokens, parseValue)];
    while (tokens.peekToken()?.type === "logicalOr") {
        tokens.nextToken(); // Consume the `,`
        list.push(parseAnd(tokens, parseValue));
    }
    return { or: list };
}

/** Parse an {@link Expr.And} expression using a `parseValue` to parse the leaves */
function parseAnd<T>(tokens: Cursor, parseValue: (cursor: Cursor) => Expr.Value<T>): Expr.And<T> {
    const list = [parseNot(tokens, parseValue)];
    while (tokens.peekToken()?.type === "logicalAnd") {
        tokens.nextToken(); // Consume the `&`
        list.push(parseNot(tokens, parseValue));
    }
    return { and: list };
}

/** Parse a {@link Expr.Not} using a `parseValue` to parse the potentially negated value */
function parseNot<T>(tokens: Cursor, parseValue: (cursor: Cursor) => Expr.Value<T>): Expr.Not<T> {
    let not = false;
    if (tokens.peekToken()?.type === "logicalNot") {
        tokens.nextToken(); // Consume the `!`
        not = true;
    }
    return { not, value: parseValue(tokens) };
}

/** Parse a single string */
export function parseString(tokens: Cursor): Expr.Value<string> {
    return tokens.nextValue();
}

/** Parse a single {@link Date} */
export function parseDate(tokens: Cursor): Expr.Value<Date> {
    const value = tokens.nextValue();
    const timestamp = Date.parse(value);
    if (Number.isNaN(timestamp)) throw new ParserError({ type: "parseValue", msg: `${value} is not a date` });
    else return new Date(timestamp);
}

/** Parse a single port i.e. a number in the range `1..65535` */
export function parsePort(tokens: Cursor): Expr.Value<number> {
    const value = tokens.nextValue();
    const number = Number(value);
    if (Number.isNaN(number) || number <= 0 || number > 65535)
        throw new ParserError({ type: "parseValue", msg: `${value} is not a valid port` });
    else return number;
}

/** Parse a single {@link PortProtocol} */
export function parsePortProtocol(tokens: Cursor): Expr.Value<PortProtocol> {
    const value = tokens.nextValue();
    switch (value.toLowerCase()) {
        case "tcp":
            return PortProtocol.Tcp;
        case "udp":
            return PortProtocol.Udp;
        case "sctp":
            return PortProtocol.Sctp;
        case "unknown":
            return PortProtocol.Unknown;
        default:
            throw new ParserError({ type: "parseValue", msg: `Unknown port protocol: ${value}` });
    }
}

/** Parse a single {@link OsType} */
export function parseOsType(tokens: Cursor): Expr.Value<OsType> {
    const value = tokens.nextValue();
    switch (value.toLowerCase()) {
        case "unknown":
            return OsType.Unknown;
        case "linux":
            return OsType.Linux;
        case "windows":
            return OsType.Windows;
        case "apple":
            return OsType.Apple;
        case "android":
            return OsType.Android;
        case "freebsd":
            return OsType.FreeBsd;
        default:
            throw new ParserError({ type: "parseValue", msg: `Unknown OS type: ${value}` });
    }
}

/** Wraps a `(cursor: Cursor) => Expr.Value<T>` to produce a `(cursor: Cursor) => Expr.Value<Expr.Range<T>>` */
export function wrapRange<T>(parseValue: (cursor: Cursor) => Expr.Value<T>) {
    return function (cursor: Cursor): Expr.Value<Expr.Range<T>> {
        const start = cursor.peekToken()?.type === "rangeOperator" ? null : parseValue(cursor);

        const range = cursor.nextToken();
        if (range === null) {
            throw new ParserError({ type: "unexpectedEnd" });
        } else if (range.type !== "rangeOperator") {
            throw new ParserError({ type: "unexpectedToken", exp: "rangeOperator", got: range });
        }

        let end;
        try {
            const cursor2 = cursor.clone();
            end = parseValue(cursor2);
            cursor.set(cursor2);
        } catch (e) {
            if (e instanceof ParserError) end = null;
            else throw e;
        }

        return {
            start,
            end,
        };
    };
}

/** Wraps a `(cursor: Cursor) => Expr.Value<T>` to produce a `(cursor: Cursor) => Expr.Value<Expr.MaybeRange<T>>` */
export function wrapMaybeRange<T>(parseValue: (cursor: Cursor) => Expr.Value<T>) {
    const parseRange = wrapRange(parseValue);
    return function (cursor: Cursor): Expr.Value<Expr.MaybeRange<T>> {
        const cursor2 = cursor.clone();
        try {
            const range = parseRange(cursor2);
            cursor.set(cursor2);
            return range;
        } catch (e) {
            if (e instanceof ParserError) return parseValue(cursor);
            else throw e;
        }
    };
}
