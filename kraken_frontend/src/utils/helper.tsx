import { toast } from "react-toastify";
import { Result } from "./result";
import { ApiError } from "../api/error";
import { inspectError } from "../context/user";
import { OsType } from "../api/generated";
import AnonymousIcon from "../svg/anonymous";
import TuxIcon from "../svg/tux";
import AppleIcon from "../svg/apple";
import WindowsIcon from "../svg/windows";
import FreeBSDIcon from "../svg/freebsd";
import AndroidIcon from "../svg/android";
import React from "react";

export namespace ObjectFns {
    /** {@link ObjectConstructor.keys `Object.keys`} which preserves the keys' type */
    export function keys<Key extends string>(obj: Record<Key, any>): Array<Key> {
        // @ts-ignore
        return Object.keys(obj);
    }

    /** {@link ObjectConstructor.entries `Object.entries`} which preserves the keys' type */
    export function entries<Key extends string, Value>(obj: Record<Key, Value>): Array<[Key, Value]> {
        // @ts-ignore
        return Object.entries(obj);
    }

    /** {@link ObjectConstructor.fromEntries `Object.fromEntries`} which preserves the keys' type */
    export function fromEntries<Key extends string, Value>(arr: Array<[Key, Value]>): Record<Key, Value> {
        // @ts-ignore
        return Object.fromEntries(arr);
    }

    export function isEmpty(obj: Record<string, any>): boolean {
        for (const key in obj) {
            return false;
        }
        return true;
    }

    export function len(obj: Record<string, any>): number {
        let len = 0;
        for (const _ in obj) {
            len += 1;
        }
        return len;
    }
}

/**
 * Sleeps x milliseconds async
 *
 * @param timeout Sleep time in milliseconds
 */
export function sleep(timeout: number): Promise<null> {
    return new Promise((resolve) => setTimeout(resolve, timeout));
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

/**
 * Produce a function which accepts and handles an api {@link Result} to be passed to {@link Promise.then}.
 * The optional `then` argument will be called when the returned function receives an `Ok` value.
 *
 * # Example
 * ```js
 * Api.some.call().then(
 *     handleApiError((okValue) => {
 *         // do something
 *     })
 * );
 * ```
 *
 * @param then function to execute when the API returned an ok
 * @return a function which should be passed to a `Promise.then`
 */
export function handleApiError<T>(then?: (ok: T) => any): (result: Result<T, ApiError>) => void;
/**
 * Take a {@link Result} from the api and handle the `Err` case.
 * When the case is `Ok` call the optional `then` argument with it.
 *
 * # Example
 * ```js
 * const result = produce_some_result();
 * handleApiError(result, (okValue) => {
 *     // do something
 * });
 * ```
 *
 * @param result {@link Result} to handle
 * @param then optional function to call in the `Ok` case
 */
export function handleApiError<T>(result: Result<T, ApiError>, then?: (ok: T) => any): void;
export function handleApiError<T>(then_or_result?: ((ok: T) => any) | Result<T, ApiError>, then?: (ok: T) => any): any {
    if (then_or_result !== undefined && then_or_result instanceof Result) {
        then_or_result.match(then || noop, handleError);
    } else {
        if (then_or_result === undefined) return noopHandler;
        else return (result: Result<T, ApiError>) => result.match(then_or_result, handleError);
    }
}
function handleError(error: ApiError) {
    inspectError(error);
    toast.error(error.message);
}
function noop<T>(_ok: T) {}
function noopHandler<T>(result: Result<T, ApiError>) {
    result.match(noop, handleError);
}

export async function copyToClipboard(text: string | null) {
    if (window.isSecureContext && navigator.clipboard) {
        if (typeof text === "string") {
            await navigator.clipboard.writeText(text).then(() => {
                toast.success("Copied to clipboard");
            });
        }
    }
}
