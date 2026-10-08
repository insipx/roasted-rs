use core::ops;

use serde::{Deserialize, Serialize};

/// Elapsed process time in milliseconds.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
#[cfg_attr(feature = "db", derive(diesel::AsExpression, diesel::FromSqlRow))]
#[cfg_attr(feature = "db", diesel(sql_type = diesel::sql_types::BigInt))]
pub struct ElapsedMs(pub u64);

impl ops::Deref for ElapsedMs {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<u64> for ElapsedMs {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<ElapsedMs> for u64 {
    fn from(value: ElapsedMs) -> Self {
        value.0
    }
}
