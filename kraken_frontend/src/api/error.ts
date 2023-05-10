import { Err, Ok, Result } from "../utils/result";
import { RequiredError, ResponseError } from "./generated";

export enum StatusCode {
    ArbitraryJSError = -2,
    JsonDecodeError = -1,

    LoginFailed = 1000,
    NotFound = 1001,
    InvalidContentType = 1002,
    InvalidJson = 1003,
    PayloadOverflow = 1004,
    Unauthenticated = 1005,
    Missing2fa = 1006,
    MissingPrivileges = 1007,
    InternalServerError = 2000,
    DatabaseError = 2001,
    SessionError = 2002,
}

export type ApiError = {
    status_code: StatusCode;
    message: string;
};

/**
 * Wraps a promise returned by the generated SDK which handles its errors and returns a {@link Result}
 */
export async function handleError<T>(promise: Promise<T>): Promise<Result<T, ApiError>> {
    try {
        return Ok(await promise);
    } catch (e) {
        if (e instanceof ResponseError) {
            return Err(await parseError(e.response));
        } else if (e instanceof RequiredError) {
            return Err({
                status_code: StatusCode.JsonDecodeError,
                message: "The server's response didn't match the spec",
            });
        } else {
            console.error("Unknown error occurred:", e);
            return Err({
                status_code: StatusCode.ArbitraryJSError,
                message: "Unknown error occurred",
            });
        }
    }
}

/**
 * Parse a response's body into an {@link ApiError}
 *
 * This function assumes but doesn't check, that the response is an error.
 */
export async function parseError(response: Response): Promise<ApiError> {
    try {
        return await response.json();
    } catch {
        console.error("Got invalid json", response.body);
        return {
            status_code: StatusCode.JsonDecodeError,
            message: "The server's response was invalid json",
        };
    }
}
