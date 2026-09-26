//! Shared types/merges into generated WebSocket types for the roasted-rs daemon server

use crate::ws::gaggimate::{Patch, Status, SystemPhase, SystemState};

mod gaggimate_state;
mod shot_set;
mod system_state;

pub use gaggimate_state::*;
pub use shot_set::*;
pub use system_state::*;

/// Merge two types
/// Implement on the canonical version of the type for a
/// instant in time. `Self` should be the post-merge type.
pub trait Merge<T> {
    /// Conduct the merge, mutating `self`
    fn merge(&mut self, other: T);
}

/// Same as [`Merge`] but merges two different types.
/// Cannot put on same trait because of the lack of specialization.
pub trait MergeOther<T> {
    /// Merge a value of another type into this state.
    fn merge_other(&mut self, other: T);
}

impl<T, U> Merge<Box<T>> for U
where
    U: Merge<T>,
{
    fn merge(&mut self, other: Box<T>) {
        self.merge(*other)
    }
}

impl<T, U> MergeOther<Patch<U>> for T
where
    T: Default + Merge<U>,
{
    fn merge_other(&mut self, patch: Patch<U>) {
        match patch {
            Patch::Null => *self = Self::default(),
            Patch::Value(value) => self.merge(value),
            Patch::Absent => {}
        }
    }
}

impl<T: Default> Merge<Patch<T>> for T {
    fn merge(&mut self, other: Patch<T>) {
        match other {
            Patch::Null => *self = Default::default(),
            Patch::Value(t) => *self = t,
            Patch::Absent => {}
        }
    }
}

impl Merge<Status> for GaggimateState {
    fn merge(&mut self, other: Status) {
        self.process.merge(other.process);
        self.current_temperature.merge(other.current_temperature);
        self.target_temperature.merge(other.target_temperature);
        self.current_pressure.merge(other.current_pressure);
        self.current_flow.merge(other.current_flow);
        self.current_weight.merge(other.current_weight);
        self.bluetooth_weight.merge(other.bluetooth_weight);
        self.scale_connected.merge(other.scale_connected);
        self.system_state.merge_other(other.system_state);
        self.warnings.merge(other.warnings);
        self.target_pressure.merge(other.target_pressure);
        self.machine_mode.merge(other.machine_mode);
        self.profile_label.merge(other.profile_label);
        self.pressure_capable.merge(other.pressure_capable);
        self.dimming_capable.merge(other.dimming_capable);
    }
}

impl Merge<SystemState> for GaggimateSystemState {
    fn merge(&mut self, other: SystemState) {
        self.phase.merge(other.phase);
        self.message.merge(other.message.map(|s| (!s.is_empty()).then_some(s)));
        self.error_code.merge(other.error_code.map(|e| (e != 0).then_some(e)));
    }
}
