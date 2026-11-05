use std::f32::consts::FRAC_PI_2;

use jayjay_core::ChangeInfo;

use crate::app::theme::Theme;

const NODE_RADIUS_BASE: f32 = 4.5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NodeShape {
    Circle,
    Diamond,
}

#[derive(Clone, Copy)]
pub(crate) enum NodeFill {
    Filled(u32),
    Outlined(u32, f32),
}

#[derive(Clone, Copy)]
pub(crate) struct DagNodeStyle {
    pub shape: NodeShape,
    pub radius: f32,
    pub fill: NodeFill,
}

impl DagNodeStyle {
    pub fn resolve(change: &ChangeInfo, theme: &Theme) -> Self {
        let has_bookmark = !change.bookmarks.is_empty();
        let shape = if change.is_immutable {
            NodeShape::Diamond
        } else {
            NodeShape::Circle
        };
        let circle_radius = if has_bookmark {
            NODE_RADIUS_BASE + 1.0
        } else {
            NODE_RADIUS_BASE
        };
        let radius = match shape {
            NodeShape::Diamond => circle_radius * FRAC_PI_2.sqrt(),
            NodeShape::Circle => circle_radius,
        };
        let fill = if change.is_working_copy {
            NodeFill::Filled(theme.selected_accent)
        } else if change.has_conflict {
            NodeFill::Filled(theme.tag_conflict_fg)
        } else if change.is_empty {
            NodeFill::Outlined(theme.dag_node, 1.5)
        } else if has_bookmark {
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

#[cfg(test)]
mod tests {
    use super::{DagNodeStyle, NodeShape};
    use crate::app::theme::Theme;
    use jayjay_core::mock::change_info;

    #[test]
    fn node_shape_follows_immutability_regardless_of_bookmarks() {
        for bookmarks in [vec![], vec!["main".to_owned()]] {
            let mut change = change_info("c", "a");
            change.bookmarks = bookmarks;
            for (immutable, expected) in [(false, NodeShape::Circle), (true, NodeShape::Diamond)] {
                change.is_immutable = immutable;
                assert_eq!(
                    DagNodeStyle::resolve(&change, &Theme::light()).shape,
                    expected
                );
            }
        }
    }
}
