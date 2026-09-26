use diesel::{
    Insertable,
    deserialize::{self, FromSql},
    serialize::{self, IsNull, Output, ToSql},
    sql_types::{BigInt, Integer, Text},
    sqlite::{Sqlite, SqliteValue},
};

use crate::{
    daemon::GaggimateState,
    ws::gaggimate::{
        ElapsedMs, MachineMode, ProcessActivity, ProcessPhase, ProcessTarget, SystemPhase,
        UtilityFlag,
    },
};

// Keep the database spelling independent of the firmware's Serde representation.
macro_rules! sqlite_text_enum {
    ($ty:ident { $($variant:ident => $text:literal),+ $(,)? }) => {
        impl ToSql<Text, Sqlite> for $ty {
            fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
                out.set_value(match self {
                    $(Self::$variant => $text),+
                });
                Ok(IsNull::No)
            }
        }

        impl FromSql<Text, Sqlite> for $ty {
            fn from_sql(value: SqliteValue<'_, '_, '_>) -> deserialize::Result<Self> {
                let value = <String as FromSql<Text, Sqlite>>::from_sql(value)?;
                match value.as_str() {
                    $($text => Ok(Self::$variant)),+,
                    _ => Err(format!("Unknown {} value: {value}", stringify!($ty)).into()),
                }
            }
        }
    };
}

sqlite_text_enum!(MachineMode {
    Standby => "standby",
    Brew => "brew",
    Steam => "steam",
    HotWater => "hot_water",
    Grind => "grind",
});

sqlite_text_enum!(ProcessActivity {
    Inactive => "inactive",
    Active => "active",
});

sqlite_text_enum!(ProcessPhase {
    Infusion => "infusion",
    Brew => "brew",
    Grind => "grind",
});

sqlite_text_enum!(ProcessTarget {
    Time => "time",
    Volumetric => "volumetric",
});

sqlite_text_enum!(SystemPhase {
    Starting => "starting",
    Waiting => "waiting",
    Ready => "ready",
    Updating => "updating",
    Autotuning => "autotuning",
    Mismatch => "mismatch",
    Error => "error",
});

impl ToSql<BigInt, Sqlite> for ElapsedMs {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        out.set_value(i64::try_from(self.0)?);
        Ok(IsNull::No)
    }
}

impl FromSql<BigInt, Sqlite> for ElapsedMs {
    fn from_sql(value: SqliteValue<'_, '_, '_>) -> deserialize::Result<Self> {
        let value = <i64 as FromSql<BigInt, Sqlite>>::from_sql(value)?;
        Ok(Self(u64::try_from(value)?))
    }
}

impl ToSql<Integer, Sqlite> for UtilityFlag {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        out.set_value(i32::from(self.0));
        Ok(IsNull::No)
    }
}

impl FromSql<Integer, Sqlite> for UtilityFlag {
    fn from_sql(value: SqliteValue<'_, '_, '_>) -> deserialize::Result<Self> {
        // Read the full SQLite integer before checking the u8 range.
        let value = <i64 as FromSql<BigInt, Sqlite>>::from_sql(value)?;
        Ok(Self(u8::try_from(value)?))
    }
}

impl<'a, T> Insertable<T> for &'_ &'a GaggimateState
where
    &'a GaggimateState: Insertable<T>,
{
    type Values = <&'a GaggimateState as Insertable<T>>::Values;

    fn values(self) -> Self::Values {
        <&'a GaggimateState as Insertable<T>>::values(*self)
    }
}
