import ParserError from "./error";

/**
 * Split an input string into tokens
 *
 * @returns list of tokens
 * @throws ParserError of the `"lexer"` type if invalid characters are encountered
 */
export function tokenize(input: string): Array<Token> {
    const tokens = [];

    let remaining = input;
    outerLoop: while (remaining.length > 0) {
        for (const { regex, then } of RULES) {
            const match = remaining.match(regex);
            if (match !== null) {
                const [text] = match;
                const token = then(text, {
                    start: input.length - remaining.length,
                    end: input.length - remaining.length + text.length,
                });
                if (token !== null) tokens.push(token);

                remaining = remaining.substring(text.length);
                continue outerLoop;
            }
        }
        throw new ParserError({ type: "lexer", remaining, position: input.length - remaining.length });
    }

    return tokens;
}

/** The tokens produced by {@link tokenize} */
export type Token = SpanlessToken & { span: Span };

/** The type of a token */
export type TokenType = Token["type"];

/** Easier to generate sub-type of {@link Token} */
export type SpanlessToken =
    | { type: "column"; value: string }
    | { type: "value"; value: string }
    | { type: "logicalOr" }
    | { type: "logicalAnd" }
    | { type: "logicalNot" }
    | { type: "rangeOperator" };

/**
 * A span defines a substring by storing its position in the main string
 *
 * Its properties directly correspond to arguments to {@link String.substring `String.substring`}
 * @property start The index of the span's first character
 * @property end The index of the first character after the span
 */
export type Span = {
    /** The index of the span's first character */
    start: number;
    /** The index of the first character after the span */
    end: number;
};

const RULES: Array<TokenRule> = [
    {
        regex: /^ +/,
        then: () => null,
    },
    {
        regex: /^[^ ,&!:"-]+ *:/,
        then: (text, span) => ({ type: "column", value: text.substring(0, text.length - 1), span }),
    },
    {
        regex: /^[^ ,&!:"-]+/,
        then: (text, span) => ({ type: "value", value: text, span }),
    },
    {
        regex: /^"[^"]+"/,
        then: (text, span) => ({ type: "value", value: text.substring(1, text.length - 1), span }),
    },
    {
        regex: /^,/,
        then: (_, span) => ({ type: "logicalOr", span }),
    },
    {
        regex: /^&/,
        then: (_, span) => ({ type: "logicalAnd", span }),
    },
    {
        regex: /^!/,
        then: (_, span) => ({ type: "logicalNot", span }),
    },
    {
        regex: /^-/,
        then: (_, span) => ({ type: "rangeOperator", span }),
    },
];

type TokenRule = {
    regex: RegExp;
    then: (match: string, span: Span) => Token | null;
};

/**
 * Converts a list of tokens to a parsable string.
 *
 * @param tokens the tokens to convert to a parsable string.
 */
export function tokensToString(tokens: SpanlessToken[]): string {
    let ret = "";
    for (const token of tokens) {
        switch (token.type) {
            case "column":
                if (ret != "" && !ret.endsWith(" ")) ret += " ";
                ret += token.value + ": ";
                break;
            case "logicalAnd":
                ret += " & ";
                break;
            case "logicalNot":
                ret += "!";
                break;
            case "logicalOr":
                ret += ", ";
                break;
            case "rangeOperator":
                ret += " - ";
                break;
            case "value":
                ret += valueToString(token.value);
                break;
            default:
                throw new Error("unexpected token type!");
        }
    }
    return ret;
}

/**
 * Escapes a value for insertion into filter strings.
 *
 * @param value The value to escape.
 * @returns The value, either unquoted if it's fine or quoted if it needs to be
 * quoted.
 * @throws `Error` if the value is not representable
 */
export function valueToString(value: string): string {
    let existing = tokenize(value);
    if (existing.length != 1 || existing[0].type != "value") {
        value = '"' + value + '"';
        if (tokenize(value).length > 1) throw new Error("Value not representable in filter language!");
    }
    return value;
}
