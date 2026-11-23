use std::sync::Arc;

use gpui::{App, Context, SharedString, point, px};
use jayjay_core::DiffHunk;
use jayjay_core::diff::FileDiff;
use jayjay_review::ReviewNoteStatus;

use super::layout::StackLayout;
use super::state::DiffStackState;
use crate::app::theme::{Theme, theme};
use crate::diff::file_status::{is_lfs, is_submodule};
use crate::diff::{
    DetailMode, DiffShape, DiffViewMode, StackedRows, file_row_height, hunk_is_image,
};
use crate::repo::view_model::{DiffLoadState, RepoViewModel};
use crate::repo::window::{DiffWrapCacheSlot, RepoWindow};
use crate::ui::icons::glyph;

const DIVIDER: f32 = 1.;
const MESSAGE_HEIGHT: f32 = 44.;
const DETAILED_MESSAGE_HEIGHT: f32 = 76.;
const STACKED_LINE_LIMIT: usize = 3000;
const IMAGE_HEIGHT: f32 = 320.;
/// Built past both edges of the view, so a short scroll shows rows that are already laid out.
const OVERDRAW: f32 = 400.;

pub(crate) struct StackPlan {
    pub(super) files: Arc<Vec<DiffHunk>>,
    pub(super) order: Vec<usize>,
    pub(super) layout: StackLayout,
    /// The block that `blocks` starts at.
    pub(super) first: usize,
    header_height: f32,
    line_height: f32,
    pub(super) blocks: Vec<StackBlock>,
}

pub(super) struct StackBlock {
    pub(super) hunk_ix: usize,
    pub(super) body_height: f32,
    pub(super) body: StackBody,
}

pub(super) enum StackBody {
    Rows {
        diff: Arc<FileDiff>,
        view_mode: DiffViewMode,
        notes: Vec<ReviewNoteStatus>,
        wrap: DiffWrapCacheSlot,
        banner_height: f32,
        supports_file_editor: bool,
    },
    Image,
    Loading,
    Collapsed,
    Message(StackMessage),
}

pub(super) struct StackMessage {
    pub(super) glyph: &'static str,
    pub(super) title: SharedString,
    pub(super) detail: Option<SharedString>,
    pub(super) action: Option<StackMessageAction>,
}

#[derive(Clone, Copy)]
pub(super) enum StackMessageAction {
    ShowDiff { supports_file_editor: bool },
    Retry,
}

impl StackPlan {
    pub(super) fn frame_height(t: &Theme) -> f32 {
        file_row_height(t) + DIVIDER
    }
}

impl RepoWindow {
    pub(crate) fn sync_diff_stack(&mut self, cx: &mut Context<Self>) {
        let mut stack = std::mem::take(&mut self.diff.stack);
        let revision = self.vm.read(cx).selected_revision();
        if stack.revision != revision {
            stack.revision = revision;
            stack.solo = false;
            stack.collapsed.expand_all();
            stack.heights.clear();
            stack.wraps.clear();
            stack.shapes.clear();
            stack.plan = None;
            stack.scroll.set_offset(point(px(0.), px(0.)));
        }
        if !crate::app::config::current(cx).diff.show_all_file_diffs {
            stack.solo = false;
        }
        let was_stacked = stack.plan.is_some();
        stack.plan = self.plan_diff_stack(&mut stack, cx);
        // Only the selected file reloads when a diff setting changes, so the files in view ask for theirs.
        if let Some(plan) = &stack.plan
            && plan
                .blocks
                .iter()
                .any(|block| matches!(block.body, StackBody::Loading))
        {
            let files = plan.files.clone();
            self.vm
                .update(cx, |vm, cx| vm.preload_diffs_async(files, cx));
        }
        if stack.plan.is_none() {
            stack.wraps.clear();
            stack.shapes.clear();
            if was_stacked {
                // A stacked file's panels are as tall as its rows, which is no viewport for the file shown alone.
                for bounds in [
                    &self.diff.unified_bounds,
                    &self.diff.sbs_old_bounds,
                    &self.diff.sbs_new_bounds,
                ] {
                    bounds.set(None);
                }
            }
        }
        self.diff.stack = stack;
    }

    /// The files a stack would show and their order, when the selection can be shown as one.
    pub(super) fn stackable_files(&self, cx: &App) -> Option<(Arc<Vec<DiffHunk>>, Vec<usize>)> {
        if !crate::app::config::current(cx).diff.show_all_file_diffs {
            return None;
        }
        let vm = self.vm.read(cx);
        if vm.compare.is_some()
            || vm.detail_mode != DetailMode::Diff
            || vm.selection_without_diff_count().is_some()
        {
            return None;
        }
        let files = vm.files.clone()?;
        let order = self.ordered_visible_file_indices(cx);
        (order.len() > 1).then_some((files, order))
    }

