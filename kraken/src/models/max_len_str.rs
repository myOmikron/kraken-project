use core::fmt;
use std::borrow::Cow;
use std::ops::Deref;

use rorm::conditions::Value;
use rorm::fields::traits::FieldType;
use rorm::internal::field::Field;
use rorm::internal::hmr::AsImr;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct MaxLenStr<const MAX: usize, Str = String, Len = Bytes> {
    string: Str,
    len_impl: Len,
}

impl<const MAX_LEN: usize, Str, Len> Deref for MaxLenStr<MAX_LEN, Str, Len>
where
    Str: Deref<Target = str>,
{
    type Target = Str;

    fn deref(&self) -> &Self::Target {
        &self.string
    }
}

impl<const MAX: usize, Str, Len> fmt::Display for MaxLenStr<MAX, Str, Len>
where
    Str: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.string.fmt(f)
    }
}

/* ======= CONSTRUCTOR BEGIN =======*/
impl<const MAX: usize, Str, Len> MaxLenStr<MAX, Str, Len>
where
    Str: Deref<Target = str>,
    Len: LenImpl,
{
    pub fn new(string: Str) -> Result<Self, MaxLenError>
    where
        Len: Default,
    {
        Self::with_impl(string, Default::default())
    }

    pub fn with_impl(string: Str, len_impl: Len) -> Result<Self, MaxLenError> {
        let len = len_impl.len(&string);
        if len > MAX {
            Err(MaxLenError { max: MAX, got: len })
        } else {
            Ok(Self { string, len_impl })
        }
    }
}
/* ======= CONSTRUCTOR END =======*/

/* ======= SERDE BEGIN =======*/
impl<const MAX: usize, Str, Len> Serialize for MaxLenStr<MAX, Str, Len>
where
    Str: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.string.serialize(serializer)
    }
}

impl<'de, const MAX: usize, Str, Len> Deserialize<'de> for MaxLenStr<MAX, Str, Len>
where
    Str: Deserialize<'de> + Deref<Target = str>,
    Len: Default + LenImpl,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        Str::deserialize(deserializer).and_then(|string| {
            Self::new(string).map_err(|err| {
                Error::custom(format_args!(
                    "String is too long: {} (max is {})",
                    err.got, err.max
                ))
            })
        })
    }
}
/* ======= SERDE END =======*/

/* ======= RORM END =======*/
impl<const MAX: usize, Len> FieldType for MaxLenStr<MAX, String, Len>
where
    Len: 'static,
{
    type Columns<T> = [T; 1];

    fn into_values(self) -> Self::Columns<Value<'static>> {
        [Value::String(Cow::Owned(self.string))]
    }

    fn as_values(&self) -> Self::Columns<Value<'_>> {
        [Value::String(Cow::Borrowed(&self.string))]
    }

    fn get_imr<F: Field<Type = Self>>() -> Self::Columns<rorm::imr::Field> {
        use rorm::imr::*;
        [Field {
            name: F::NAME.to_string(),
            db_type: DbType::VarChar,
            annotations: F::EFFECTIVE_ANNOTATIONS.unwrap_or_default().as_imr(),
            source_defined_at: None,
        }]
    }

    type Decoder = ();
    type AnnotationsModifier<F: Field<Type = Self>> = ();
    type CheckModifier<F: Field<Type = Self>> = ();
    type ColumnsFromName<F: Field<Type = Self>> = ();
}
/* ======= RORM END =======*/

pub struct MaxLenError {
    pub max: usize,
    pub got: usize,
}

pub trait LenImpl {
    fn len(&self, string: &str) -> usize;
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct Bytes;
impl LenImpl for Bytes {
    fn len(&self, string: &str) -> usize {
        string.as_bytes().len()
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct CodePoints;
impl LenImpl for CodePoints {
    fn len(&self, string: &str) -> usize {
        string.chars().count()
    }
}
