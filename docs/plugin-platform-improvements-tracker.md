# Plugin Platform Improvements Tracker

Status legend:
- `planned`: scoped but not started
- `in_progress`: actively being implemented
- `done`: implemented locally and validated
- `blocked`: cannot continue without upstream design decision

## Goal

Implement generic plugin-platform improvements (not Jelly J specific), in small additive slices that are merge-friendly upstream.

## Workstreams

| ID | Workstream | Scope | Status | Notes |
|---|---|---|---|---|
| W1 | Canonical plugin identity | Normalize/alias plugin identity across path vs `file:` URL forms for permission/cache checks | done | Aliasing + server callsites + tests validated |
| W2 | Permission introspection API | Host/shim API to query currently granted permissions | done | Added `GetGrantedPluginPermissions` command end-to-end |
| W3 | Workspace bootstrap snapshot API | One-shot state snapshot on plugin startup | done | Added `RequestPluginStateSnapshot` host/shim command path |
| W4 | Structured pipe envelope v2 | Optional metadata (`request_id`, delivery semantics hints) | done | Backward-compatible optional fields in pipe message proto/data conversions |
| W5 | Atomic pane launch API | Single host command for open+run+name | done | Added `LaunchTerminalPane` with atomic spawn response and optional initial input |
| W6 | Host-managed plugin logs | CLI/API for plugin logs | done | Added per-plugin in-memory ring-buffer access via host commands |
| W7 | Plugin test harness mode | Deterministic CI/dev support | done | Added `cargo xtask pluginharness` workflow |

## Execution Log

### 2026-02-10

- Created tracker and selected first implementation slice: `W1`.
- `W1` approach:
  - Add identity aliasing in permission cache so the same plugin is recognized via path and URL forms.
  - Migrate server-side permission cache read/write call sites to a canonical display form where possible.
  - Add tests for alias behavior.
- Finished `W1` validation:
  - `cargo test -p zellij-utils permission_cache_matches -- --nocapture`
  - `cargo check -p zellij-server`
- Implemented `W2`:
  - Added a new plugin command: `GetGrantedPluginPermissions`.
  - Added host handling in `zellij-server` to resolve and return cached granted permissions for the requesting plugin identity.
  - Added a `zellij-tile` shim API: `get_granted_plugin_permissions() -> Vec<PermissionType>`.
  - Added plugin command serialization/deserialization regression tests.
- `W2` validation:
  - `cargo test -p zellij-utils get_granted_plugin_permissions_command -- --nocapture`
  - `cargo check -p zellij-server -p zellij-tile`
- Implemented `W3`:
  - Added plugin command: `RequestPluginStateSnapshot`.
  - Wired command conversion and shim helper (`request_plugin_state_snapshot()`).
  - Implemented host dispatch to request state refresh for all plugins and for the calling plugin.
- Implemented `W4`:
  - Extended `PipeMessage` with optional envelope fields:
    - `request_id: Option<String>`
    - `delivery_hint: Option<PipeDeliveryHint>`
  - Added proto enum `DeliveryHint` and conversion coverage.
  - Added roundtrip regression test for envelope v2 compatibility.
- Implemented `W5`:
  - Added command: `LaunchTerminalPane` with payload for pane naming, cwd, command/shell, and optional initial stdin.
  - Added host response path returning `pane_id` on success or a structured error message.
  - Added permission gate requiring `OpenTerminalsOrPlugins` and, when writing initial input, `WriteToStdin`.
  - Exposed shim helper: `launch_terminal_pane(...) -> Result<PaneId, String>`.
- Implemented `W6`:
  - Added commands: `GetPluginLogs` and `ClearPluginLogs`.
  - Added host-managed per-plugin log registry in `logging_pipe.rs` (bounded in-memory ring buffer).
  - Added unload cleanup hook in `wasm_bridge.rs` to clear stale logs.
  - Exposed shim helpers:
    - `get_plugin_logs(max_lines: Option<u32>) -> Vec<String>`
    - `clear_plugin_logs() -> bool`
- Implemented `W7`:
  - Added deterministic harness command: `cargo xtask pluginharness`.
  - Harness runs targeted plugin-platform tests plus server/tile compile checks.
- Hardened `W5` launch semantics:
  - Switched `LaunchTerminalPane` host implementation from direct `PtyInstruction::SpawnTerminal` to routed `Action` execution (`route_action`).
  - This aligns behavior with existing stable plugin pane-launch code paths (`OpenTerminal*`, `OpenCommandPane*`) and avoids completion-only false positives where pane IDs were reported without durable pane presence.
  - Preserved existing response contract and optional `initial_input` write behavior.
- `W3`-`W7` validation:
  - `cargo xtask pluginharness`
  - Verified successful execution of:
    - `permission_cache_matches`
    - `get_granted_plugin_permissions_command`
    - `request_plugin_state_snapshot_command`
    - `get_plugin_logs_command`
    - `pipe_message_envelope_v2_roundtrip`
    - `cargo check -p zellij-server -p zellij-tile`

## Current Focus

### W3 checklist

- [x] Define additive command shape
- [x] Add plugin API command + payload/response to proto
- [x] Wire command in `zellij-utils` command conversion layer
- [x] Implement host-side responder in `zellij-server`
- [x] Expose shim helper in `zellij-tile`
- [x] Add conversion regression tests
- [x] Run targeted test/compile validation
- [x] Mark `W3` done

### W4 checklist

- [x] Define envelope extension shape
- [x] Add pipe message proto fields + enum
- [x] Update `zellij-utils` data model + conversion logic
- [x] Add backward-compatible roundtrip regression test
- [x] Run targeted test/compile validation
- [x] Mark `W4` done

### W5 checklist

- [x] Define atomic launch payload/response shape
- [x] Add plugin command proto and conversion support
- [x] Implement host-side spawn + optional stdin write + response
- [x] Add permission checks for launch/write semantics
- [x] Expose shim helper in `zellij-tile`
- [x] Add command conversion regression tests
- [x] Run targeted test/compile validation
- [x] Mark `W5` done

### W6 checklist

- [x] Define plugin log retrieval/clear command shape
- [x] Add plugin command proto and conversion support
- [x] Implement host-side log store query/clear paths
- [x] Integrate logging pipe writes into per-plugin buffer
- [x] Clear plugin log buffers on unload
- [x] Expose shim helpers in `zellij-tile`
- [x] Add command conversion regression tests
- [x] Run targeted test/compile validation
- [x] Mark `W6` done

### W7 checklist

- [x] Define deterministic harness command shape
- [x] Add `xtask` command wiring and flags
- [x] Add deterministic env setup and ordered checks
- [x] Cover key W1-W4 regression tests in harness
- [x] Include server/tile compile check in harness
- [x] Run harness and confirm pass
- [x] Mark `W7` done
