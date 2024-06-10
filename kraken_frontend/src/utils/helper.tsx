import React from "react";
import { toast } from "react-toastify";
import { ApiError } from "../api/error";
import { inspectError } from "../context/user";
import { Result } from "./result";

export namespace ObjectFns {
    // eslint-disable-next-line jsdoc/require-param, jsdoc/require-returns
    /** {@link ObjectConstructor.keys `Object.keys`} which preserves the keys' type */
    export function keys<Key extends string>(obj: Partial<Record<Key, unknown>>): Array<Key> {
        // @ts-ignore: DOM type declaration aren't good enough, that's this function's whole point
        return Object.keys(obj);
    }

    // eslint-disable-next-line jsdoc/require-param, jsdoc/require-returns
    /** {@link ObjectConstructor.entries `Object.entries`} which preserves the keys' type */
    export function entries<Key extends string, Value>(obj: Partial<Record<Key, Value>>): Array<[Key, Value]> {
        // @ts-ignore: DOM type declaration aren't good enough, that's this function's whole point
        return Object.entries(obj);
    }

    // eslint-disable-next-line jsdoc/require-param, jsdoc/require-returns
    /** {@link Array.isArray} but for objects */
    export function isObject(obj: unknown): obj is object {
        return typeof obj === "object" && obj !== null && !Array.isArray(obj);
    }

    /**
     * Checks whether an object has any properties (including inherited ones)
     *
     * This function is slightly faster than checking {@link ObjectFns.len} to be zero.
     *
     * @param obj object to check
     * @returns `true` if the `obj` has no properties
     */
    export function isEmpty(obj: Record<string, unknown>): boolean {
        for (const _ in obj) {
            return false;
        }
        return true;
    }

    /**
     * Counts an object's properties (including inherited ones)
     *
     * @param obj object whose properties to count
     * @returns number of properties in `obj`
     */
    export function len(obj: Record<string, unknown>): number {
        let len = 0;
        for (const _ in obj) {
            len += 1;
        }
        return len;
    }

    /**
     * Checks two values for equality
     *
     * Unlike `===` this doesn't just compare pointers for object and arrays, but traverses them recursively.
     *
     * @param lhs value to compare with `rhs`
     * @param rhs value to compare with `lhs`
     * @returns `true` if `lhs` and `rhs` are equal
     */
    export function deepEquals(lhs: unknown, rhs: unknown): boolean {
        if (typeof lhs != typeof rhs) {
            return false;
        } else if (Array.isArray(lhs) && Array.isArray(rhs))
            return lhs.length == rhs.length && lhs.every((v, i) => deepEquals(v, rhs[i]));
        else if (ObjectFns.isObject(lhs) && ObjectFns.isObject(rhs)) {
            const lhsKeys = ObjectFns.keys(lhs);
            const rhsKeys = ObjectFns.keys(rhs);
            return lhsKeys.length == rhsKeys.length && lhsKeys.every((k) => deepEquals(lhs[k], rhs[k]));
        } else {
            return lhs === rhs;
        }
    }

    /**
     * Duplicates a value by recursively duplicating nested objects and arrays
     *
     * @param v value to duplicate
     * @returns new value
     */
    export function deepDuplicate<T>(v: T): T {
        if (typeof v === "object" && v !== null) {
            if (Array.isArray(v)) return v.map(deepDuplicate) as T;
            else {
                const ret: Partial<T> = {};
                for (const key of ObjectFns.keys(v)) ret[key] = deepDuplicate(v[key]);
                return ret as T;
            }
        } else {
            // make sure we keep delegates identical
            return v;
        }
    }

    /**
     * Creates a new array without duplicates
     *
     * Similar to `new Set(v).values()`, but using {@link deepEquals} instead of
     * reference checks for variables.
     *
     * Not very performant, only use for small-ish sets of data.
     *
     * @param array input array to create a deduplicated version for
     * @returns the input but with duplicate values removed.
     */
    export function uniqueObjects<T>(array: T[]): T[] {
        const res: T[] = [];
        for (const v of array) {
            let exists = false;
            for (const existing of res) {
                if (ObjectFns.deepEquals(existing, v)) {
                    exists = true;
                    break;
                }
            }
            if (!exists) res.push(v);
        }
        return res;
    }

