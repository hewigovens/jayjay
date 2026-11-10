use std::collections::VecDeque;
use std::ops::Range;

use super::TextSelection;

const UNDO_LIMIT: usize = 100;

#[derive(Debug, Clone)]
pub(crate) struct TextSnapshot<T = String> {
    pub(crate) text: T,
    pub(crate) selection: TextSelection,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum HistoryStep {
    Undo,
    Redo,
}

/// Where the current run of single-character edits continues, so the run undoes as one step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Run {
    Typing { end: usize },
    Deleting { at: usize },
}

#[derive(Debug, Clone)]
pub(crate) struct EditHistory<T = String> {
    undo: VecDeque<TextSnapshot<T>>,
    redo: Vec<TextSnapshot<T>>,
    run: Option<Run>,
}

impl<T> Default for EditHistory<T> {
    fn default() -> Self {
        Self {
            undo: VecDeque::new(),
            redo: Vec::new(),
            run: None,
        }
    }
}

impl<T: AsRef<str>> EditHistory<T> {
    pub(crate) fn record(&mut self, before: TextSnapshot<T>, range: &Range<usize>, inserted: &str) {
        let single_char = |text: &str| text.chars().count() == 1;
        let (continues, run) = if range.is_empty() && single_char(inserted) {
            (
                self.run == Some(Run::Typing { end: range.start }),
                Some(Run::Typing {
                    end: range.start + inserted.len(),
                }),
            )
        } else if inserted.is_empty()
            && before
                .text
                .as_ref()
                .get(range.clone())
                .is_some_and(single_char)
        {
            (
                matches!(self.run, Some(Run::Deleting { at }) if at == range.start || at == range.end),
                Some(Run::Deleting { at: range.start }),
            )
        } else {
            (false, None)
        };
        if !continues {
            self.push_undo(before);
        }
        self.run = run;
        self.redo.clear();
    }

    pub(crate) fn step(
        &mut self,
        step: HistoryStep,
        current: TextSnapshot<T>,
    ) -> Option<TextSnapshot<T>> {
        let restored = match step {
            HistoryStep::Undo => self.undo.pop_back()?,
            HistoryStep::Redo => self.redo.pop()?,
        };
        match step {
            HistoryStep::Undo => self.redo.push(current),
            HistoryStep::Redo => self.push_undo(current),
        }
        self.run = None;
        Some(restored)
    }

    pub(crate) fn clear(&mut self) {
        *self = Self::default();
    }

    fn push_undo(&mut self, snapshot: TextSnapshot<T>) {
        if self.undo.len() == UNDO_LIMIT {
            self.undo.pop_front();
        }
        self.undo.push_back(snapshot);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot(text: &str) -> TextSnapshot {
        TextSnapshot {
            text: text.to_owned(),
            selection: TextSelection::at(text.len()),
        }
    }

    fn undo(history: &mut EditHistory, current: &str) -> Option<String> {
        history
            .step(HistoryStep::Undo, snapshot(current))
            .map(|snapshot| snapshot.text)
    }

    #[test]
    fn a_run_of_single_character_deletions_undoes_as_one_step() {
        let mut history = EditHistory::default();
        history.record(snapshot("abcdef"), &(5..6), "");
        history.record(snapshot("abcde"), &(4..5), "");
        history.record(snapshot("abcd"), &(1..2), "");
        history.record(snapshot("acd"), &(1..2), "");
        assert_eq!(undo(&mut history, "ad").as_deref(), Some("abcd"));
        assert_eq!(undo(&mut history, "abcd").as_deref(), Some("abcdef"));
        assert_eq!(undo(&mut history, "abcdef"), None);
    }

    #[test]
    fn the_undo_stack_drops_its_oldest_steps_past_the_limit() {
        let mut history = EditHistory::default();
        for step in 0..=UNDO_LIMIT {
            history.record(snapshot(&step.to_string()), &(0..0), "xy");
        }
        let mut current = String::new();
        while let Some(previous) = undo(&mut history, &current) {
            current = previous;
        }
        assert_eq!(current, "1");
    }
}
