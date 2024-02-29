import { Span, SpanlessToken, TokenType, tokenize, tokensToString, valueToString } from "./lexer";

/**
 * Given an input `filter`, return the span of its whole value starting after
 * the colon `:` up until the next column.
 * @param filter The filter to operate on.
 * @param column The column name to search for.
 * @returns The full value span or undefined if the column was not found.
 */
function findColumnValueSpan(filter: string, columnName: string): { column: Span; value: Span } | undefined {
    if (!filter.length) return undefined;

    let tokens = tokenize(filter);
    let column: Span | undefined = undefined;
    let endIndex = -1;
    for (const token of tokens) {
        if (!column && token.type == "column" && token.value == columnName) {
            column = token.span;
        } else if (column && token.type == "column") {
            endIndex = token.span.start;
        }
    }

    if (!column) return undefined;

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
    let range = tryParseRange(value);
    if (range) {
        if (!filter.length) return column + ":" + valueToString(range[0]) + "-" + valueToString(range[1]);

        let span = findColumnValueSpan(filter, column)?.value;
        if (!span) return filter + " " + column + ":" + valueToString(range[0]) + "-" + valueToString(range[1]);

        return (
            filter.substring(0, span.start) +
            insertRange(filter.substring(span.start, span.end), range, op) +
            filter.substring(span.end)
        );
    } else {
        if (!filter.length) return column + ":" + valueToString(value);

        let span = findColumnValueSpan(filter, column)?.value;
        if (!span) return filter + " " + column + ":" + valueToString(value);

        return (
            filter.substring(0, span.start) +
            insertValue(filter.substring(span.start, span.end), value, op) +
            filter.substring(span.end)
        );
    }
}

export function removeExprs(filter: string, column: string, value: string): string {
    let span = findColumnValueSpan(filter, column);
    if (!span) return filter;

    let range = tryParseRange(value);
    let newValue = "";
    if (range) {
        newValue = removeRange(filter.substring(span.value.start, span.value.end), range);
        if (!newValue) return filter.substring(0, span.column.start) + filter.substring(span.value.end);
    } else {
        newValue = removeValue(filter.substring(span.value.start, span.value.end), value);
        if (!newValue) return filter.substring(0, span.column.start) + filter.substring(span.value.end);
    }

    return filter.substring(0, span.value.start) + newValue + filter.substring(span.value.end);
}

function insertRange(existing: string, addValue: [string, string], op: "or" | "and"): string {
    let tokens = tokenize(existing) as SpanlessToken[];
    if (
        tokens.some(
            (t, i) =>
                t.type == "value" &&
                t.value == addValue[0] &&
                tokens[i + 1]?.type == "rangeOperator" &&
                tokens[i + 2]?.type == "value" &&
                (tokens as any)[i + 2]?.value == addValue[1],
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
    let tokens = tokenize(existing) as SpanlessToken[];
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

function tryParseRange(value: string): [string, string] | undefined {
    try {
        const tok = tokenize(value);
        if (tok.length == 3 && tok[0].type == "value" && tok[1].type == "rangeOperator" && tok[2].type == "value")
            return [tok[0].value, tok[2].value];
    } catch (e) {
        // ignore parse error on value
    }
    return undefined;
}

function removeValue(existing: string, value: string): string {
    let tokens = tokenize(existing) as SpanlessToken[];
    let modified = false;
    for (let i = tokens.length - 1; i >= 0; i--) {
        let token = tokens[i];
        if (token.type == "value" && token.value == value && tokens[i - 1]?.type != "rangeOperator") {
            let removeCount = 1;
            let ti = i;
            if (tokens[i - 1]?.type == "logicalNot") {
                i--;
                removeCount++;
            }
            if (tokens[i + 1]?.type == "rangeOperator" && tokens[i + 2]?.type == "value") {
                continue;
            }
            let precedence: { [type in TokenType | ""]?: number } = {
                logicalAnd: 1,
                logicalOr: 2,
            };
            let leftType = i > 0 ? tokens[i - 1].type : undefined;
            let rightType = ti + 1 < tokens.length ? tokens[ti + 1].type : undefined;
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
    let tokens = tokenize(existing) as SpanlessToken[];
    console.log("removeRange", tokens, range);
    let modified = false;
    for (let i = tokens.length - 3; i >= 0; i--) {
        let token = tokens[i];
        let toToken = tokens[i + 2];
        if (
            token.type == "value" &&
            token.value == range[0] &&
            tokens[i + 1].type == "rangeOperator" &&
            toToken.type == "value" &&
            toToken.value == range[1]
        ) {
            let removeCount = 3;
            let ti = i + 2;
            if (tokens[i - 1]?.type == "logicalNot") {
                i--;
                removeCount++;
            }
            let precedence: { [type in TokenType | ""]?: number } = {
                logicalAnd: 1,
                logicalOr: 2,
            };
            let leftType = i > 0 ? tokens[i - 1].type : undefined;
            let rightType = ti + 1 < tokens.length ? tokens[ti + 1].type : undefined;
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
