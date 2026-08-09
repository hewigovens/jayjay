use std::time::Duration;

use gpui::{AppContext as _, Context, SharedString};
use jayjay_core::diff::{
    DiffLine, DiffSpan, DiffSpanStyle, highlight_file, highlight_file_against_base,
};

use super::{TextArea, TextAreaUpdated};

pub(super) struct SyntaxHighlightState {
    path: String,
    base: Option<SharedString>,
    generation: u64,
    highlighted_content: Option<SharedString>,
    lines: Vec<Vec<DiffSpan>>,
    line_styles: Vec<DiffSpanStyle>,
}

impl SyntaxHighlightState {
    fn new(
        path: impl Into<String>,
        base: Option<SharedString>,
        highlighted_content: Option<SharedString>,
        lines: Vec<Vec<DiffSpan>>,
        line_styles: Vec<DiffSpanStyle>,
    ) -> Self {
        Self {
            path: path.into(),
            base,
            generation: 0,
            highlighted_content,
            lines,
            line_styles,
        }
    }
}

impl TextArea {
    pub fn code_editor(
        content: impl Into<SharedString>,
        path: impl Into<String>,
        placeholder: impl Into<SharedString>,
        height: f32,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut view = Self::new(content, placeholder, true, height, cx);
        view.syntax = Some(SyntaxHighlightState::new(
            path,
            None,
            None,
            Vec::new(),
            Vec::new(),
        ));
        view.schedule_syntax_highlight(false, cx);
        view
    }

    pub(crate) fn prepared_code_editor(
        content: impl Into<SharedString>,
        path: impl Into<String>,
        placeholder: impl Into<SharedString>,
        height: f32,
        highlighted_lines: Vec<Vec<DiffSpan>>,
        cx: &mut Context<Self>,
    ) -> Self {
        let content = content.into();
        let mut view = Self::new(content.clone(), placeholder, true, height, cx);
        view.syntax = Some(SyntaxHighlightState::new(
            path,
            None,
            Some(content),
            highlighted_lines,
            Vec::new(),
        ));
        view
    }

    pub(crate) fn highlighted_code_block(
        content: impl Into<SharedString>,
        path: impl Into<String>,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut view =
            Self::syntax_code_block(content, path, None, None, Vec::new(), Vec::new(), cx);
        view.schedule_syntax_highlight(false, cx);
        view
    }

    pub(crate) fn prepared_highlighted_code_block(
        content: impl Into<SharedString>,
        path: impl Into<String>,
        highlighted_lines: Vec<Vec<DiffSpan>>,
        cx: &mut Context<Self>,
    ) -> Self {
        let content = content.into();
        Self::syntax_code_block(
            content.clone(),
            path,
            None,
            Some(content),
            highlighted_lines,
            Vec::new(),
            cx,
        )
    }

    pub(crate) fn diff_highlighted_code_block(
        content: impl Into<SharedString>,
        path: impl Into<String>,
        base: impl Into<SharedString>,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut view = Self::syntax_code_block(
            content,
            path,
            Some(base.into()),
            None,
            Vec::new(),
            Vec::new(),
            cx,
        );
        view.schedule_syntax_highlight(false, cx);
        view
    }

    pub(crate) fn prepared_diff_highlighted_code_block(
        content: impl Into<SharedString>,
        path: impl Into<String>,
        highlighted_lines: Vec<DiffLine>,
        cx: &mut Context<Self>,
    ) -> Self {
        let content = content.into();
        let (line_styles, lines) = highlighted_lines
            .into_iter()
            .map(|line| (line.style, line.spans))
            .unzip();
        Self::syntax_code_block(
            content.clone(),
            path,
            None,
            Some(content),
            lines,
            line_styles,
            cx,
        )
    }

    fn syntax_code_block(
        content: impl Into<SharedString>,
        path: impl Into<String>,
        base: Option<SharedString>,
        highlighted_content: Option<SharedString>,
        lines: Vec<Vec<DiffSpan>>,
        line_styles: Vec<DiffSpanStyle>,
        cx: &mut Context<Self>,
    ) -> Self {
        let content = content.into();
        let line_count = content.matches('\n').count() + 1;
        let mut view = Self::selectable_code_block(content, line_count, None, cx);
        view.syntax = Some(SyntaxHighlightState::new(
            path,
            base,
            highlighted_content,
            lines,
            line_styles,
        ));
        view
    }

    pub(in crate::ui::text_area) fn syntax_spans(&self, line: usize) -> Option<&[DiffSpan]> {
        self.syntax
            .as_ref()
            .and_then(|syntax| syntax.lines.get(line))
            .map(Vec::as_slice)
    }

    pub(in crate::ui::text_area) fn line_style(&self, line: usize) -> DiffSpanStyle {
        self.syntax
            .as_ref()
            .and_then(|syntax| syntax.line_styles.get(line))
            .copied()
            .unwrap_or(DiffSpanStyle::Context)
    }

    pub fn has_syntax_highlights(&self) -> bool {
        self.syntax.as_ref().is_some_and(|syntax| {
            syntax
                .lines
                .iter()
                .flatten()
                .any(|span| span.token != jayjay_core::diff::syntax::SyntaxToken::Plain)
        })
    }

    pub fn has_diff_highlights(&self) -> bool {
        self.syntax.as_ref().is_some_and(|syntax| {
            syntax
                .line_styles
                .iter()
                .any(|style| matches!(style, DiffSpanStyle::Added | DiffSpanStyle::Removed))
        })
    }

    pub fn has_current_syntax_highlights(&self) -> bool {
        self.syntax
            .as_ref()
            .is_some_and(|syntax| syntax.highlighted_content.as_ref() == Some(&self.content))
    }

    pub(super) fn schedule_syntax_highlight(&mut self, debounce: bool, cx: &mut Context<Self>) {
        let Some(syntax) = self.syntax.as_mut() else {
            return;
        };
        syntax.generation = syntax.generation.wrapping_add(1);
        let generation = syntax.generation;
        cx.spawn(async move |this, cx| {
            if debounce {
                cx.background_executor()
                    .timer(Duration::from_millis(120))
                    .await;
            }
            let request = this
                .update(cx, |input, _| {
                    let syntax = input.syntax.as_ref()?;
                    (syntax.generation == generation).then(|| {
                        (
                            syntax.path.clone(),
                            syntax.base.clone(),
                            input.content.clone(),
                        )
                    })
                })
                .ok()
                .flatten();
            let Some((path, base, content)) = request else {
                return;
            };
            let highlighted_content = content.clone();
            let (lines, line_styles) = cx
                .background_spawn(async move {
                    if let Some(base) = base {
                        highlight_file_against_base(&path, base.as_ref(), content.as_ref())
                            .into_iter()
                            .map(|line| (line.spans, line.style))
                            .unzip()
                    } else {
                        (highlight_file(&path, content.as_ref()), Vec::new())
                    }
                })
                .await;
            let _ = this.update(cx, move |input, cx| {
                let Some(syntax) = input.syntax.as_mut() else {
                    return;
                };
                if syntax.generation != generation {
                    return;
                }
                syntax.highlighted_content = Some(highlighted_content);
                syntax.lines = lines;
                syntax.line_styles = line_styles;
                input.last_layout = None;
                cx.emit(TextAreaUpdated);
                cx.notify();
            });
        })
        .detach();
    }
}
