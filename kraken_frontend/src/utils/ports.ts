import { PortOrRange } from "../api/generated/models/PortOrRange";
import { Err, Ok, Result } from "./result";

/**
 * Parses a user input of a single or multiple ports in format `PORT` or
 * `FROM-TO`, concatenated with spaces or commas.
 *
 * @param input the user's input to parse
 * @returns The parsed ports/ranges or a human-readable error message
 */
export function parseUserPorts(input: string): Result<PortOrRange[], string> {
    const parts = input.split(/[, ]+/g);
    try {
        return Ok(
            parts.map((part) => {
                const p = parseUserPort(part);
                if (p !== false) return p;
                const parts = part.split("-");
                if (parts.length != 2)
                    throw part == "-"
                        ? "Don't use white-space inside port ranges, e.g. use `from-to` over `from - to`"
                        : "Invalid part '" + part + "' - must be in form `from-to`";
                const lower = parseUserPort(parts[0]);
                if (lower === false) throw "Malformed lower port number in range '" + part + "'";
                const upper = parseUserPort(parts[1]);
                if (upper === false) throw "Malformed upper port number in range '" + part + "'";
                return lower + "-" + upper;
            }),
        );
    } catch (e) {
        return Err("" + e);
    }
}

/**
 * Parses a single port (1-65535) from a string or returns `false`.
 *
 * @param input string containing a port number
 * @returns the parsed port (1-65535) or `false` in case of invalid `input`
 */
export function parseUserPort(input: string): number | false {
    const p = Number(input.trim());
    if (p !== null && Number.isSafeInteger(p) && p >= 1 && p <= 65535) return p;
    else return false;
}
