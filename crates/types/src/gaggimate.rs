//! GaggiMate's JSON messages on `/ws`.
//!
//! Mirrors upstream revision `001a475105cdfa11c54e2c688046f8159dda8d3e`:
//! <https://github.com/jniebuhr/gaggimate/blob/001a475105cdfa11c54e2c688046f8159dda8d3e/docs/websocket-api.yaml>.
//! Profiles follow `schema/profile.json` at that revision.
//!
//! Decode with `serde_json::from_str::<Message>()`. Status frames are patches:
//! omitted fields retain prior state, explicit null clears it. These types
//! describe the JSON shape; numerical operating limits and profile constraints
//! such as nonempty phases require separate validation. No network I/O or
//! machine control is performed here.
//!
//! ```
//! use roasted_types::gaggimate::{Message, Patch};
//! let message: Message = serde_json::from_str(r#"{"tp":"evt:status","pr":8.5}"#)?;
//! if let Message::Status(status) = message {
//!     assert_eq!(status.pr, Patch::Value(8.5));
//!     assert_eq!(status.ct, Patch::Absent);
//! }
//! # Ok::<(), serde_json::Error>(())
//! ```

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::collections::BTreeMap;

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

/// A request, response, or event, identified by the JSON `tp` field.
///
/// Requests are modeled for API completeness; roasted's ingest remains read-only.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "tp")]
pub enum Message {
    /// `evt:status`.
    #[serde(rename = "evt:status")]
    Status(Box<Status>),
    /// `res:ota-settings`.
    #[serde(rename = "res:ota-settings")]
    OtaSettingsResponse(OtaSettingsResponse),
    /// `evt:ota-progress`.
    #[serde(rename = "evt:ota-progress")]
    OtaProgress(OtaProgress),
    /// `evt:autotune-result`.
    #[serde(rename = "evt:autotune-result")]
    AutotuneResult(AutotuneResult),
    /// `res:profiles:list`.
    #[serde(rename = "res:profiles:list")]
    ProfilesListResponse(ProfilesListResponse),
    /// `res:profiles:load`.
    #[serde(rename = "res:profiles:load")]
    ProfilesLoadResponse(Box<ProfileResponse>),
    /// `res:profiles:save`.
    #[serde(rename = "res:profiles:save")]
    ProfilesSaveResponse(Box<ProfileResponse>),
    /// `res:profiles:delete`.
    #[serde(rename = "res:profiles:delete")]
    ProfilesDeleteResponse(ResultResponse),
    /// `res:profiles:select`.
    #[serde(rename = "res:profiles:select")]
    ProfilesSelectResponse(ResultResponse),
    /// `res:profiles:favorite`.
    #[serde(rename = "res:profiles:favorite")]
    ProfilesFavoriteResponse(ResultResponse),
    /// `res:profiles:unfavorite`.
    #[serde(rename = "res:profiles:unfavorite")]
    ProfilesUnfavoriteResponse(ResultResponse),
    /// `res:profiles:reorder`.
    #[serde(rename = "res:profiles:reorder")]
    ProfilesReorderResponse(CorrelatedMessage),
    /// `evt:brew:confirm`.
    #[serde(rename = "evt:brew:confirm")]
    BrewConfirm(BrewConfirm),
    /// `evt:brew:confirm:cancel`.
    #[serde(rename = "evt:brew:confirm:cancel")]
    BrewConfirmCancel,
    /// `res:flush:start`.
    #[serde(rename = "res:flush:start")]
    FlushStartResponse(FlushResponse),
    /// `res:flush:stop`.
    #[serde(rename = "res:flush:stop")]
    FlushStopResponse(FlushResponse),
    /// `req:ota-settings`.
    #[serde(rename = "req:ota-settings")]
    OtaSettingsRequest(OtaSettingsRequest),
    /// `req:ota-start`.
    #[serde(rename = "req:ota-start")]
    OtaStartRequest(OtaStartRequest),
    /// `req:autotune-start`.
    #[serde(rename = "req:autotune-start")]
    AutotuneStartRequest(AutotuneStartRequest),
    /// `req:profiles:list`.
    #[serde(rename = "req:profiles:list")]
    ProfilesListRequest(CorrelatedMessage),
    /// `req:profiles:load`.
    #[serde(rename = "req:profiles:load")]
    ProfilesLoadRequest(ProfileIdRequest),
    /// `req:profiles:save`.
    #[serde(rename = "req:profiles:save")]
    ProfilesSaveRequest(Box<ProfileSaveRequest>),
    /// `req:profiles:delete`.
    #[serde(rename = "req:profiles:delete")]
    ProfilesDeleteRequest(ProfileIdRequest),
    /// `req:profiles:select`.
    #[serde(rename = "req:profiles:select")]
    ProfilesSelectRequest(ProfileIdRequest),
    /// `req:profiles:favorite`.
    #[serde(rename = "req:profiles:favorite")]
    ProfilesFavoriteRequest(ProfileIdRequest),
    /// `req:profiles:unfavorite`.
    #[serde(rename = "req:profiles:unfavorite")]
    ProfilesUnfavoriteRequest(ProfileIdRequest),
    /// `req:profiles:reorder`.
    #[serde(rename = "req:profiles:reorder")]
    ProfilesReorderRequest(ProfilesReorderRequest),
    /// `req:process:activate`.
    #[serde(rename = "req:process:activate")]
    ProcessActivateRequest(ProcessActivateRequest),
    /// `req:brew:confirm:cancel`.
    #[serde(rename = "req:brew:confirm:cancel")]
    BrewConfirmCancelRequest,
    /// `req:flush:start`.
    #[serde(rename = "req:flush:start")]
    FlushStartRequest(CorrelatedMessage),
    /// `req:flush:stop`.
    #[serde(rename = "req:flush:stop")]
    FlushStopRequest(CorrelatedMessage),
    /// An unrecognized message type; its payload is ignored and cannot be serialized.
    #[serde(other, skip_serializing)]
    Unknown,
}

