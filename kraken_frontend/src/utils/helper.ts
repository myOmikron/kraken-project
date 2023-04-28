import { toast } from "react-toastify";
import { Result } from "./result";
import { ApiError } from "../api/error";

/**
 * Sleeps x milliseconds async
 *
 * @param timeout Sleep time in milliseconds
 */
export async function sleep(timeout: number): Promise<null> {
    return await new Promise((resolve) => setTimeout(resolve, timeout));
}

/**
 * Take a list of checks and return true if all checks are true
 *
 * For any false check, toast the provided error message.
 */
export function check(checks: Array<[boolean, string]>): boolean {
    let ok = true;
    for (const [check, error] of checks) {
        if (!check) {
            toast.error(error);
            ok = false;
        }
    }
    return ok;
}

function noop<T>(_ok: T) {}
/**
 * Small function to abstract the common error handling pattern of do something on ok or toast the error
 *
 * # Example
 * ```js
 * Api.some.call().then(
 *     okOrToast((okValue) => {
 *         // do something
 *     })
 * );
 * ```
 * ## Without this function
 * ```js
 * Api.some.call().then(
 *     (result) => result.match(
 *         (okValue) => {
 *             // do something
 *         },
 *         (error) => toast.error(error.message),
 *     )
 * );
 * ```
 *
 * @param then function to execute when the API returned an ok
 * @return a function which should be passed to a `Promise.then`
 */
export function okOrToast<T>(then?: (ok: T) => any): (result: Result<T, ApiError>) => void {
    return (result) => result.match(then || noop, (err) => toast.error(err.message));
}
