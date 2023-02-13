type InnerOption<T> =
    | {
          isSome: true;
          value: T;
      }
    | {
          isSome: false;
      };

/**
 * A [Rust](https://doc.rust-lang.org/std/option/enum.Option.html) inspired type for optional values
 *
 * The `Option` type is a nice representation for a nullable value.
 */
export class Option<T> {
    private inner: InnerOption<T>;

    private constructor(inner: InnerOption<T>) {
        this.inner = inner;
    }

    /**
     * Returns `true` if the option is a `Some` value.
     */
    is_some(): boolean {
        return this.inner.isSome;
    }

    /**
     * Returns `true` if the option is a `None` value.
     */
    is_none(): boolean {
        return !this.inner.isSome;
    }

    if_let_some(then: (value: T) => undefined) {
        if (this.inner.isSome) {
            return then(this.inner.value);
        }
    }

    /**
     * Maps an `Option<T>` to `Option<U>` by applying a function to a contained value.
     *
     * @param func
     */
    map<U>(func: (value: T) => U): Option<U> {
        if (this.inner.isSome) {
            return Some(func(this.inner.value));
        } else {
            return None();
        }
    }

    /**
     * Returns the provided default result (if none), or applies a function to the contained value (if any).
     *
     * Arguments passed to `map_or` are eagerly evaluated;
     * if you are passing the result of a function call,
     * it is recommended to use `map_or_else`, which is lazily evaluated.
     *
     * @param default_
     * @param func
     */
    map_or<U>(default_: U, func: (value: T) => U): U {
        if (this.inner.isSome) {
            return func(this.inner.value);
        } else {
            return default_;
        }
    }

    /**
     * Computes a default function result (if none), or applies a different function to the contained value (if any).
     *
     * @param default_
     * @param func
     */
    map_or_else<U>(default_: () => U, func: (value: T) => U): U {
        if (this.inner.isSome) {
            return func(this.inner.value);
        } else {
            return default_();
        }
    }

    /**
     * Returns the contained `Some` value.
     *
     * Because this function may panic, its use is generally discouraged.
     * Instead, prefer to use pattern matching and handle the `None` case explicitly,
     * or call `unwrap_or`, `unwrap_or_else`, or `unwrap_or_default`.
     */
    unwrap(): T {
        if (this.inner.isSome) {
            return this.inner.value;
        } else {
            throw new Error("called `Option::unwrap()` on a `None` value");
        }
    }

    /**
     * Returns the contained `Some` value or a provided default.
     *
     * Arguments passed to `unwrap_or` are eagerly evaluated;
     * if you are passing the result of a function call,
     * it is recommended to use `unwrap_or_else`, which is lazily evaluated.
     *
     * @param default_
     */
    unwrap_or(default_: T): T {
        if (this.inner.isSome) {
            return this.inner.value;
        } else {
            return default_;
        }
    }

    /**
     * Returns the contained `Some` value or computes it from a closure.
     *
     * @param default_
     */
    unwrap_or_else(default_: () => T): T {
        if (this.inner.isSome) {
            return this.inner.value;
        } else {
            return default_();
        }
    }

    /**
     * Constructs "some value" of type T.
     *
     * @param value
     * @constructor
     */
    public static Some<T, E>(value: T): Option<T> {
        return new Option<T>({ isSome: true, value });
    }

    /**
     * Constructs "no value".
     *
     * @constructor
     */
    public static None<T>(): Option<T> {
        return new Option<T>({ isSome: false });
    }
}

export const Some = Option.Some;
export const None = Option.None;