/// Partial telemetry/state frame. No scale weight or pump-active key is documented.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Status {
    /// Current temperature.
    #[serde(default, skip_serializing_if = "Patch::is_absent")]
    pub ct: Patch<f64>,
    /// Target temperature.
    #[serde(default, skip_serializing_if = "Patch::is_absent")]
    pub tt: Patch<f64>,
    /// Current pressure.
    #[serde(default, skip_serializing_if = "Patch::is_absent")]
    pub pr: Patch<f64>,
    /// Current flow.
    #[serde(default, skip_serializing_if = "Patch::is_absent")]
    pub fl: Patch<f64>,
    /// Display system state.
    #[serde(default, skip_serializing_if = "Patch::is_absent")]
    pub sys: Patch<SystemState>,
    /// Machine warnings; an empty list is a real update.
    #[serde(default, skip_serializing_if = "Patch::is_absent")]
    pub warn: Patch<Vec<WarningState>>,
    /// Target pressure.
    #[serde(default, skip_serializing_if = "Patch::is_absent")]
    pub pt: Patch<f64>,
    /// Machine mode; upstream does not enumerate its numeric values.
    #[serde(default, skip_serializing_if = "Patch::is_absent")]
    pub m: Patch<i64>,
    /// Selected profile label.
    #[serde(default, skip_serializing_if = "Patch::is_absent")]
    pub p: Patch<String>,
    /// Pressure capability.
    #[serde(default, skip_serializing_if = "Patch::is_absent")]
    pub cp: Patch<bool>,
    /// Dimming capability.
    #[serde(default, skip_serializing_if = "Patch::is_absent")]
    pub cd: Patch<bool>,
}

/// Display state carried within a status frame.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SystemState {
    /// System phase.
    #[serde(default, skip_serializing_if = "Patch::is_absent")]
    pub s: Patch<SystemPhase>,
    /// Display message; empty when ready.
    #[serde(default, skip_serializing_if = "Patch::is_absent")]
    pub m: Patch<String>,
    /// Controller error code; zero means none.
    #[serde(default, skip_serializing_if = "Patch::is_absent")]
    pub c: Patch<i64>,
}

/// Documented display system phases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SystemPhase {
    /// Starting.
    Starting,
    /// Waiting.
    Waiting,
    /// Ready.
    Ready,
    /// Updating.
    Updating,
    /// Autotuning.
    Autotuning,
    /// Mismatch.
    Mismatch,
    /// Error.
    Error,
}

