use bon::Builder;
use diesel::prelude::*;
use roasted_types::{daemon::GaggimateState, db::UUID};

#[derive(Debug, Clone, Insertable, Builder)]
#[diesel(table_name = roasted_db_schema::schema::gaggimate_status_frames)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewStatusFrame<'a> {
    shot_id: &'a UUID,
    #[diesel(embed)]
    state: &'a GaggimateState,
}
