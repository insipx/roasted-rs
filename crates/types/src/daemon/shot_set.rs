use crate::daemon::GaggimateState;

/// The accumulated state of a single shot pull
#[derive(Debug, Clone)]
pub struct ShotSet {
    inner: Vec<GaggimateState>,
}

impl ShotSet {
    /// Create a new ShotSet from a collection of `GaggimateState`
    pub fn new(frames: Vec<GaggimateState>) -> Self {
        Self { inner: frames }
    }

    /// get the inner set
    pub fn iter(&self) -> impl Iterator<Item = &GaggimateState> {
        self.inner.iter()
    }
}