/// Warning entry; in brew-confirm an omitted active flag means true.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WarningState {
    /// Warning category.
    pub k: WarningKey,
    /// Configured severity, encoded as an integer.
    pub l: WarningLevel,
    /// Whether the warning is active; context determines the omitted value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub a: Option<bool>,
}

/// Documented warning categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WarningKey {
    /// Water.
    Water,
    /// Flush.
    Flush,
    /// Switch.
    Switch,
    /// ScaleConnected.
    ScaleConnected,
    /// ScaleBattery.
    ScaleBattery,
    /// Temperature.
    Temperature,
}

/// Warning severity on the JSON wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum WarningLevel {
    /// 0: ignore.
    Ignore = 0,
    /// 1: warn.
    Warn = 1,
    /// 2: error.
    Error = 2,
}

/// Firmware update status; all payload fields are optional.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OtaSettingsResponse {
    /// latest version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_version: Option<String>,
    /// display version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_version: Option<String>,
    /// controller version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub controller_version: Option<String>,
    /// hardware.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hardware: Option<String>,
    /// Whether a display update is available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_update_available: Option<bool>,
    /// Whether a controller update is available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub controller_update_available: Option<bool>,
    /// Update channel.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    /// Whether an update is running.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updating: Option<bool>,
}

/// Progress within a firmware update phase.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OtaProgress {
    /// Current update phase, encoded as an integer.
    pub phase: OtaPhase,
    /// Percent within the phase; retries can move backward.
    pub progress: i64,
}

/// Firmware update phase on the JSON wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum OtaPhase {
    /// 0: idle.
    Idle = 0,
    /// 1: display firmware.
    DisplayFirmware = 1,
    /// 2: display filesystem (unused).
    DisplayFilesystem = 2,
    /// 3: controller firmware.
    ControllerFirmware = 3,
    /// 4: finished.
    Finished = 4,
    /// 5: failed.
    Failed = 5,
}

/// Autotune result returned as a string.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutotuneResult {
    /// PID result; upstream specifies a string.
    pub pid: String,
}

/// Firmware update settings request.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct OtaSettingsRequest {
    /// Whether to update settings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub update: Option<bool>,
    /// Update channel.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
}

/// Start-update payload.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct OtaStartRequest {
    /// Optional string; unrelated to status's boolean cp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cp: Option<String>,
}

/// Autotune parameters.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AutotuneStartRequest {
    /// Optional duration parameter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time: Option<i64>,
    /// Optional sample count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub samples: Option<i64>,
}

/// Payload containing only an optional correlation ID.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CorrelatedMessage {
    /// Optional request correlation ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rid: Option<String>,
}

/// Common profile-operation response.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ResultResponse {
    /// Optional request correlation ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rid: Option<String>,
    /// Optional error description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Profile listing or error.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ProfilesListResponse {
    /// Optional request correlation ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rid: Option<String>,
    /// Profiles returned by the firmware.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profiles: Option<Vec<Profile>>,
    /// Optional error description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Loaded or saved profile, or an error.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ProfileResponse {
    /// Optional request correlation ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rid: Option<String>,
    /// Returned profile.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<Profile>,
    /// Optional error description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Operation on a required profile identifier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProfileIdRequest {
    /// Optional request correlation ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rid: Option<String>,
    /// Firmware-managed profile ID.
    pub id: String,
}

/// Save a required profile payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProfileSaveRequest {
    /// Optional request correlation ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rid: Option<String>,
    /// Profile to save.
    pub profile: Profile,
}

/// Set the full profile ordering.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProfilesReorderRequest {
    /// Optional request correlation ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rid: Option<String>,
    /// Complete ordered list of profile IDs.
    pub order: Vec<String>,
}

/// Warnings that blocked a brew start.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrewConfirm {
    /// Active, non-ignored warnings; missing a means active.
    pub warn: Vec<WarningState>,
}

/// Start the current process.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ProcessActivateRequest {
    /// Omitted means false.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ignoreWarnings")]
    pub ignore_warnings: Option<bool>,
}

