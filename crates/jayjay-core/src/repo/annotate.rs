use std::collections::HashMap;

use chrono::DateTime;
use chrono::Local;
use chrono::Utc;
use jj_lib::annotate::FileAnnotator;
use jj_lib::backend::CommitId;
use jj_lib::commit::Commit as JjCommit;
use jj_lib::fileset::FilesetExpression;
use jj_lib::hex_util::encode_reverse_hex;
use jj_lib::object_id::ObjectId;
use jj_lib::revset::RevsetExpression;
use jj_lib::revset::RevsetFilterPredicate;
use jj_lib::revset::SymbolResolver;
use jj_lib::revset::SymbolResolverExtension;

use super::Repo;
use super::support::{block_on, on_worker_stack};
use crate::types::*;

impl Repo {
    /// Annotate a file: shows which revision last modified each line.
    pub fn annotate_file(&self, rev: &str, path: &str) -> CoreResult<Vec<AnnotationLine>> {
        let repo = self.get_repo();
        let starting = self.resolve_commit(&repo, rev)?;
        let repo_path = self.parse_repo_path(path)?;

        let mut annotator = block_on(FileAnnotator::from_commit(&starting, repo_path.as_ref()))
            .map_err(|e| CoreError::Internal {
                message: format!("init annotate: {e}"),
            })?;

        let user_domain = RevsetExpression::all();
        // jj's `SymbolResolver::new` takes `&[impl AsRef<dyn SymbolResolverExtension>]`;
        // `Box<T>` impls `AsRef<T>` but `&dyn T` does not, so we have to keep the Box.
        #[allow(clippy::borrowed_box)]
        let empty_extensions: &[&Box<dyn SymbolResolverExtension>] = &[];
        let resolver = SymbolResolver::new(repo.as_ref(), empty_extensions);
        let domain = user_domain
            .resolve_user_expression(repo.as_ref(), &resolver)
            .map_err(|e| CoreError::Internal {
                message: format!("resolve annotate domain: {e}"),
            })?;

        block_on(annotator.compute(repo.as_ref(), &domain)).map_err(|e| CoreError::Internal {
            message: format!("compute annotate: {e}"),
        })?;

        let annotation = annotator.to_annotation();
        let lines = on_worker_stack(|| {
            let mut cache: HashMap<CommitId, AnnotationMeta> = HashMap::new();
            let mut lines = Vec::new();
            for (line_idx, (commit_id_result, raw_line)) in annotation.lines().enumerate() {
                let commit_id = match commit_id_result {
                    Ok(id) | Err(id) => id,
                };
                let meta = match cache.get(commit_id) {
                    Some(m) => m.clone(),
                    None => {
                        let m = AnnotationMeta::load(repo.as_ref(), commit_id);
                        cache.insert(commit_id.clone(), m.clone());
                        m
                    }
                };
                let trimmed = raw_line.strip_suffix(b"\n").unwrap_or(raw_line);
                let text = String::from_utf8_lossy(trimmed).into_owned();
                lines.push(AnnotationLine {
                    change_id: meta.change_id,
                    author: meta.author,
                    timestamp: meta.timestamp,
                    line_number: (line_idx + 1) as u32,
                    text,
                });
            }
            lines
        });
        Ok(lines)
    }

    /// File history: list revisions that modified a given file path.
    pub fn file_history(&self, path: &str) -> CoreResult<Vec<ChangeInfo>> {
        let repo_path = self.parse_repo_path(path)?;
        let expression = RevsetExpression::filter(RevsetFilterPredicate::File(
            FilesetExpression::file_path(repo_path),
        ));
        self.log_typed(expression)
    }
}

#[derive(Clone)]
struct AnnotationMeta {
    change_id: ShortId,
    author: String,
    timestamp: String,
}

impl AnnotationMeta {
    fn load(repo: &dyn jj_lib::repo::Repo, commit_id: &CommitId) -> Self {
        let Ok(commit) = repo.store().get_commit(commit_id) else {
            return Self::placeholder(commit_id);
        };
        let change_id = encode_reverse_hex(commit.change_id().as_bytes());
        let short_len = block_on(repo.shortest_unique_change_id_prefix_len(commit.change_id()))
            .unwrap_or(change_id.len()) as u32;
        Self {
            change_id: ShortId::new(change_id, short_len),
            author: commit.author().email.clone(),
            timestamp: format_timestamp(&commit),
        }
    }

    fn placeholder(commit_id: &CommitId) -> Self {
        let id = commit_id.hex();
        let len = id.len() as u32;
        Self {
            change_id: ShortId::new(id, len),
            author: String::new(),
            timestamp: String::new(),
        }
    }
}

fn format_timestamp(commit: &JjCommit) -> String {
    let millis = commit.author().timestamp.timestamp.0;
    let Some(utc) = DateTime::<Utc>::from_timestamp_millis(millis) else {
        return String::new();
    };
    utc.with_timezone(&Local)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}
