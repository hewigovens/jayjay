use std::collections::{HashMap, HashSet};

use jj_lib::backend::CommitId;
use jj_lib::op_store::RefTarget;
use jj_lib::ref_name::RefName;

/// Ref names per commit in name order; a conflicted ref that repeats a commit across sides is listed once.
pub(crate) struct CommitRefNames(HashMap<CommitId, Vec<String>>);

impl CommitRefNames {
    pub(crate) fn from_refs<'a>(refs: impl Iterator<Item = (&'a RefName, &'a RefTarget)>) -> Self {
        let mut names: HashMap<CommitId, Vec<String>> = HashMap::new();
        for (name, target) in refs {
            for id in target.added_ids().collect::<HashSet<_>>() {
                names
                    .entry(id.clone())
                    .or_default()
                    .push(name.as_str().to_owned());
            }
        }
        Self(names)
    }

    pub(crate) fn names(&self, commit_id: &CommitId) -> Vec<String> {
        self.0.get(commit_id).cloned().unwrap_or_default()
    }

    pub(crate) fn contains(&self, commit_id: &CommitId) -> bool {
        self.0.contains_key(commit_id)
    }
}
