//! Author avatar fetcher / cache (GPUI shell only).
//!
//! Strategy (mirrors the macOS shell's `AvatarStore`):
//!   1. `<id>+<user>@users.noreply.github.com`:
//!      - bots (`<user>` ends in `[bot]`) → API `user/<id>` → its `in/<app-id>` avatar.
//!      - otherwise → `https://avatars.githubusercontent.com/u/<id>`.
//!   2. `<user>@users.noreply.github.com` (old form) → `https://github.com/<user>.png?size=N`.
//!   3. `<id>-<user>@users.noreply.gitlab.com` → gitlab.com `users?username=<user>` → its `avatar_url`.
//!   4. Fallback to Gravatar.
//!
//! Once cached (`$HOME/.cache/jayjay/avatars/<email-hash>.png`) the element renders
//! straight from the file with no network call.

mod cache;
mod element;
mod resolve;

pub(crate) use cache::{cache_path, fetch_blocking};
pub(crate) use element::element;
