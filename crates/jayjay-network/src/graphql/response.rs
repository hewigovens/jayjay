use std::{error::Error, fmt};

use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use super::GraphQlError;

#[derive(Debug, Deserialize)]
pub struct GraphQlResponse {
    data: Option<Value>,
    #[serde(default)]
    errors: Vec<GraphQlError>,
}

impl GraphQlResponse {
    /// Rejects errors before decoding data, which may be incomplete when a field failed.
    pub fn into_data<T: DeserializeOwned>(self) -> Result<T, GraphQlResponseError> {
        if !self.errors.is_empty() {
            return Err(GraphQlResponseError::Errors(self.errors));
        }
        let data = self.data.ok_or(GraphQlResponseError::MissingData)?;
        serde_json::from_value(data).map_err(GraphQlResponseError::InvalidData)
    }
}

#[derive(Debug)]
pub enum GraphQlResponseError {
    Errors(Vec<GraphQlError>),
    MissingData,
    InvalidData(serde_json::Error),
}

impl fmt::Display for GraphQlResponseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Errors(errors) => f.write_str(
                errors
                    .first()
                    .map(|error| error.message.as_str())
                    .filter(|message| !message.is_empty())
                    .unwrap_or("GraphQL request failed"),
            ),
            Self::MissingData => f.write_str("GraphQL returned no data"),
            Self::InvalidData(error) => write!(f, "Invalid GraphQL data: {error}"),
        }
    }
}

impl Error for GraphQlResponseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidData(error) => Some(error),
            _ => None,
        }
    }
}
