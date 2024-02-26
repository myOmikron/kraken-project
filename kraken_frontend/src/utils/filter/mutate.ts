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
    if (!filter.length) return column + ":" + valueToString(value);

    let span = findColumnValueSpan(filter, column)?.value;
    if (!span) return filter + " " + column + ":" + valueToString(value);

    return (
        filter.substring(0, span.start) +
        insertValue(filter.substring(span.start, span.end), value, op) +
        filter.substring(span.end)
    );
}

export function removeExprs(filter: string, column: string, value: string): string {
    let span = findColumnValueSpan(filter, column);
    if (!span) return filter;

    let newValue = removeValue(filter.substring(span.value.start, span.value.end), value);
    if (!newValue) return filter.substring(0, span.column.start) + filter.substring(span.value.end);

    return filter.substring(0, span.value.start) + newValue + filter.substring(span.value.end);
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

function removeValue(existing: string, value: string): string {
    let tokens = tokenize(existing) as SpanlessToken[];
    let modified = false;
    for (let i = tokens.length - 1; i >= 0; i--) {
        let token = tokens[i];
        if (token.type == "value" && token.value == value) {
            let removeCount = 1;
            let ti = i;
            if (i > 0 && tokens[i - 1].type == "logicalNot") {
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
