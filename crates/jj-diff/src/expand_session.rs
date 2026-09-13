use std::sync::Arc;

use crate::expand::ExpandableDiff;
use crate::types::{
    ContextExpansion, ContextExpansionError, ContextExpansionResult, FileDiff, LineSpan,
};

/// One expansion request. `AllRegions` carries no region id so a queued expand-all outlives the separator that started it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextExpansionRequest {
    Region {
        region_id: u32,
        expansion: ContextExpansion,
    },
    AllRegions,
}

impl ContextExpansionRequest {
    fn targets_available_region(self, diff: &FileDiff) -> bool {
        diff.lines.iter().any(|line| match self {
            Self::AllRegions => line.context_region.is_some(),
            Self::Region { region_id, .. } => line
                .context_region
                .is_some_and(|region| region.id == region_id),
        })
    }
}

pub struct ContextExpansionSource {
    pub diff: FileDiff,
    pub old_content: Arc<str>,
    pub new_content: Arc<str>,
}

/// An accepted request detached from its session so the expansion runs off the UI thread.
pub struct ContextExpansionAttempt {
    generation: u64,
    request: ContextExpansionRequest,
    document: Option<ExpandableDiff>,
}

impl ContextExpansionAttempt {
    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn needs_source(&self) -> bool {
        self.document.is_none()
    }

    pub fn run(self, source: Option<ContextExpansionSource>) -> ContextExpansionOutcome {
        let document = self.document.or_else(|| {
            source.map(|source| {
                ExpandableDiff::from_shared(source.diff, source.old_content, source.new_content)
            })
        });
        let Some(mut document) = document else {
            return ContextExpansionOutcome {
                generation: self.generation,
                document: None,
                result: Err(ContextExpansionError::SessionUnavailable),
            };
        };
        let result = match self.request {
            ContextExpansionRequest::Region {
                region_id,
                expansion,
            } => document.expand(region_id, expansion),
            ContextExpansionRequest::AllRegions => document.expand_all(),
        };
        ContextExpansionOutcome {
            generation: self.generation,
            document: Some(document),
            result,
        }
    }
}

pub struct ContextExpansionOutcome {
    generation: u64,
    document: Option<ExpandableDiff>,
    result: Result<ContextExpansionResult, ContextExpansionError>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContextExpansionReveal {
    pub generation: u64,
    pub new_lines: LineSpan,
}

pub enum ContextExpansionFinish {
    Discarded,
    Applied {
        diff: FileDiff,
        reveal: Option<ContextExpansionReveal>,
        /// Monotonic across resets so a shell can hand its renderer a token that is never reused.
        selection_generation: u64,
        next: Option<ContextExpansionRequest>,
    },
    Failed {
        message: String,
    },
}

/// Coalescing, supersession, and stale-basis policy shared by both shells: `begin` with the displayed diff's key, run the attempt off the UI thread, `finish` with the key as it stands then.
#[derive(Default)]
pub struct ContextExpansionSession {
    basis: Option<String>,
    document: Option<ExpandableDiff>,
    generation: u64,
    in_flight: bool,
    pending: Option<ContextExpansionRequest>,
    selection_generation: u64,
}

impl ContextExpansionSession {
    /// Revalidate after preparing a finished result off-thread; a reset or newer request can supersede it before the shell installs it.
    pub fn is_current(&self, basis: &str, generation: u64) -> bool {
        self.basis.as_deref() == Some(basis) && self.generation == generation
    }

    /// Returns the attempt to run, or `None` when the request was queued behind an in-flight one or no longer targets a live region.
    pub fn begin(
        &mut self,
        basis: &str,
        request: ContextExpansionRequest,
    ) -> Option<ContextExpansionAttempt> {
        if self.basis.as_deref() != Some(basis) {
            self.reset();
            self.basis = Some(basis.to_owned());
        }
        if let Some(document) = self.document.as_ref()
            && !request.targets_available_region(document.diff())
        {
            return None;
        }
        if self.in_flight {
            self.pending = Some(request);
            return None;
        }
        self.generation = self.generation.wrapping_add(1);
        self.in_flight = true;
        Some(ContextExpansionAttempt {
            generation: self.generation,
            request,
            document: self.document.take(),
        })
    }

    pub fn finish(
        &mut self,
        basis: &str,
        outcome: ContextExpansionOutcome,
    ) -> ContextExpansionFinish {
        // A newer attempt already owns the session, so this one must not touch its state.
        if self.generation != outcome.generation {
            return ContextExpansionFinish::Discarded;
        }
        if self.basis.as_deref() != Some(basis) {
            self.reset();
            return ContextExpansionFinish::Discarded;
        }
        self.in_flight = false;
        self.document = outcome.document;
        match outcome.result {
            Ok(result) => {
                self.selection_generation = self.selection_generation.wrapping_add(1);
                ContextExpansionFinish::Applied {
                    reveal: revealed_lines(&result, self.generation),
                    diff: result.diff,
                    selection_generation: self.selection_generation,
                    next: self.pending.take(),
                }
            }
            Err(error) => {
                self.pending = None;
                ContextExpansionFinish::Failed {
                    message: error_message(&error).to_owned(),
                }
            }
        }
    }

    /// True once a session exists for a different diff, so a shell can drop it eagerly instead of waiting for the next request.
    pub fn is_stale(&self, basis: &str) -> bool {
        self.basis
            .as_deref()
            .is_some_and(|current| current != basis)
    }

    pub fn reset(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.basis = None;
        self.document = None;
        self.in_flight = false;
        self.pending = None;
    }
}

fn revealed_lines(
    result: &ContextExpansionResult,
    generation: u64,
) -> Option<ContextExpansionReveal> {
    let start = result.inserted.start as usize;
    let end = start.checked_add(result.inserted.count as usize)?;
    let first_new_line = result
        .diff
        .lines
        .get(start..end)?
        .iter()
        .filter_map(|line| line.new_line_no)
        .min()?;
    Some(ContextExpansionReveal {
        generation,
        new_lines: LineSpan {
            start: first_new_line,
            count: result.inserted.count,
        },
    })
}

fn error_message(error: &ContextExpansionError) -> &'static str {
    match error {
        ContextExpansionError::UnknownRegion { .. } => {
            "The diff changed before its context could be expanded. Refresh and try again."
        }
        ContextExpansionError::InvalidLineCount
        | ContextExpansionError::InvalidRegion { .. }
        | ContextExpansionError::MissingSourceLine { .. }
        | ContextExpansionError::SessionUnavailable => {
            "This context could not be expanded. Refresh the diff and try again."
        }
    }
}
