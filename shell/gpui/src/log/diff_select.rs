use gpui::Context;

use super::LogView;
use crate::diff::DiffSelection;

impl LogView {
    pub fn start_diff_selection(&mut self, line_ix: usize, cx: &mut Context<Self>) {
        self.diff_selection = Some(DiffSelection::start(line_ix));
        cx.notify();
    }

    pub fn extend_diff_selection(&mut self, line_ix: usize, cx: &mut Context<Self>) {
        let Some(sel) = self.diff_selection.as_mut() else {
            return;
        };
        if !sel.dragging || sel.focus == line_ix {
            return;
        }
        sel.extend(line_ix);
        cx.notify();
    }

    pub fn finish_diff_selection(&mut self, cx: &mut Context<Self>) {
        let Some(sel) = self.diff_selection.as_mut() else {
            return;
        };
        sel.dragging = false;
        cx.notify();
    }

    pub fn copy_diff_selection(&mut self, cx: &mut Context<Self>) {
        let Some(sel) = self.diff_selection else {
            return;
        };
        let lines = self.vm.read(cx).current_diff.as_ref().map(|fd| {
            sel.range()
                .filter_map(|ix| fd.lines.get(ix))
                .map(|l| l.spans.iter().map(|s| s.text.as_str()).collect::<String>())
                .collect::<Vec<_>>()
        });
        let Some(lines) = lines else {
            return;
        };
        if lines.is_empty() {
            return;
        }
        let text = lines.join("\n");
        cx.write_to_clipboard(gpui::ClipboardItem::new_string(text));
    }
}
