import { Token } from "./lexer";

export type ErrorData =
    | { type: "lexer"; position: number; remaining: string }
    | { type: "unexpectedEnd" }
    | { type: "unexpectedToken"; got: Token; exp: Token["type"] }
    | { type: "unknownColumn"; column: string };
export default class ParserError extends Error {
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
                }
            })(),
        );
        this.data = data;
    }
}
