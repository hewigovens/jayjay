#[cfg(not(target_os = "macos"))]
use gpui::Action;
use jayjay_core::UndoState;
use jayjay_i18n::repo;

#[cfg(not(target_os = "macos"))]
use crate::app::actions::{RedoOperation, UndoOperation};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OperationStep {
    Undo,
    Redo,
}

#[cfg(not(target_os = "macos"))]
impl OperationStep {
    pub(crate) fn for_action(action: &dyn Action) -> Option<Self> {
        if action.as_any().is::<UndoOperation>() {
            Some(Self::Undo)
        } else if action.as_any().is::<RedoOperation>() {
            Some(Self::Redo)
        } else {
            None
        }
    }

    pub(crate) fn menu_title(self, description: Option<&str>) -> String {
        match (self, description) {
            (Self::Undo, Some(operation)) => jayjay_i18n::menu::undo_operation(operation),
            (Self::Undo, None) => repo::undo(),
            (Self::Redo, Some(operation)) => jayjay_i18n::menu::redo_operation(operation),
            (Self::Redo, None) => jayjay_i18n::menu::redo(),
        }
    }
}

impl OperationStep {
    pub(crate) fn description(self, state: &UndoState) -> Option<&str> {
        match self {
            Self::Undo => state.undo_description.as_deref(),
            Self::Redo => state.redo_description.as_deref(),
        }
    }

    pub(crate) fn stepped(self, operation: &str) -> String {
        match self {
            Self::Undo => repo::undid_operation(operation),
            Self::Redo => repo::redid_operation(operation),
        }
    }
}
