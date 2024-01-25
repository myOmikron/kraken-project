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
export type Token =
    | { span: Span; type: "column"; value: string }
    | { span: Span; type: "value"; value: string }
    | { span: Span; type: "logicalOr" }
    | { span: Span; type: "logicalAnd" }
    | { span: Span; type: "logicalNot" }
    | { span: Span; type: "rangeOperator" };

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
