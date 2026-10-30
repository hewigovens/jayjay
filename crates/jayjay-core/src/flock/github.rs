use jayjay_network::graphql::{GraphQlRequest, GraphQlResponse, GraphQlResponseError};
use jayjay_network::{Fetch, Header, Method, Request};
use serde::Deserialize;
use serde_json::{Value, json};

use super::{FlockMember, FlockVerification};
use crate::repo::forge_auth::TokenProvider;
use crate::{JayError, JayResult, SyncToken};

const QUERY: &str = include_str!("sponsorship.graphql");

#[derive(Deserialize)]
struct Data {
    viewer: FlockMember,
    user: Option<Sponsorable>,
}

#[derive(Deserialize)]
struct Sponsorable {
    #[serde(rename = "sponsorshipForViewerAsSponsor")]
    sponsorship: Option<Value>,
}

pub(super) fn verify(
    sponsorable: &str,
    fetch: &impl Fetch,
    tokens: &impl TokenProvider,
    sync: &SyncToken,
) -> JayResult<FlockVerification> {
    sync.check()?;
    if tokens.token().is_none() {
        return Ok(FlockVerification::SignInRequired);
    }
    let body = GraphQlRequest {
        query: QUERY,
        variables: json!({ "sponsorable": sponsorable }),
    };
    let response = tokens
        .fetch(fetch, |auth| {
            let mut request =
                Request::json("https://api.github.com/graphql", Method::Post, auth, &body)
                    .expect("the sponsorship query is JSON");
            request.headers.extend([
                Header::new("Accept", "application/vnd.github+json"),
                Header::new("X-GitHub-Api-Version", "2022-11-28"),
            ]);
            request
        })
        .map_err(JayError::internal)?;
    sync.check()?;
    if response.status == 401 {
        return Ok(FlockVerification::SignInRequired);
    }
    let response = response.error_for_status().map_err(|response| {
        JayError::internal(format!(
            "GitHub sponsorship verification failed (HTTP {}).",
            response.status
        ))
    })?;
    let reply: GraphQlResponse = response.json().map_err(JayError::internal)?;
    let data: Data = match reply.into_data() {
        Ok(data) => data,
        Err(GraphQlResponseError::Errors(errors)) => {
            if errors
                .iter()
                .any(|error| matches!(error.kind(), Some("INSUFFICIENT_SCOPES" | "FORBIDDEN")))
            {
                return Ok(FlockVerification::SignInRequired);
            }
            return Err(JayError::internal(
                "GitHub couldn't verify sponsorship with this login. Try signing in again.",
            ));
        }
        Err(GraphQlResponseError::MissingData) => return Err(invalid_reply()),
        Err(GraphQlResponseError::InvalidData(error)) => return Err(JayError::internal(error)),
    };
    if data.viewer.id.is_empty() || data.viewer.login.is_empty() {
        return Err(invalid_reply());
    }
    match data.user.ok_or_else(invalid_reply)?.sponsorship {
        Some(_) => Ok(FlockVerification::Unlocked {
            member: data.viewer,
        }),
        None => Ok(FlockVerification::NotSponsored {
            login: data.viewer.login,
        }),
    }
}

fn invalid_reply() -> JayError {
    JayError::internal("GitHub returned an incomplete sponsorship verification.")
}
