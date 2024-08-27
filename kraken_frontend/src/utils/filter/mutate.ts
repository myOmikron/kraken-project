import CONSOLE from "../console";
import { Span, SpanlessToken, Token, TokenType, tokenize, tokensToString, valueToString } from "./lexer";

/**
 * Given an input `filter`, return the span of its whole value starting after
 * the colon `:` up until the next column.
 *
 * @param filter The filter to operate on.
 * @param columnName The column name to search for.
 * @returns The full value span or undefined if the column was not found.
 */
function findColumnValueSpan(filter: string, columnName: string): { column: Span; value: Span } | undefined {
    if (!filter.length) return undefined;

    const tokens = tokenize(filter);
    let column: Span | undefined = undefined;
    let endIndex = -1;
    let last: Token | undefined;
    for (const token of tokens) {
        if (column === undefined && token.type == "column" && token.value == columnName) {
            column = token.span;
        } else if (column !== undefined && token.type == "column") {
            if (!last) throw new Error("logic error: last should always be defined here");
            endIndex = last.span.end;
            break;
        }
        last = token;
    }

    if (column === undefined) return undefined;
    if (endIndex == -1) endIndex = filter.length;

    return {
        column,
        value: {
            start: column.end,
            end: endIndex,
        },
    };
}

export function addExprs(filter: string, column: string, value: string, op: "or" | "and"): string {
    if (!filter.length) return prettifyFilter(column + ":" + valueToString(value));

    const span = findColumnValueSpan(filter, column)?.value;
    if (!span) return prettifyFilter(filter + " " + column + ":" + valueToString(value));

    return prettifyFilter(
        filter.substring(0, span.start) +
            insertValue(filter.substring(span.start, span.end), value, op) +
            filter.substring(span.end),
    );
}

export function addExprRange(filter: string, column: string, from: string, to: string, op: "or" | "and"): string {
    if (!filter.length) return column + ":" + valueToString(from) + "-" + valueToString(to);

    const span = findColumnValueSpan(filter, column)?.value;
    if (!span) return prettifyFilter(filter + " " + column + ":" + valueToString(from) + "-" + valueToString(to));

    return prettifyFilter(
        filter.substring(0, span.start) +
            insertRange(filter.substring(span.start, span.end), [from, to], op) +
            filter.substring(span.end),
    );
}

export function removeExprs(filter: string, column: string, value: string): string {
    const span = findColumnValueSpan(filter, column);
    CONSOLE.log(filter, span);
    if (!span) return filter;

    let newValue = "";
    newValue = removeValue(filter.substring(span.value.start, span.value.end), value);
    if (!newValue) return prettifyFilter(filter.substring(0, span.column.start) + filter.substring(span.value.end));

    return prettifyFilter(filter.substring(0, span.value.start) + newValue + filter.substring(span.value.end));
}

export function removeExprRange(filter: string, column: string, from: string, to: string): string {
    const span = findColumnValueSpan(filter, column);
    if (!span) return filter;

    let newValue = "";
    newValue = removeRange(filter.substring(span.value.start, span.value.end), [from, to]);
    if (!newValue) return prettifyFilter(filter.substring(0, span.column.start) + filter.substring(span.value.end));

    return prettifyFilter(filter.substring(0, span.value.start) + newValue + filter.substring(span.value.end));
}

export function getExprs(filter: string, column: string): SpanlessToken[] | undefined {
    const span = findColumnValueSpan(filter, column);
    if (!span) return undefined;

    return tokenize(filter.substring(span.value.start, span.value.end)) as SpanlessToken[];
}

export function replaceRaw(filter: string, column: string, raw: string): string {
    const hasValue = raw.trim() != "";
    const span = findColumnValueSpan(filter, column);
    if (!span) return hasValue ? prettifyFilter(filter + " " + column + ":" + raw) : filter;
    return prettifyFilter(
        hasValue
            ? filter.substring(0, span.value.start) + raw + filter.substring(span.value.end)
            : filter.substring(0, span.column.start) + filter.substring(span.value.end),
    );
}

