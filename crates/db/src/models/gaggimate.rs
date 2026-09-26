use roasted_types::daemon::GaggimateState;
use uuid::Uuid;

pub struct NewStatusFrame<'a> {
    shot_id: Uuid,
    #[diesel(embed)]
    state: &'a GaggimateState,
}
