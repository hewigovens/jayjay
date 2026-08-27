use jayjay_network::NetError;

// This adapter stays local so UniFFI can convert unexpected foreign callback failures without adding an FFI dependency to jayjay-network.
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum NetworkError {
    #[error("network resource not found")]
    NotFound,
    #[error("HTTP {status}")]
    Http { status: u16 },
    #[error("network transport failed: {message}")]
    Transport { message: String },
}

impl From<NetError> for NetworkError {
    fn from(error: NetError) -> Self {
        match error {
            NetError::NotFound => Self::NotFound,
            NetError::Http(status) => Self::Http { status },
            NetError::Transport => Self::Transport {
                message: error.to_string(),
            },
        }
    }
}

impl From<uniffi::UnexpectedUniFFICallbackError> for NetworkError {
    fn from(error: uniffi::UnexpectedUniFFICallbackError) -> Self {
        Self::Transport {
            message: error.reason,
        }
    }
}
