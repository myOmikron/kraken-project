import ParserError from "./error";
import { Token } from "./lexer";

/**
 * An iterator over `Token` specialized for our parser
 *
 * - use `nextToken` as shorthand for `next_token` with the additional check
 */
export class Cursor {
    protected array: ReadonlyArray<Token>;
    protected position: number;

    /** Construct a cursor from an array of tokens */
    public constructor(array: ReadonlyArray<Token>, position?: number) {
        this.array = array;
        this.position = position || 0;
    }

    /**
     * Duplicate the cursor without copying the underlying array
     *
     * Use this method together with `set` to implement forking branches in the parser.
     */
    public clone(): Cursor {
        return new Cursor(this.array, this.position);
    }

    /**
     * Overwrite `this` cursor with a potentially changed duplicate
     *
     * This method is the substitute of a pointer write i.e. `*cursor = other;`
     *
     * Use this method together with `clone` to implement forking branches in the parser.
     */
    public set(other: Cursor) {
        this.array = other.array;
        this.position = other.position;
    }

    /** Yield the next token without advancing the cursor */
    public peekToken(): Readonly<Token> | null {
        if (this.position >= this.array.length) return null;

        return this.array[this.position];
    }

    /** Yield the next token and advance the cursor */
    public nextToken(): Readonly<Token> | null {
        if (this.position >= this.array.length) return null;

        const token = this.array[this.position];
        this.position += 1;
        return token;
    }

    /**
     * Yield the next token, check it to be a [`Token::Value`] and advance the cursor
     *
     * @throws ParserError of the `"unexpectedEnd"` type if all tokens have been consumed
     * @throws ParserError of the `"unexpectedToken"` type if the next token was not of type `"value"`
     */
    public nextValue(): string {
        const token = this.nextToken();
        if (token === null) {
            throw new ParserError({ type: "unexpectedEnd" });
        } else if (token.type !== "value") {
            throw new ParserError({ type: "unexpectedToken", exp: "value", got: token });
        } else {
            return token.value;
        }
    }
}
