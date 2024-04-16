import { Token } from "./lexer";

/** The different types of {@link ParserError} and their unique data */
export type ErrorData =
    | { type: "lexer"; position: number; remaining: string }
    | { type: "parseValue"; msg: string }
    | { type: "unexpectedEnd" }
    | { type: "unexpectedToken"; got: Token; exp: Token["type"] }
    | { type: "unknownColumn"; column: string };

/**
 * An error encountered while parsing a filter ast
 */
export default class ParserError extends Error {
    /** The different types of {@link ParserError} and their unique data */
    data: ErrorData;

    constructor(data: ErrorData) {
        super(
            (function () {
                switch (data.type) {
                    case "lexer":
                        return `Invalid input @${data.position}: "${data.remaining}"`;
                    case "unexpectedEnd":
                        return "Unexpected end of string";
                    case "unexpectedToken":
                        return `Unexpected token: ${data.got.type}`;
                    case "unknownColumn":
                        return `Unknown column: ${data.column}`;
                    case "parseValue":
                        return `Failed to parse value: ${data.msg}`;
                }
            })(),
        );
        this.data = data;
    }
}
