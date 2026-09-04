---
name: gpui-parity-audit
description: Audit or close GPUI Linux parity gaps against SwiftUI at affordance granularity. Use for "check shell parity" or "what is still missing in GPUI".
argument-hint: "[feature-or-parity-row]"
---

# GPUI Parity Audit

Load [Shell Parity](../../../agents/shell-parity.md) and [GPUI](../../../agents/gpui.md) first. Linux GPUI is the parity target; GPUI on macOS is development-only, and SwiftUI-only macOS integrations are intentional differences, not gaps. Unattended runs on a schedule audit only and never close gaps.

## Procedure

1. Start from the parity matrix, not from source exploration. For each requested row, map the SwiftUI implementation, the GPUI implementation, the shared Rust or UniFFI contract, the tests, and the guide claim.
2. Classify at affordance granularity. Confirm implementation and tests before calling anything absent; a row is not a feature-level verdict.
3. To close a gap, put shared behavior in Rust core first, then implement the GPUI affordance. Add one focused regression for state-sensitive behavior, such as commit-draft preservation across change movement.
4. Verify with focused GPUI tests while iterating, then once with `cargo clippy -p jayjay-gpui --all-targets -- -D warnings` and `just test-gpui`.
5. Do not edit the parity matrix, user guide, `docs/llms.txt`, or Help Book in the feature change; list the rows to refresh in the release docs pass.

## Report

Per row: status with evidence, intentional platform boundary or real gap, and what remains documented as missing. Never imply complete parity.
