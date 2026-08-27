use jayjay_network::{Request, Response};

use super::NetworkError;

#[uniffi::export(rust, foreign)]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
pub trait HttpTransport: Send + Sync {
    async fn fetch(&self, request: Request) -> Result<Response, NetworkError>;
}
