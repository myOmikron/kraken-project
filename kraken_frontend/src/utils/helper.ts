import { toast } from "react-toastify";

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
