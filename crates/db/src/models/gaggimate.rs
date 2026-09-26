use diesel::prelude::*;
use roasted_types::{daemon::GaggimateState, db::UUID};

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = roasted_db_schema::schema::gaggimate_status_frames)]
pub struct NewStatusFrame<'a> {
    shot_id: UUID,
    #[diesel(embed)]
    state: &'a GaggimateState,
}
