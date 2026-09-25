/// Which commits a rebase moves, matching `jj rebase -s` and `-b`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RebaseMode {
    Source,
    Branch,
}
