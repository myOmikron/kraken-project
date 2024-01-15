import { PortOrRange } from "../api/generated/models/PortOrRange";
import { Err, Ok, Result } from "./result";

/// Parses a user input of a single or multiple ports in format `PORT` or
/// `FROM-TO`, concatenated with spaces or commas.
///
/// Returns: `Err("human readable error message")` in case of error.
export function parseUserPorts(input: string): Result<PortOrRange[], string> {
    let parts = input.split(/[, ]+/g);
    try
    {
        return Ok(parts.map(part => {
            let p = parseUserPort(part);
            if (p !== false)
                return p;
            let parts = part.split('-');
            if (parts.length != 2)
                throw part == '-'
                    ? "Don't use white-space inside port ranges, e.g. use `from-to` over `from - to`"
                    : "Invalid part '" + part + "' - must be in form `from-to`";
            let lower = parseUserPort(parts[0])
            if (lower === false)
                throw "Malformed lower port number in range '" + part + "'";
            let upper = parseUserPort(parts[1])
            if (upper === false)
                throw "Malformed upper port number in range '" + part + "'";
            return lower + "-" + upper;
        }));
    }
    catch (e)
    {
        return Err("" + e);
    }
}

/// Given the input string coming from a user, parse the number and check that
/// it's within 1-65535. In case of error, return `false`, otherwise the input
/// cleaned up and converted to a number.
export function parseUserPort(input: string): number | false {
    let p = Number(input.trim());
    if (p !== null && Number.isSafeInteger(p) && (p >= 1 && p <= 65535))
        return p;
    else
        return false;
}