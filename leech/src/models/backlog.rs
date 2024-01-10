use std::borrow::Cow;

use kraken_proto::AnyAttackResponse;
use prost::Message;
use rorm::conditions::Value;
use rorm::fields::traits::FieldType;
use rorm::internal::field::as_db_type::AsDbType;
use rorm::internal::field::modifier::{MergeAnnotations, SingleColumnCheck, SingleColumnFromName};
use rorm::internal::field::Field;
use rorm::internal::hmr::annotations::Annotations;
use rorm::internal::hmr::db_type::{Binary, DbType};
use rorm::internal::hmr::AsImr;
use rorm::{imr, new_converting_decoder, Model};
use uuid::Uuid;

/// A response and its associated attack which couldn't be sent
#[derive(Model)]
pub struct BacklogEntry {
    /// The primary key
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The attack this entry contains a response for
    pub attack_uuid: Uuid,

    /// The response which couldn't be sent
    pub response: Proto<AnyAttackResponse>,
}

/// Stores data by serializing it using protobuf.
pub struct Proto<T: Message + Default>(
    /// The wrapped proto message
    pub T,
);

impl<T: Message + Default + 'static> FieldType for Proto<T> {
    type Columns<C> = [C; 1];

    fn into_values(self) -> Self::Columns<Value<'static>> {
        [Value::Binary(Cow::Owned(self.0.encode_to_vec()))]
    }

    fn as_values(&self) -> Self::Columns<Value<'_>> {
        [Value::Binary(Cow::Owned(self.0.encode_to_vec()))]
    }

    fn get_imr<F: Field<Type = Self>>() -> Self::Columns<imr::Field> {
        [imr::Field {
            name: F::NAME.to_string(),
            db_type: Binary::IMR,
            annotations: F::EFFECTIVE_ANNOTATIONS
                .unwrap_or_else(Annotations::empty)
                .as_imr(),
            source_defined_at: None,
        }]
    }

    type Decoder = ProtoDecoder<T>;
    type AnnotationsModifier<F: Field<Type = Self>> = MergeAnnotations<Self>;
    type CheckModifier<F: Field<Type = Self>> = SingleColumnCheck<Binary>;
    type ColumnsFromName<F: Field<Type = Self>> = SingleColumnFromName;
}

impl<T: Message + Default + 'static> AsDbType for Proto<T> {
    type Primitive = Vec<u8>;
    type DbType = Binary;

    fn from_primitive(primitive: Self::Primitive) -> Self {
        // This already deprecated rorm API doesn't allow propagating errors
        #[allow(clippy::unwrap_used)]
        Self(T::decode(primitive.as_slice()).unwrap())
    }
}

new_converting_decoder!(
    #[doc(hidden)]
    pub ProtoDecoder<T: Message + Default>,
    |value: Vec<u8>| -> Proto<T> {
        T::decode(value.as_slice())
            .map(Proto)
            .map_err(|err| rorm::Error::DecodeError(format!("Couldn't decoder protobuf: {err}")))
    }
);
