//! Trait for extracting shot features from Gaggimate telemetry data

pub trait Feature {
    fn compute(&self, traces: &[Status]);
}