    /**
     * For a 2-dimensional non-jagged array of size AxB, return its transposed
     * i.e. 90 degree rotated version of size BxA.
     *
     * @param array 2-dimensional non-jagged array
     * @returns array of shape `height x width` for input of shape `width x height`
     * @throws Error an Error if this is passed a jagged array.
     */
    export function transpose2D<T>(array: T[][]): T[][] {
        if (!array.length) return array;

        const w = array.length;
        const h = array[0].length;
        const ret = new Array(h);
        for (let i = 0; i < h; i++) ret[i] = new Array(w);
        for (const v of array) {
            if (v.length != h) throw new Error("passed in jagged array into transpose2D");
        }

        for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) ret[y][x] = array[x][y];

        return ret;
    }
}

/**
 * Sleeps x milliseconds async
 *
 * @param timeout Sleep time in milliseconds
 * @returns a promise which resolves after `timeout` milliseconds
 */
export function sleep(timeout: number): Promise<null> {
    return new Promise((resolve) => setTimeout(resolve, timeout));
}

/**
 * Take a list of checks and return true if all checks are true
 *
 * For any false check, toast the provided error message.
 *
 * @param checks a list of checks and their error message
 * @returns `true` if all checks have been `true`
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
 * @returns a function which should be passed to a `Promise.then`
 */
export function handleApiError<T>(then?: (ok: T) => void): (result: Result<T, ApiError>) => void;
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
export function handleApiError<T>(result: Result<T, ApiError>, then?: (ok: T) => void): void;
// eslint-disable-next-line jsdoc/require-jsdoc
export function handleApiError<T>(
    then_or_result?: ((ok: T) => void) | Result<T, ApiError>,
    then?: (ok: T) => void,
): ((result: Result<T, ApiError>) => void) | undefined {
    if (then_or_result !== undefined && then_or_result instanceof Result) {
        then_or_result.match(then || noop, handleError);
    } else {
        if (then_or_result === undefined) return noopHandler;
        else return (result: Result<T, ApiError>) => result.match(then_or_result, handleError);
    }
}

// eslint-disable-next-line jsdoc/require-param
/** Handles an {@link ApiError}, implementation detail of {@link handleApiError} */
function handleError(error: ApiError) {
    inspectError(error);
    toast.error(error.message);
}

// eslint-disable-next-line jsdoc/require-param
/** Callback which does nothing, implementation detail of {@link handleApiError} */
function noop<T>(_ok: T) {}

// eslint-disable-next-line jsdoc/require-param
/** Result handler which does nothing, implementation detail of {@link handleApiError} */
function noopHandler<T>(result: Result<T, ApiError>) {
    result.match(noop, handleError);
}

/**
 * Convenience function for copying some text to the user's clipboard
 *
 * This will do some checks and notify the user of success
 *
 * @param text the string to copy
 */
export async function copyToClipboard(text: string | null) {
    if (window.isSecureContext && navigator.clipboard) {
        if (typeof text === "string") {
            await navigator.clipboard.writeText(text).then(() => {
                toast.success("Copied to clipboard");
            });
        }
    }
}

/**
 * Hook which stabilizes the address of the object passed in
 *
 * I.e. you pass it any object (`{foo: 1}`) and it will return another object with the exact same fields,
 * but the returned object will always be the same one (have the same address) each re-render.
 *
 * ## Intended usage
 *
 * ```js
 * React.useEffect(() => {
 *     const handler = () => {
 *         // something using `someVar`
 *     };
 *     eventEmitter.addEventListener("event", handler);
 *     return () => eventEmitter.removeEventListener("event", handler);
 * }, [someVar]);
 *
 * // becomes
 *
 * const vars = useStableObj({
 *     someVar,
 * });
 * React.useEffect(() => {
 *     const handler = () => {
 *         // something using `vars.someVar`
 *     };
 *     eventEmitter.addEventListener("event", handler);
 *     return () => eventEmitter.removeEventListener("event", handler);
 * }, []); // <- no dependency here
 * ```
 *
 * @param obj object whose fields to copy into the stable object.
 * @returns stable object with the same properties as `obj`
 */
export function useStableObj<T extends Record<string, unknown>>(obj: T): T {
    const { current } = React.useRef(obj);
    for (const [key, value] of Object.entries(obj)) {
        // @ts-ignore TS2862: obj should only ever be an object literal which is mutable
        current[key] = value;
    }
    return current;
}

Object.defineProperty(useStableObj, "name", { value: "useStableObj" });