function insertRange(existing: string, addValue: [string, string], op: "or" | "and"): string {
    const tokens = tokenize(existing) as SpanlessToken[];
    if (
        tokens.some(
            (t, i) =>
                t.type == "value" &&
                t.value == addValue[0] &&
                tokens[i + 1]?.type == "rangeOperator" &&
                tokens[i + 2]?.type == "value" &&
                (tokens[i + 2] as { value?: string })?.value == addValue[1],
        )
    )
        return existing;

    if (op == "or") {
        return existing + ", " + valueToString(addValue[0]) + "-" + valueToString(addValue[1]);
    } else if (op == "and") {
        const insertTokens: SpanlessToken[] = [
            { type: "logicalAnd" },
            { type: "value", value: addValue[0] },
            { type: "rangeOperator" },
            { type: "value", value: addValue[1] },
        ];
        tokens.push(...insertTokens);
        for (let i = tokens.length - 1; i >= 0; i--) {
            if (tokens[i].type == "logicalOr") {
                tokens.splice(i, 0, ...insertTokens);
            }
        }
        return tokensToString(tokens);
    } else {
        throw new Error("invalid operator");
    }
}

function insertValue(existing: string, addValue: string, op: "or" | "and"): string {
    const tokens = tokenize(existing) as SpanlessToken[];
    if (tokens.some((t) => t.type == "value" && t.value == addValue)) return existing;

    if (op == "or") {
        return existing + ", " + valueToString(addValue);
    } else if (op == "and") {
        tokens.push({ type: "logicalAnd" }, { type: "value", value: addValue });
        for (let i = tokens.length - 1; i >= 0; i--) {
            if (tokens[i].type == "logicalOr") {
                tokens.splice(i, 0, { type: "logicalAnd" }, { type: "value", value: addValue });
            }
        }
        return tokensToString(tokens);
    } else {
        throw new Error("invalid operator");
    }
}

function removeValue(existing: string, value: string): string {
    const tokens = tokenize(existing) as SpanlessToken[];
    let modified = false;
    for (let i = tokens.length - 1; i >= 0; i--) {
        const token = tokens[i];
        if (token.type == "value" && token.value == value && tokens[i - 1]?.type != "rangeOperator") {
            let removeCount = 1;
            const ti = i;
            if (tokens[i - 1]?.type == "logicalNot") {
                i--;
                removeCount++;
            }
            if (tokens[i + 1]?.type == "rangeOperator" && tokens[i + 2]?.type == "value") {
                continue;
            }
            const precedence: { [type in TokenType | ""]?: number } = {
                logicalAnd: 1,
                logicalOr: 2,
            };
            const leftType = i > 0 ? tokens[i - 1].type : undefined;
            const rightType = ti + 1 < tokens.length ? tokens[ti + 1].type : undefined;
            if ((precedence[leftType || ""] ?? -1) > (precedence[rightType || ""] ?? -1)) {
                if (i > 0) {
                    i--;
                    removeCount++;
                }
            } else if (rightType !== undefined) {
                removeCount++;
            }
            tokens.splice(i, removeCount);
            modified = true;
        }
    }
    return modified ? tokensToString(tokens) : existing;
}

function removeRange(existing: string, range: [string, string]): string {
    const tokens = tokenize(existing) as SpanlessToken[];
    CONSOLE.log("removeRange", tokens, range);
    let modified = false;
    for (let i = tokens.length - 3; i >= 0; i--) {
        const token = tokens[i];
        const toToken = tokens[i + 2];
        if (
            token.type == "value" &&
            token.value == range[0] &&
            tokens[i + 1].type == "rangeOperator" &&
            toToken.type == "value" &&
            toToken.value == range[1]
        ) {
            let removeCount = 3;
            const ti = i + 2;
            if (tokens[i - 1]?.type == "logicalNot") {
                i--;
                removeCount++;
            }
            const precedence: { [type in TokenType | ""]?: number } = {
                logicalAnd: 1,
                logicalOr: 2,
            };
            const leftType = i > 0 ? tokens[i - 1].type : undefined;
            const rightType = ti + 1 < tokens.length ? tokens[ti + 1].type : undefined;
            if ((precedence[leftType || ""] ?? -1) > (precedence[rightType || ""] ?? -1)) {
                if (i > 0) {
                    i--;
                    removeCount++;
                }
            } else if (rightType !== undefined) {
                removeCount++;
            }
            tokens.splice(i, removeCount);
            modified = true;
        }
    }
    return modified ? tokensToString(tokens) : existing;
}

export function prettifyFilter(filter: string): string {
    return tokensToString(tokenize(filter));
}
