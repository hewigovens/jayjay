//! Platform-neutral HTTP request and response handling for JayJay.

mod types;

#[cfg(feature = "blocking")]
mod native;

#[cfg(feature = "blocking")]
pub use native::HttpClient;
pub use types::{Auth, DEFAULT_TEXT_CAP, Header, Method, NetError, Request, Response};
