---
name: gpui-parity-audit
description: Audit or close GPUI Linux parity gaps against SwiftUI at affordance granularity. Use for "check shell parity" or "what is still missing in GPUI".
argument-hint: "[feature-or-parity-row]"
---

# GPUI Parity Audit

Load [Shell Parity](../../../agents/shell-parity.md) and [GPUI](../../../agents/gpui.md) first. Linux GPUI is the parity target; GPUI on macOS is development-only, and SwiftUI-only macOS integrations are intentional differences, not gaps. Audit requests are findings-only; close gaps only when the user or saved task authorizes implementation.

## Procedure

1. Use the parity matrix as an index of release claims; current source and tests establish implementation status. For each requested row, map the SwiftUI implementation, the GPUI implementation, the shared Rust or UniFFI contract, the tests, and the guide claim.
2. Classify at affordance granularity. Confirm implementation and tests before calling anything absent; a row is not a feature-level verdict.
3. When closing an authorized gap, reuse the existing shared contract. Put new shared behavior in Rust core and shell-specific interaction in GPUI. Add one focused regression for state-sensitive behavior, such as commit-draft preservation across change movement.
4. For implementation, validate the changed behavior using [Testing](../../../agents/testing.md) and the [inner-loop policy](../../../AGENTS.md#inner-loop). Linux-only paths need Linux evidence; a macOS build does not establish Linux behavior. Do not repeat passing tests or run lint solely to finish an audit.
5. Feature changes list rows for the release docs pass. Edit the parity matrix, user guide, `docs/llms.txt`, or Help Book only during that pass or an explicit request to update those docs, per [AGENTS.md](../../../AGENTS.md#user-facing-docs).

## Report

Per row: status with evidence, intentional platform boundary or real gap, and what remains documented as missing. Limit parity conclusions to the affordances and platforms verified.
