use jj_lib::bisect::BisectionResult;
use jj_lib::commit::Commit;
use jj_lib::hex_util::encode_reverse_hex;
use jj_lib::object_id::ObjectId as _;

use super::session::{Session, change_subject};
use crate::repo::Repo;
use crate::types::*;

impl Repo {
    pub(super) fn finish(&self, session: &mut Session, result: BisectionResult) -> JayResult<()> {
        let FoundPieces {
            kind,
            message,
            bad,
            possibly_bad,
        } = match result {
            BisectionResult::Found(bad) => found_pieces(false, bad, Vec::new()),
            BisectionResult::FoundDespiteSkips {
                bad_commits,
                possibly_bad,
            } => found_pieces(true, bad_commits, possibly_bad),
            BisectionResult::Indeterminate => FoundPieces {
                kind: BisectResultKind::Indeterminate,
                message: jayjay_i18n::repo::bisect_no_first_bad(),
                bad: Vec::new(),
                possibly_bad: Vec::new(),
            },
            BisectionResult::Abort => FoundPieces {
                kind: BisectResultKind::Abort,
                message: jayjay_i18n::repo::bisect_command_not_found(),
                bad: Vec::new(),
                possibly_bad: Vec::new(),
            },
            BisectionResult::VerificationFailed => {
                return Err(JayError::internal(
                    "bisect precondition verification is not used",
                ));
            }
        };
        let (restore_note, kept) = if session.temps.is_empty() {
            (None, Vec::new())
        } else {
            (
                self.restore_target(&session.restore)?,
                self.kept_changes(&session.temps)?,
            )
        };
        session.finish(BisectResultView {
            kind,
            message,
            bad,
            possibly_bad,
            kept,
            restore_note,
        });
        Ok(())
    }
}

struct FoundPieces {
    kind: BisectResultKind,
    message: String,
    bad: Vec<BisectChange>,
    possibly_bad: Vec<BisectChange>,
}

fn found_pieces(despite_skips: bool, bad: Vec<Commit>, possibly_bad: Vec<Commit>) -> FoundPieces {
    let bad: Vec<BisectChange> = bad.iter().map(change_view).collect();
    let possibly_bad: Vec<BisectChange> = possibly_bad.iter().map(change_view).collect();
    let (kind, message) = if despite_skips {
        (
            BisectResultKind::FoundDespiteSkips,
            jayjay_i18n::repo::bisect_first_bad_possibly_skipped(
                &change_list(&bad),
                &change_list(&possibly_bad),
            ),
        )
    } else {
        (
            BisectResultKind::Found,
            jayjay_i18n::repo::bisect_first_bad(bad.len(), &change_list(&bad)),
        )
    };
    FoundPieces {
        kind,
        message,
        bad,
        possibly_bad,
    }
}

fn change_list(changes: &[BisectChange]) -> String {
    if changes.is_empty() {
        return jayjay_i18n::repo::bisect_none();
    }
    changes
        .iter()
        .map(|change| change_subject(&change.description))
        .collect::<Vec<_>>()
        .join(", ")
}

pub(super) fn change_view(commit: &Commit) -> BisectChange {
    BisectChange {
        change_id: change_id_of(commit),
        commit_id: commit.id().hex(),
        description: commit.description().to_owned(),
    }
}

pub(super) fn change_id_of(commit: &Commit) -> String {
    encode_reverse_hex(commit.change_id().as_bytes())
}