    fn plan_diff_stack(&self, stack: &mut DiffStackState, cx: &App) -> Option<StackPlan> {
        if stack.solo {
            return None;
        }
        let (files, order) = self.stackable_files(cx)?;
        let vm = self.vm.read(cx);
        let t = theme(cx);
        let frame = StackPlan::frame_height(t);
        let header_height = file_row_height(t);
        let line_height = t.code_line_height();
        let mut heights: Vec<f32> = order
            .iter()
            .map(|ix| {
                let hunk = &files[*ix];
                if stack.collapsed.contains(&hunk.path) {
                    return frame;
                }
                let body = stack
                    .heights
                    .get(&hunk.path)
                    .copied()
                    .unwrap_or_else(|| estimated_body_height(vm, hunk, line_height));
                frame + body
            })
            .collect();

        let scroll_y = -f32::from(stack.scroll.offset().y);
        // The file under the top edge keeps its place when files above it turn out taller or shorter than estimated.
        let anchor = stack.plan.as_ref().and_then(|plan| {
            let (block, offset) = plan.layout.locate(scroll_y)?;
            let path = &plan.files[*plan.order.get(block)?].path;
            let block = match order.get(block) {
                Some(ix) if files[*ix].path == *path => block,
                _ => order.iter().position(|ix| files[*ix].path == *path)?,
            };
            // A file that just folded under the top edge has only its header left to keep there.
            if stack.collapsed.contains(path) {
                let folded_before = offset < plan.header_height + DIVIDER;
                return Some((block, if folded_before { offset } else { 0. }));
            }
            // Rows that changed height keep the row under the top edge there, not the distance into the file.
            let rows = (offset - plan.header_height).max(0.) / plan.line_height * line_height;
            Some((block, offset.min(header_height) + rows))
        });
        let viewport = match f32::from(stack.scroll.bounds().size.height) {
            height if height > 0. => height,
            _ => 1200.,
        };

        let mut planned = None;
        for _ in 0..4 {
            let layout = StackLayout::new(&heights);
            let top = match anchor {
                Some((block, offset)) => {
                    layout.top(block) + offset.min((heights[block] - 1.).max(0.))
                }
                None => scroll_y,
            };
            let range = layout.blocks_between(top - OVERDRAW, top + viewport + OVERDRAW);
            let blocks: Vec<StackBlock> = range
                .clone()
                .map(|block| self.stack_block(order[block], &files[order[block]], stack, cx))
                .collect();
            let mut changed = false;
            for (block, built) in range.clone().zip(&blocks) {
                let height = frame + built.body_height;
                if (heights[block] - height).abs() > 0.5 {
                    heights[block] = height;
                    changed = true;
                }
                // A folded file keeps the height it had, for when it opens again out of view.
                if !matches!(built.body, StackBody::Collapsed) {
                    stack
                        .heights
                        .insert(files[built.hunk_ix].path.clone(), built.body_height);
                }
            }
            planned = Some((range.start, blocks, top));
            if !changed {
                break;
            }
        }
        let (first, blocks, top) = planned?;
        if (top - scroll_y).abs() > 0.5 {
            let offset = stack.scroll.offset();
            stack.scroll.set_offset(point(offset.x, px(-top)));
        }
        let built = |path: &String| {
            blocks
                .iter()
                .any(|block| files[block.hunk_ix].path == *path)
        };
        stack.wraps.retain(|path, _| built(path));
        stack.shapes.retain(|path, _| built(path));
        Some(StackPlan {
            layout: StackLayout::new(&heights),
            files,
            order,
            first,
            header_height,
            line_height,
            blocks,
        })
    }

