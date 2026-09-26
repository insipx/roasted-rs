#![cfg(feature = "diesel")]

use diesel::{IntoSql, connection::SimpleConnection, prelude::*, sql_types::*};
use roasted_db_schema::schema::gaggimate_status_frames as frames;
use roasted_types::{daemon::GaggimateState, ws::gaggimate::*};

#[test]
fn unsigned_sql_conversions_check_boundaries_and_preserve_null() {
    let conn = &mut SqliteConnection::establish(":memory:").unwrap();
    for value in [0, 1500, i64::MAX as u64] {
        assert_eq!(
            diesel::select(ElapsedMs(value).into_sql::<BigInt>())
                .get_result::<ElapsedMs>(conn)
                .unwrap(),
            ElapsedMs(value),
        );
    }
    for value in [i64::MAX as u64 + 1, u64::MAX] {
        assert!(
            diesel::select(ElapsedMs(value).into_sql::<BigInt>()).get_result::<i64>(conn).is_err()
        );
    }
    assert!(diesel::select((-1_i64).into_sql::<BigInt>()).get_result::<ElapsedMs>(conn).is_err());
    for value in [0, 1, u8::MAX] {
        assert_eq!(
            diesel::select(UtilityFlag(value).into_sql::<Integer>())
                .get_result::<UtilityFlag>(conn)
                .unwrap(),
            UtilityFlag(value),
        );
    }
    // Include values that would silently wrap if read as i32 first.
    for value in ["-1", "256", "4294967296", "9223372036854775807"] {
        assert!(
            diesel::select(diesel::dsl::sql::<Integer>(value))
                .get_result::<UtilityFlag>(conn)
                .is_err()
        );
    }
    assert_eq!(
        diesel::select(None::<ElapsedMs>.into_sql::<Nullable<BigInt>>())
            .get_result::<Option<ElapsedMs>>(conn)
            .unwrap(),
        None
    );
    assert_eq!(
        diesel::select(None::<UtilityFlag>.into_sql::<Nullable<Integer>>())
            .get_result::<Option<UtilityFlag>>(conn)
            .unwrap(),
        None
    );
}
