use jayjay_core::{CoreResult, Repo, Stack, StackedPrResult, SubmitStackLayer};

pub trait StackedPrProvider: Send + Sync {
    fn detect(&self, repo: &Repo, base_rev: &str, tip_rev: &str) -> CoreResult<Stack>;
    fn submit(&self, repo: &Repo, layers: Vec<SubmitStackLayer>) -> CoreResult<StackedPrResult>;
}

pub(crate) struct CoreStackedPrProvider;

impl StackedPrProvider for CoreStackedPrProvider {
    fn detect(&self, repo: &Repo, base_rev: &str, tip_rev: &str) -> CoreResult<Stack> {
        repo.detect_stack(base_rev, tip_rev)
    }

    fn submit(&self, repo: &Repo, layers: Vec<SubmitStackLayer>) -> CoreResult<StackedPrResult> {
        repo.submit_stack(layers)
    }
}