    fn stack_block(
        &self,
        hunk_ix: usize,
        hunk: &DiffHunk,
        stack: &mut DiffStackState,
        cx: &App,
    ) -> StackBlock {
        let message = |glyph, title: String, detail: Option<String>, action| StackBlock {
            hunk_ix,
            body_height: if detail.is_some() {
                DETAILED_MESSAGE_HEIGHT
            } else {
                MESSAGE_HEIGHT
            },
            body: StackBody::Message(StackMessage {
                glyph,
                title: title.into(),
                detail: detail.map(Into::into),
                action,
            }),
        };
        if stack.collapsed.contains(&hunk.path) {
            return StackBlock {
                hunk_ix,
                body_height: 0.,
                body: StackBody::Collapsed,
            };
        }
        if hunk_is_image(hunk) {
            return StackBlock {
                hunk_ix,
                body_height: IMAGE_HEIGHT,
                body: StackBody::Image,
            };
        }
        if is_submodule(hunk) {
            return message(
                glyph::PACKAGE,
                jayjay_i18n::diff::git_submodule(),
                Some(jayjay_i18n::diff::submodule_shows_commit_only()),
                None,
            );
        }
        if is_lfs(hunk) {
            return message(
                glyph::DATABASE,
                jayjay_i18n::diff::git_lfs_backed_file(),
                Some(jayjay_i18n::diff::this_file_is_tracked_through_git()),
                None,
            );
        }
        if hunk.is_content_free_rename_or_copy() {
            let glyph = if hunk.hunk_type == jayjay_core::HunkType::Copied {
                glyph::COPY
            } else {
                glyph::ARROW_CIRCLE_RIGHT
            };
            return message(glyph, jayjay_i18n::diff::no_content_changes(), None, None);
        }
        let vm = self.vm.read(cx);
        let state = vm.diff_load_state(hunk);
        // A failed load leaves the selected file an empty shown diff, which would otherwise read as "no textual diff".
        if matches!(state, DiffLoadState::Failed) {
            return message(
                glyph::WARNING,
                jayjay_i18n::repo::couldnt_load_diff(),
                None,
                Some(StackMessageAction::Retry),
            );
        }
        // The selected file's shown diff carries what was done to it since it loaded, such as expanded context.
        let shown = (vm.selected_file_ix == Some(hunk_ix) && vm.shown.is_for(&hunk.path))
            .then(|| vm.shown.diff().cloned())
            .flatten()
            .map(|diff| (diff, vm.shown.supports_file_editor));
        let loaded = shown.or(match state {
            DiffLoadState::Loaded(loaded) => Some((loaded.diff, loaded.supports_file_editor)),
            DiffLoadState::Failed | DiffLoadState::Missing => None,
        });
        let Some((diff, supports_file_editor)) = loaded else {
            return StackBlock {
                hunk_ix,
                body_height: MESSAGE_HEIGHT,
                body: StackBody::Loading,
            };
        };
        if diff.lines.is_empty() {
            return message(
                glyph::INFO,
                jayjay_i18n::diff::no_textual_diff_binary_identical_or(),
                None,
                None,
            );
        }
        if diff.lines.len() > STACKED_LINE_LIMIT {
            return message(
                glyph::UNFOLD_VERTICAL,
                jayjay_i18n::diff::large_diff_folded(diff.lines.len()),
                None,
                Some(StackMessageAction::ShowDiff {
                    supports_file_editor,
                }),
            );
        }
        let shape = match stack.shapes.get(&hunk.path) {
            Some((scanned, shape)) if Arc::ptr_eq(scanned, &diff) => *shape,
            _ => {
                let shape = DiffShape::of(&diff);
                stack
                    .shapes
                    .insert(hunk.path.clone(), (diff.clone(), shape));
                shape
            }
        };
        let view_mode = vm.view_mode.effective_for(Some(shape));
        let notes = self.notes_for_hunk(hunk, cx);
        let wrap = stack.wraps.entry(hunk.path.clone()).or_default().clone();
        let rows = StackedRows {
            file_diff: &diff,
            view_mode,
            notes: &notes,
            wrap_cache: &wrap,
            unified_bounds: &self.diff.unified_bounds,
            sbs_old_bounds: &self.diff.sbs_old_bounds,
            sbs_new_bounds: &self.diff.sbs_new_bounds,
        };
        let banner_height = rows.banner_height();
        let body_height = banner_height + rows.rows_height(theme(cx), cx);
        StackBlock {
            hunk_ix,
            body_height,
            body: StackBody::Rows {
                diff,
                view_mode,
                notes,
                wrap,
                banner_height,
                supports_file_editor,
            },
        }
    }

    /// Find and scroll-to-line act on the selected file's rows, which the stack does not build for a folded file.
    pub(crate) fn folds_selected_file(&self, cx: &App) -> bool {
        let vm = self.vm.read(cx);
        self.shows_diff_stack()
            && (vm
                .shown
                .diff()
                .is_some_and(|diff| diff.lines.len() > STACKED_LINE_LIMIT)
                || vm
                    .selected_hunk()
                    .is_some_and(|hunk| self.diff.stack.collapsed.contains(&hunk.path)))
    }

    pub(crate) fn stack_row_top(&self, row: usize, cx: &App) -> Option<f32> {
        if self.diff.stack.solo {
            return None;
        }
        let plan = self.diff.stack.plan.as_ref()?;
        let vm = self.vm.read(cx);
        let selected = vm.selected_file_ix?;
        let block = plan.order.iter().position(|ix| *ix == selected)?;
        let banner = plan
            .blocks
            .iter()
            .find(|built| built.hunk_ix == selected)
            .map_or(0., |built| match built.body {
                StackBody::Rows { banner_height, .. } => banner_height,
                _ => 0.,
            });
        let t = theme(cx);
        let rows_top = plan.layout.top(block) + file_row_height(t) + banner;
        Some(rows_top + row as f32 * t.code_line_height())
    }
}

fn estimated_body_height(vm: &RepoViewModel, hunk: &DiffHunk, line_height: f32) -> f32 {
    let lines = vm
        .stats
        .per_file
        .get(&hunk.path)
        .map_or(3, |stats| stats.insertions + stats.deletions + 6);
    lines as f32 * line_height
}
