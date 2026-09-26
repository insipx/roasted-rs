use serde::{Deserialize, Serialize};

/// Numeric utility profile flag reported by the firmware.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
#[cfg_attr(feature = "diesel", derive(diesel::AsExpression, diesel::FromSqlRow))]
#[cfg_attr(feature = "diesel", diesel(sql_type = diesel::sql_types::Integer))]
pub struct UtilityFlag(pub u8);

impl std::ops::Deref for UtilityFlag {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<u8> for UtilityFlag {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<UtilityFlag> for u8 {
    fn from(value: UtilityFlag) -> Self {
        value.0
    }
}
