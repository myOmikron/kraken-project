//! Small library for adding a `context` string to errors similar to anyhow

use std::borrow::Cow;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::ops::Deref;

/// Error of type `E` with a `context` string attached
#[derive(Debug)]
pub struct Extended<E: Error> {
    /// The extended error
    pub inner: E,

    /// String identifying the context the error occurred in
    pub context: Cow<'static, str>,
}

impl<E: Error> Display for Extended<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}
impl<E: Error> Error for Extended<E> {
    fn cause(&self) -> Option<&dyn Error> {
        Some(&self.inner)
    }
}
impl<E: Error> Deref for Extended<E> {
    type Target = E;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// Syntax extension which allows adding a context string to any error
pub trait ErrorExt: Error + Sized {
    fn context(self, msg: impl Into<Cow<'static, str>>) -> Extended<Self>;
}
impl<E: Error + Sized> ErrorExt for E {
    fn context(self, msg: impl Into<Cow<'static, str>>) -> Extended<Self> {
        Extended {
            inner: self,
            context: msg.into(),
        }
    }
}

/// Syntax extension which allows adding a context string to a result's error
pub trait ResultExt {
    /// The `Result`'s `Ok` type
    type Ok;

    /// The `Result`'s `Err` type to be extended
    type Err: Error + Sized;

    fn context(self, msg: impl Into<Cow<'static, str>>) -> Result<Self::Ok, Extended<Self::Err>>;
}
impl<T, E: Error + Sized> ResultExt for Result<T, E> {
    type Ok = T;
    type Err = E;
    fn context(self, msg: impl Into<Cow<'static, str>>) -> Result<T, Extended<E>> {
        self.map_err(|error| Extended {
            inner: error,
            context: msg.into(),
        })
    }
}
