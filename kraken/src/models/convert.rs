//! Specialized version of `From` and `Into` with a more concrete semantic.

/// Specialized version of [`From`] which converts between two types which represent the same value,
/// in a `rorm` friendly and an API friendly way.
pub trait FromDb {
    /// Type used with the database which represents `Self`.
    ///
    /// Depending on `Self` this is probably:
    /// - a [`Patch`] containing all fields required for `Self`
    /// - a [`DbEnum`] containing the same variants as `Self`
    /// - some primitive (example `i32`) which stores `Self` in a very compact form
    type DbFormat;

    /// Convert to this type from its db type
    fn from_db(db_format: Self::DbFormat) -> Self;
}

/// Specialized version of [`Into`] which converts between two types which represent the same value,
/// in a `rorm` friendly and an API friendly way.
pub trait IntoDb: FromDb {
    /// Converts this type into its db type
    fn into_db(self) -> Self::DbFormat;
}
