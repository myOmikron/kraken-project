/** The inner representation of a {@link Result}*/
type InnerResult<T, E> =
    | {
          /** Marks this result as `Ok` */
          isErr: false;
          /** The `Ok`'s value */
          ok: T;
      }
    | {
          /** Marks this result as `Err` */
          isErr: true;
          /** The `Err`'s value */
          err: E;
      };

/**
 * A [Rust](https://doc.rust-lang.org/std/result/enum.Result.html) inspired type for error handling
 *
 * `Result` is a type that represents either success (`Ok`) or failure (`Err`).
 */
export class Result<T, E> {
    private inner: InnerResult<T, E>;

    // eslint-disable-next-line jsdoc/require-param, jsdoc/require-returns
    /** Basic constructor use {@link Ok} or {@link Err} instead */
    private constructor(inner: InnerResult<T, E>) {
        this.inner = inner;
    }

    /**
     * Calls `func` if the result is `Ok`, otherwise returns the `Err` value of `this`.
     *
     * This function can be used for control flow based on `Result` values.
     *
     * @param func function to process the `OK` value
     * @returns either the old `Err` or the new result returned by `func`
     */
    and_then<U>(func: (ok: T) => Result<U, E>): Result<U, E> {
        if (this.inner.isErr) {
            return Err(this.inner.err);
        } else {
            return func(this.inner.ok);
        }
    }

    /**
     * Returns `true` if the result is `Ok`.
     *
     * @returns `true` if the result is `Ok`
     */
    is_ok(): boolean {
        return !this.inner.isErr;
    }

    /**
     * Returns `true` if the result is `Err`.
     *
     * @returns `true` if the result is `Err`
     */
    is_err(): boolean {
        return this.inner.isErr;
    }

    /**
     * Maps a `Result<T, E>` to `Result<U, E>` by applying a function to a contained `Ok` value, leaving an `Err` value untouched.
     *
     * This function can be used to compose the results of two functions.
     *
     * @param func function to apply to the `Ok`'s value
     * @returns a result with a new `Ok` value
     */
    map<U>(func: (ok: T) => U): Result<U, E> {
        if (this.inner.isErr) {
            return Err(this.inner.err);
        } else {
            return Ok(func(this.inner.ok));
        }
    }

    /**
     * Maps a `Result<T, E>` to `Result<T, F>` by applying a function to a contained `Err` value, leaving an `Ok` value untouched.
     *
     * This function can be used to pass through a successful result while handling an error.
     *
     * @param func function to apply to the `Err`'s value
     * @returns a result with a new `Err` value
     */
    map_err<F>(func: (err: E) => F): Result<T, F> {
        if (this.inner.isErr) {
            return Err(func(this.inner.err));
        } else {
            return Ok(this.inner.ok);
        }
    }

    /**
     * Runs one of two functions based on the result's variant.
     *
     * @param ok function to execute in case of `Ok`
     * @param err function to execute in case of `Err`
     */
    match(ok: (ok: T) => void, err: (err: E) => void) {
        if (this.inner.isErr) {
            err(this.inner.err);
        } else {
            ok(this.inner.ok);
        }
    }

    /**
     * Returns the contained `Ok` value, consuming the `this` value.
     *
     * Because this method may panic, its use is generally discouraged.
     * Instead, prefer to use pattern matching and handle the Err case explicitly,
     * or call `unwrap_or` or `unwrap_or_else`.
     *
     * @throws Error if the value is an `Err`, with an error message provided by the `Err`’s `toString()` implementation.
     * @returns the `Ok`'s value
     */
    unwrap(): T {
        if (this.inner.isErr) {
            throw new Error("" + this.inner.err);
        } else {
            return this.inner.ok;
        }
    }

    /**
     * Returns the contained `Ok` value or a provided `default_`.
     *
     * Arguments passed to `unwrap_or` are eagerly evaluated; if you are passing the result of a function call,
     * it is recommended to use unwrap_or_else, which is lazily evaluated.
     *
     * @param default_ default value to return in case of `Err`
     * @returns the `Ok`'s value or the `default_`
     */
    unwrap_or(default_: T): T {
        if (this.inner.isErr) {
            return default_;
        } else {
            return this.inner.ok;
        }
    }

    /**
     * Returns the contained `Ok` value or computes it using a function.
     *
     * @param default_ function computing the return value in case of `Err`
     * @returns the `Ok`'s value or the return from `default_`
     */
    unwrap_or_else(default_: () => T): T {
        if (this.inner.isErr) {
            return default_();
        } else {
            return this.inner.ok;
        }
    }

    /**
     * Constructs an `Ok` result
     *
     * @param ok the `Ok`'s value
     * @returns an `Ok` result
     */
    public static Ok<T, E>(ok: T): Result<T, E> {
        return new Result<T, E>({ isErr: false, ok });
    }

    /**
     * Constructs an `Err` result
     *
     * @param err the `Err`'s value
     * @returns an `Err` result
     */
    public static Err<T, E>(err: E): Result<T, E> {
        return new Result<T, E>({ isErr: true, err });
    }
}

/** {@inheritDoc Result#Ok} */
export const Ok = Result.Ok;
/** {@inheritDoc Result#Err} */
export const Err = Result.Err;
