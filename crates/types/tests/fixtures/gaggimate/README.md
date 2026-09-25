# GaggiMate telemetry fixtures

These fixed JSON inputs test decoding without a machine or network connection.
Debug snapshots preserve descriptive Rust field names, enum variants, and the
distinction between `Patch::Absent`, `Patch::Null`, and `Patch::Value`.

## Sources

- `live-idle.json`: the first raw text message captured from
  `ws://espresso.jupiter.lan/ws` on 2026-09-25. Kept verbatim, including unknown
  fields. Capture was read-only; the machine was in standby and the scale was
  disconnected. No application commands were sent.
- `active-infusion.json`, `active-extraction.json`, `finished-shot.json`:
  reconstructed JSON subsets from the earlier `ws-output-2` Rust debug capture
  of the Adaptive v2 shot. These are **not raw wire captures**. No scale readings
  were invented for these fixtures; that older capture discarded them.
- Remaining fixtures are constructed protocol cases: activity-only processes,
  cup removal, disconnected scale, partial updates, explicit nulls, and an
  unrelated OTA response. They do not assert that these exact payloads were
  observed on the device.

## Running and reviewing

From the workspace root:

```sh
cargo test -p roasted-types
cargo insta review
```

`cargo-insta` is optional review tooling (`cargo install cargo-insta --locked`).
Ordinary `cargo test` runs the snapshots. Changed expectations produce
`.snap.new` files; inspect their diff before accepting them. CI can explicitly
disable snapshot writes with:

```sh
INSTA_UPDATE=no cargo test -p roasted-types --locked
```

To refresh a live fixture, capture raw WebSocket text while the machine is in
the desired state, save one complete JSON message, and review the decoded
snapshot change. Never recapture data automatically during tests.

The existing assertion tests still cover numeric wire mappings, invalid inputs,
partial-update semantics, and JSON round trips.
