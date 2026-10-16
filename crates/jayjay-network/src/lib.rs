//! Platform-neutral HTTP request and response handling for JayJay.

mod fetch;
#[cfg(feature = "json")]
mod json;
#[cfg(feature = "mock")]
mod mock;
mod path;
mod types;

#[cfg(feature = "blocking")]
mod native;

pub use fetch::Fetch;
#[cfg(feature = "mock")]
pub use mock::MockFetch;
#[cfg(feature = "blocking")]
pub use native::HttpClient;
pub use path::encode_path_segment;
pub use types::{Auth, DEFAULT_TEXT_CAP, Header, Method, NetError, Request, Response};
