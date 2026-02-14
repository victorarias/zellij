# Plugin Platform Upstream Notes (Jelly J Driven, Generic Scope)

Status date: 2026-02-11
Base: `origin/main` commit `97744ad0`
Local commits:
- `81dc6834` feat(plugin-platform): add plugin API improvements and harness
- `304046a1` fix(plugin-api): route LaunchTerminalPane through action engine
- `4aee2044` fix(plugin-routing): scope specific pipes to source client

## Why this document exists

This captures what changed in the local Zellij fork, why each change is generic (not Jelly J-only), how Jelly J consumes it, and what to propose upstream.

## Change Inventory

1. Canonical plugin identity + permission aliasing (`81dc6834`)
- Generic value:
  - Prevents permission/cache mismatches between equivalent plugin URLs/paths (`file:`, absolute path variants).
  - Benefits every plugin loaded through multiple path spellings.
- Jelly J usage:
  - Reduces permission nondeterminism in e2e/harness and local installs.
- Upstream risk:
  - Low; additive and correctness-oriented.

2. Permission introspection API (`GetGrantedPluginPermissions`) (`81dc6834`)
- Generic value:
  - Lets any plugin inspect effective permissions at runtime.
- Jelly J usage:
  - Enables robust readiness/diagnostics paths.
- Upstream risk:
  - Low; additive command surface.

3. Plugin state snapshot API (`RequestPluginStateSnapshot`) (`81dc6834`)
- Generic value:
  - Gives deterministic initial state bootstrap to plugins without waiting for organic updates.
- Jelly J usage:
  - Butler startup and readiness stabilization.
- Upstream risk:
  - Low/medium; additive command with server dispatch path.

4. Pipe envelope v2 metadata (`request_id`, `delivery_hint`) (`81dc6834`)
- Generic value:
  - Adds structured metadata for observability and future routing semantics.
- Jelly J usage:
  - Future-proofing for strict delivery behavior and request correlation.
- Upstream risk:
  - Low if backward-compatibility is preserved (it is optional fields).

5. Atomic terminal launch API (`LaunchTerminalPane`) (`81dc6834`)
- Generic value:
  - Single host call for open+name+optional initial stdin write, avoiding plugin-side launch state machines.
- Jelly J usage:
  - Used for deterministic assistant pane launch with immediate command injection.
- Upstream risk:
  - Medium; behavior-sensitive API.

6. `LaunchTerminalPane` host hardening via action routing (`304046a1`)
- Generic value:
  - Aligns plugin launch path with stable action engine semantics used elsewhere.
  - Avoids completion-only false positives where pane IDs are returned but pane state is not durable.
- Jelly J usage:
  - Eliminates intermittent launch inconsistency for butler-triggered terminal panes.
- Upstream risk:
  - Medium; changes host execution path but preserves response contract.

7. Host-managed plugin logs API (`GetPluginLogs`, `ClearPluginLogs`) (`81dc6834`)
- Generic value:
  - Uniform plugin observability channel without plugin-specific logging hacks.
- Jelly J usage:
  - Debugging toggle and startup flows.
- Upstream risk:
  - Medium; memory policy and retention behavior need review.

8. Plugin harness command (`cargo xtask pluginharness`) (`81dc6834`)
- Generic value:
  - Reproducible plugin-platform regression checks for contributors.
- Jelly J usage:
  - Fast validation loop while iterating API/runtime behavior.
- Upstream risk:
  - Low; CI/developer tooling only.

9. Keybind pipe client scoping for specific-plugin delivery (`4aee2044`)
- Generic value:
  - In multi-client sessions, keybind-origin messages target the initiating client's plugin instance when available.
  - Prevents duplicate state transitions from per-client plugin replicas handling the same keypress.
- Jelly J usage:
  - Prevents duplicate toggle handling/flicker across client-scoped plugin instances.
- Upstream risk:
  - Medium; delivery semantics change for keybind-specific path.
  - Mitigation: this is limited to `PipeSource::Keybind`; CLI behavior remains unchanged.

## Genericness Assessment

- Generic/additive API work: items 1-8 are plugin-platform primitives, not tied to Jelly J names, commands, or UI.
- Potentially contentious semantic change: item 9 (keybind scoping) is still generic, but should likely be paired upstream with explicit delivery-mode documentation and/or `delivery_hint` policy.

## How Jelly J Uses These Changes

Current Jelly J plugin calls:
- `request_plugin_state_snapshot()` during startup/readiness.
- `launch_terminal_pane(...)` for atomic assistant launch.

Operationally, Jelly J benefits from:
- stable permission matching,
- deterministic startup snapshot,
- stable launch semantics,
- improved debugging surface.

## Recommendation for Upstreaming

Propose in slices:
1. Low-risk additive APIs and conversion tests (identity aliasing, permission introspection, snapshot command, envelope v2 fields).
2. LaunchTerminalPane API + routed-host implementation together (single proposal, with behavior tests).
3. Host-managed logs + harness tooling.
4. Keybind client-scoping as separate RFC/PR with explicit semantics discussion:
   - current behavior and failure mode in multi-client plugin replicas,
   - why keybind should be client-local by default,
   - interaction with future `delivery_hint` semantics.

## Open Question (Not solved by Zellij changes)

Jelly J requirement "one process per computer, available from all sessions" is an application architecture concern:
- Zellij sessions are separate servers and cannot directly share a process-local REPL pane.
- Solving this requires Jelly J control-plane IPC (global backend + per-session frontends/proxies), not only plugin API changes.
