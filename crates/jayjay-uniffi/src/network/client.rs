use std::sync::Arc;

use jayjay_network::{Auth, DEFAULT_TEXT_CAP, NetError, Request, Response};

use super::{HttpTransport, NetworkError};

#[derive(uniffi::Object)]
pub struct NetworkClient {
    transport: Arc<dyn HttpTransport>,
}

#[uniffi::export]
impl NetworkClient {
    #[uniffi::constructor]
    pub fn new(transport: Arc<dyn HttpTransport>) -> Arc<Self> {
        Arc::new(Self { transport })
    }

    pub async fn fetch(&self, request: Request) -> Result<Response, NetworkError> {
        let max_bytes = request.max_response_bytes;
        let mut response = self.transport.fetch(request).await?;
        response.body.truncate(max_bytes as usize);
        Ok(response)
    }

    pub async fn get_text(
        &self,
        url: String,
        authorization: Option<String>,
    ) -> Result<String, NetworkError> {
        let bytes = self.get_bytes(url, DEFAULT_TEXT_CAP, authorization).await?;
        String::from_utf8(bytes).map_err(|_| NetError::Transport.into())
    }

    pub async fn get_bytes(
        &self,
        url: String,
        max_bytes: u32,
        authorization: Option<String>,
    ) -> Result<Vec<u8>, NetworkError> {
        let auth = Auth::authorization(authorization);
        let request = Request::get(url, max_bytes, &auth);
        let response = self.fetch(request).await?;
        response.into_bytes(max_bytes).map_err(Into::into)
    }
}
