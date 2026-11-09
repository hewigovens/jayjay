use gpui::Context;

use super::bisect::BisectSheet;
use super::confirmation::Confirmation;
use super::converge::ConvergeSheet;
use super::dag_drag::DagRebaseRequest;
use super::stacked_pr::StackedPrState;
use super::{PrImportState, RepoWindow, TextModalState};

/// Sheets and prompts that cover the repo window one at a time.
pub(crate) enum RepoModal {
    Text(TextModalState),
    Bisect(BisectSheet),
    PrImport(PrImportState),
    Confirmation(Confirmation),
    Rebase(DagRebaseRequest),
    StackedPr(StackedPrState),
    Converge(ConvergeSheet),
    RatingPrompt,
}

impl RepoWindow {
    /// An open modal owns the window like a SwiftUI sheet, so a command that would present another is ignored until it closes.
    pub(crate) fn present_modal(&mut self, modal: RepoModal, cx: &mut Context<Self>) -> bool {
        if self.modal.is_some() {
            return false;
        }
        self.modal = Some(modal);
        cx.notify();
        true
    }

    pub(crate) fn rating_prompt_open(&self) -> bool {
        matches!(self.modal, Some(RepoModal::RatingPrompt))
    }

    pub(crate) fn close_modal(&mut self, is: fn(&RepoModal) -> bool, cx: &mut Context<Self>) {
        if self.modal.take_if(|modal| is(modal)).is_some() {
            cx.notify();
        }
    }

    /// Each sheet's own close, so one that is mid-write (stacked PR submit, converge apply, PR import) stays up.
    pub(crate) fn dismiss_modal(&mut self, cx: &mut Context<Self>) {
        match &self.modal {
            Some(RepoModal::Text(_)) => self.close_text_modal(cx),
            Some(RepoModal::Bisect(_)) => self.close_bisect_sheet(cx),
            Some(RepoModal::PrImport(_)) => self.cancel_pr_import(cx),
            Some(RepoModal::Confirmation(_)) => self.cancel_confirmation(cx),
            Some(RepoModal::Rebase(_)) => self.cancel_drag_rebase(cx),
            Some(RepoModal::StackedPr(_)) => self.close_stacked_pr(cx),
            Some(RepoModal::Converge(_)) => self.close_converge(cx),
            Some(RepoModal::RatingPrompt) => self.close_rating_prompt(cx),
            None => {}
        }
    }

    pub(crate) fn text_modal(&self) -> Option<&TextModalState> {
        match &self.modal {
            Some(RepoModal::Text(modal)) => Some(modal),
            _ => None,
        }
    }

    pub(crate) fn pr_import(&self) -> Option<&PrImportState> {
        match &self.modal {
            Some(RepoModal::PrImport(state)) => Some(state),
            _ => None,
        }
    }

    pub(crate) fn stacked_pr(&self) -> Option<&StackedPrState> {
        match &self.modal {
            Some(RepoModal::StackedPr(state)) => Some(state),
            _ => None,
        }
    }
}
