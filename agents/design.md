# Design Guide

Load this file before changing visual style, copy, interaction patterns, or user-facing workflows.

## Users

JayJay users are developers who use jj for version control. They value keyboard-driven workflows, fast iteration, and tools that stay out of the way. Many come from git, so familiar patterns are useful only when adapted to jj's model.

## Brand

Clean, modern, approachable. The blue jaybird mascot adds personality without making the tool feel childish.

## Visual Direction

- Playful jaybird theme: blue gradient `#3B82F6` to `#1E3A8A`, orange accent `#F59E0B`, light blue `#93C5FD`.
- Reference: zed.dev, technical but polished.
- Anti-reference: cluttered enterprise tools and generic SaaS dashboards.
- Support light and dark modes through system preference.
- Prefer macOS-native feel: SF Symbols, system fonts, native controls.

## Interaction Principles

1. **Native first** - Use SwiftUI forms, system fonts, SF Symbols, and platform conventions.
2. **Keyboard-driven** - Every action should be reachable through command palette or shortcut.
3. **Dense, not cluttered** - Optimize for scanning, comparison, and repeated developer workflows.
4. **Performance is UX** - Prefer quiet refreshes over loading spinners where possible.
5. **Jujutsu-native** - Embrace changes, bookmarks, revsets, and working-copy semantics rather than forcing git branch/commit mental models.

## GPUI Layout

- Row-like controls should usually set `.w_full()` before centering content; `justify_center()` only centers within the element's own width.
