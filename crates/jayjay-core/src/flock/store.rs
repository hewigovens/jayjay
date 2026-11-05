use std::path::Path;

use jayjay_network::{Fetch, HttpClient};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{FlockMember, FlockVerification, github};
use crate::json_store::JsonStore;
use crate::repo::forge_auth::TokenProvider;
use crate::{JayError, JayResult, RepoHost, SyncToken};

// Only the hash ships, and the code is random enough that the hash can't be brute-forced.
const SPONSOR_CODE_SHA256: &str =
    "b05fb29aec2d8d7aefb9837aa3446b3795093626281fd4a80259ff6d656a646c";

#[derive(Clone, Default, Deserialize, Serialize)]
struct State {
    member: Option<FlockMember>,
}

impl State {
    fn normalize(&mut self) {
        if self
            .member
            .as_ref()
            .is_some_and(|member| member.id.is_empty() || member.login.is_empty())
        {
            self.member = None;
        }
    }
}

pub struct FlockStore(JsonStore<State>);

impl FlockStore {
    pub fn load() -> Self {
        Self(JsonStore::load(
            "flock",
            "JAYJAY_FLOCK_PATH",
            "flock.json",
            State::normalize,
        ))
    }

    pub fn at(path: &Path) -> Self {
        Self(JsonStore::new(
            "flock",
            Some(path.to_owned()),
            State::normalize,
        ))
    }

    pub fn member(&mut self) -> Option<FlockMember> {
        self.0.read().member.clone()
    }

    pub fn verify(&mut self, sponsorable: &str, sync: &SyncToken) -> JayResult<FlockVerification> {
        self.verify_existing_login(sponsorable, &HttpClient::default(), &RepoHost::GitHub, sync)
    }

    pub fn redeem(&mut self, code: &str, name: &str) -> JayResult<Option<FlockMember>> {
        // Lets UI scenes redeem a test code, since the real one stays out of the repository.
        #[cfg(debug_assertions)]
        if let Ok(hash) = std::env::var("JAYJAY_FLOCK_CODE_SHA256") {
            return self.redeem_with(code, name, &hash);
        }
        self.redeem_with(code, name, SPONSOR_CODE_SHA256)
    }

    fn redeem_with(
        &mut self,
        code: &str,
        name: &str,
        expected_sha256: &str,
    ) -> JayResult<Option<FlockMember>> {
        let name = name.trim();
        let hash = hex::encode(Sha256::digest(code.trim().to_ascii_uppercase()));
        if name.is_empty() || hash != expected_sha256 {
            return Ok(None);
        }
        let member = FlockMember {
            id: "sponsor-code".into(),
            login: name.into(),
            name: None,
        };
        self.save(&member)?;
        Ok(Some(member))
    }

    pub fn forget(&mut self) -> JayResult<()> {
        let saved = self.0.update(|state| state.member.take().is_some());
        if saved.member.is_some() {
            return Err(JayError::internal("Couldn't forget the Flock account."));
        }
        Ok(())
    }

    pub(super) fn verify_existing_login(
        &mut self,
        sponsorable: &str,
        fetch: &impl Fetch,
        tokens: &impl TokenProvider,
        sync: &SyncToken,
    ) -> JayResult<FlockVerification> {
        // Without read:user GitHub may hide a sponsorship, so only the profile sign-in can prove there is none.
        Ok(match self.verify_with(sponsorable, fetch, tokens, sync)? {
            FlockVerification::NotSponsored { .. } => FlockVerification::SignInRequired,
            verification => verification,
        })
    }

    pub(super) fn verify_with(
        &mut self,
        sponsorable: &str,
        fetch: &impl Fetch,
        tokens: &impl TokenProvider,
        sync: &SyncToken,
    ) -> JayResult<FlockVerification> {
        let verification = github::verify(sponsorable, fetch, tokens, sync)?;
        sync.check()?;
        if let FlockVerification::Unlocked { member } = &verification {
            self.save(member)?;
        }
        Ok(verification)
    }

    fn save(&mut self, member: &FlockMember) -> JayResult<()> {
        let saved = self.0.update(|state| {
            if state.member.as_ref() == Some(member) {
                return false;
            }
            state.member = Some(member.clone());
            true
        });
        if saved.member.as_ref() != Some(member) {
            return Err(JayError::internal("Couldn't save the Flock unlock."));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
