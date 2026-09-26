use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A field in a partial status update.
///
/// Use `#[serde(default, skip_serializing_if = "Patch::is_absent")]` on fields.
/// Ordinary `Option<T>` would collapse missing and null into the same value.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Patch<T> {
    /// No update; retain the previous value.
    #[default]
    Absent,
    /// Explicit JSON null; clear the previous value.
    Null,
    /// Replace the previous value, including zero, false, or an empty collection.
    Value(T),
}

impl<T> Patch<T> {
    /// Whether this field was omitted from the update.
    pub fn is_absent(&self) -> bool {
        matches!(self, Self::Absent)
    }

    /// Map `Patch` from `T` to `U`
    pub fn map<F, U>(self, f: F) -> Patch<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Patch::Absent => Patch::Absent,
            Patch::Null => Patch::Null,
            Patch::Value(t) => Patch::Value(f(t)),
        }
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Patch<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Option::<T>::deserialize(deserializer).map(|value| value.map_or(Self::Null, Self::Value))
    }
}

impl<T: Serialize> Serialize for Patch<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Absent => Err(serde::ser::Error::custom(
                "absent patch fields must be omitted by the containing struct",
            )),
            Self::Null => serializer.serialize_none(),
            Self::Value(value) => value.serialize(serializer),
        }
    }
}
