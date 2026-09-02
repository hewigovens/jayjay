use jayjay_core::ChangeInfo;

use crate::app::theme::Theme;
use crate::repo::revset::is_trunk_bookmark;

#[derive(Clone, Copy)]
pub(super) enum NodeShape {
    Circle,
    Diamond,
}

#[derive(Clone, Copy)]
pub(super) enum NodeFill {
    Filled(u32),
    Outlined(u32, f32),
}

#[derive(Clone, Copy)]
pub(super) struct DagNodeStyle {
    pub shape: NodeShape,
    pub radius: f32,
    pub fill: NodeFill,
}

impl DagNodeStyle {
    pub fn resolve(change: &ChangeInfo, theme: &Theme, base_radius: f32) -> Self {
        let is_trunk = change.bookmarks.iter().any(|b| is_trunk_bookmark(b));
        let has_bookmark = !change.bookmarks.is_empty();
        let shape = if is_trunk {
            NodeShape::Diamond
        } else {
            NodeShape::Circle
        };
        let radius = if is_trunk || has_bookmark {
            base_radius + 1.0
        } else {
            base_radius
        };
        let fill = if change.is_working_copy {
            NodeFill::Filled(theme.selected_accent)
        } else if change.has_conflict {
            NodeFill::Filled(theme.tag_conflict_fg)
        } else if change.is_empty {
            NodeFill::Outlined(theme.dag_node, 1.5)
        } else if is_trunk || has_bookmark {
            NodeFill::Outlined(theme.selected_accent, 1.8)
        } else {
            NodeFill::Filled(theme.dag_node)
        };
        Self {
            shape,
            radius,
            fill,
        }
    }
}
