use jayjay_core::{JayResult, Repo, Stack, StackedPrResult, SubmitStackLayer};

pub trait StackedPrProvider: Send + Sync {
    fn detect(&self, repo: &Repo, base_rev: &str, tip_rev: &str) -> JayResult<Stack>;
    fn submit(&self, repo: &Repo, layers: Vec<SubmitStackLayer>) -> JayResult<StackedPrResult>;
}

pub(crate) struct CoreStackedPrProvider;

impl StackedPrProvider for CoreStackedPrProvider {
    fn detect(&self, repo: &Repo, base_rev: &str, tip_rev: &str) -> JayResult<Stack> {
        repo.detect_stack(base_rev, tip_rev)
    }

    fn submit(&self, repo: &Repo, layers: Vec<SubmitStackLayer>) -> JayResult<StackedPrResult> {
        repo.submit_stack(layers)
    }
}
