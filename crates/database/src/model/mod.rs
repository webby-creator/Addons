use std::ops::{Deref, DerefMut};

use serde::{de::DeserializeOwned, Serialize};
use sqlx::{
    encode::IsNull, error::BoxDynError, sqlite::SqliteTypeInfo, Decode, Encode, Sqlite, Type,
};

mod addon;
mod media_upload;
mod schema;
mod schema_data;
mod schema_data_tag;
mod settings;

pub use addon::*;
pub use media_upload::*;
pub use schema::*;
pub use schema_data::*;
pub use schema_data_tag::*;
pub use settings::*;

#[derive(Debug, Clone)]
pub struct Binary<T: ?Sized>(pub T);

impl<T> Deref for Binary<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Binary<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> AsRef<T> for Binary<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> AsMut<T> for Binary<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> Type<Sqlite> for Binary<T> {
    fn type_info() -> SqliteTypeInfo {
        <Vec<u8> as Type<Sqlite>>::type_info()
    }

    fn compatible(ty: &SqliteTypeInfo) -> bool {
        <Vec<u8> as Type<Sqlite>>::compatible(ty)
    }
}

impl<T> Encode<'_, Sqlite> for Binary<T>
where
    T: Serialize,
{
    fn encode_by_ref(
        &self,
        buf: &mut <Sqlite as sqlx::database::HasArguments<'_>>::ArgumentBuffer,
    ) -> IsNull {
        Encode::<Sqlite>::encode(serde_json::to_vec(&self.0).unwrap(), buf)
    }
}

impl<'de, T> Decode<'de, Sqlite> for Binary<T>
where
    T: DeserializeOwned,
{
    fn decode(
        value: <Sqlite as sqlx::database::HasValueRef<'de>>::ValueRef,
    ) -> Result<Self, BoxDynError> {
        let dec = <Vec<u8> as Decode<Sqlite>>::decode(value)?;
        let from = serde_json::from_slice(&dec)?;

        Ok(Self(from))
    }
}
