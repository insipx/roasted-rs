use std::fmt::{self, Display, Formatter};

use diesel::{
    AsExpression, FromSqlRow,
    backend::Backend,
    deserialize::{self, FromSql},
    serialize::{self, Output, ToSql},
    sql_types::Binary,
};

/// Diesel-compatible UUID V7 for SQLite
#[derive(Debug, Default, Clone, Copy, FromSqlRow, AsExpression, Hash, Eq, PartialEq)]
#[diesel(sql_type = Binary)]
pub struct UUID(pub uuid::Uuid);

impl UUID {
    /// Construct a v7 UUID from the current timestamp
    pub fn v7_now() -> Self {
        Self(uuid::Uuid::now_v7())
    }
}

impl From<UUID> for uuid::Uuid {
    fn from(s: UUID) -> Self {
        s.0
    }
}

impl From<uuid::Uuid> for UUID {
    fn from(s: uuid::Uuid) -> Self {
        UUID(s)
    }
}

impl Display for UUID {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<B: Backend> FromSql<Binary, B> for UUID
where
    [u8; 16]: FromSql<Binary, B>,
{
    fn from_sql(bytes: <B as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        let value = <[u8; 16]>::from_sql(bytes)?;
        uuid::Uuid::from_slice(&value).map(UUID).map_err(|e| e.into())
    }
}

impl<B: Backend> ToSql<Binary, B> for UUID
where
    [u8]: ToSql<Binary, B>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, B>) -> serialize::Result {
        <[u8; 16] as ToSql<Binary, B>>::to_sql(self.0.as_bytes(), out)
    }
}