/// Flush operation acknowledgement.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct FlushResponse {
    /// Optional request correlation ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rid: Option<String>,
    /// Optional success flag; missing is not false.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub success: Option<bool>,
}

/// Firmware profile JSON; numerical limits must be validated separately.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    /// Firmware-managed identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Required nonempty display label.
    pub label: String,
    /// Profile category.
    #[serde(rename = "type")]
    pub kind: ProfileType,
    /// Required description; may be empty.
    pub description: String,
    /// Boiler setpoint in Celsius, 0..150.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    /// Omitted means false; firmware controls live value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub favorite: Option<bool>,
    /// Omitted means false; firmware controls live value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected: Option<bool>,
    /// Omitted means false; true for non-brew routines.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub utility: Option<bool>,
    /// Ordered phases; upstream requires at least one.
    pub phases: Vec<ProfilePhase>,
    /// Extra profile properties, including upstream's arbitrary `_*` annotations.
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// Profile duration behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProfileType {
    /// A volumetric target blocks the duration timeout.
    Standard,
    /// Duration remains a hard cap.
    Pro,
}

/// One phase of a brew or utility profile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProfilePhase {
    /// Required nonempty display name.
    pub name: String,
    /// Phase category.
    pub phase: PhaseType,
    /// Solenoid state, JSON integer 0 or 1.
    pub valve: ValveState,
    /// Seconds, 0.5..300.
    pub duration: f64,
    /// Integer duty cycle or advanced control object.
    pub pump: Pump,
    /// Omitted/zero inherits profile temperature; override up to 160 Celsius.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    /// Omitted means an instant, nonadaptive transition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transition: Option<Transition>,
    /// OR-combined stop conditions; omitted/empty uses duration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub targets: Option<Vec<Target>>,
}

/// Brew phase category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PhaseType {
    /// Low-pressure saturation.
    Preinfusion,
    /// Main extraction.
    Brew,
}

/// Solenoid state on the JSON wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum ValveState {
    /// 0: closed.
    Closed = 0,
    /// 1: open.
    Open = 1,
}

/// A simple integer pump duty cycle or a closed-loop control object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Pump {
    /// Duty cycle in percent; validate the upstream range 0..100 separately.
    Percent(u8),
    /// Pressure or flow setpoint and the other quantity's soft limit.
    Advanced(AdvancedPump),
}

/// Closed-loop pump settings; -1 holds the value measured at phase entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdvancedPump {
    /// Active setpoint; the other quantity is a soft limit.
    pub target: PumpTarget,
    /// Bar: -1 or 0..12; zero as a limit disables it.
    pub pressure: f64,
    /// Water g/s: -1 or 0..15; zero as a limit disables it.
    pub flow: f64,
}

/// Controlled pump variable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PumpTarget {
    /// Pressure setpoint.
    Pressure,
    /// Flow setpoint.
    Flow,
}

/// Pump setpoint transition; all fields required when present.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transition {
    /// Easing function.
    #[serde(rename = "type")]
    pub kind: TransitionType,
    /// Nonnegative ramp duration in seconds.
    pub duration: f64,
    /// Start at the measured value rather than previous setpoint.
    pub adaptive: bool,
}

/// Pump easing function.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TransitionType {
    /// Instant.
    Instant,
    /// Linear.
    Linear,
    /// EaseIn.
    EaseIn,
    /// EaseOut.
    EaseOut,
    /// EaseInOut.
    EaseInOut,
}

/// A phase stop condition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Target {
    /// Measured quantity.
    #[serde(rename = "type")]
    pub kind: TargetType,
    /// Nonnegative threshold; volumetric zero disables that target.
    pub value: f64,
    /// Omitted means greater than or equal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operator: Option<TargetOperator>,
}

/// Supported stop-condition quantities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TargetType {
    /// Scale weight in grams; JSON weight is not a valid alias.
    Volumetric,
    /// Pressure in bar.
    Pressure,
    /// Flow in ml/s.
    Flow,
    /// Cumulative water pumped in the phase, in ml.
    Pumped,
}

/// Comparison used by a stop condition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TargetOperator {
    /// Greater than or equal.
    Gte,
    /// Less than or equal.
    Lte,
}
