use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct FlockMember {
    pub id: String,
    pub login: String,
    pub name: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FlockVerification {
    SignInRequired,
    NotSponsored { login: String },
    Unlocked { member: FlockMember },
}
