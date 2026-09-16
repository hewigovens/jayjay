use crate::repo::Repo;
use crate::types::*;

impl Repo {
    /// `jj commit -m <message>` = describe @ + new empty change on top.
    pub fn jj_commit(&self, message: &str) -> CoreResult<()> {
        self.run_jj_reload(&["commit", "-m", message])
    }
}
