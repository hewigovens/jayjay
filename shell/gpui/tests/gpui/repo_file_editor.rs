use std::fs;

use crate::harness::{open_fixture, select_file, settle_visual};
use gpui::{Modifiers, TestAppContext, px};
use jj_test::LinearFixture;

#[gpui::test]
fn working_copy_file_edits_and_saves_inside_the_repository_window(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let path = fixture.path.clone();
    fs::write(
        path.join("notes.md"),
        format!(
            "# JayJay notes\n\n- prepared highlighting\n\n{}\n{}",
            "soft-wrapped editor text ".repeat(80),
            (0..80)
                .map(|line| format!("tail line {line}\n"))
                .collect::<String>()
        ),
    )
    .expect("write markdown working-copy file");
    let (view, cx) = open_fixture(&fixture, cx);
    select_file(&view, "notes.md", cx);

    let edit = cx
        .debug_bounds("edit-working-copy-file")
        .expect("Edit File button");
    cx.simulate_click(edit.center(), Modifiers::default());
    settle_visual(cx);

    assert!(view.read_with(cx, |view, _| view.file_editor_active()));
    assert!(view.read_with(cx, |view, cx| view.file_editor_has_syntax_highlights(cx)));
    assert_eq!(
        view.read_with(cx, |view, cx| view.file_editor_scroll_offset_y(cx)),
        px(0.),
        "a newly opened file editor should start at the beginning"
    );

    fs::write(path.join("while-editing.txt"), "external edit\n")
        .expect("write external working-copy file");
    view.update_in(cx, |view, _, cx| view.handle_fs_event(cx));
    assert!(
        !view.read_with(cx, |view, cx| view.view_model().read(cx).loading.refreshing),
        "background refresh should wait while the file editor owns a draft"
    );

    view.update_in(cx, |view, _, cx| {
        view.set_file_editor_content("edited in JayJay\n".to_owned(), cx);
    });
    settle_visual(cx);
    let save = cx.debug_bounds("file-editor-save").expect("Save button");
    cx.simulate_click(save.center(), Modifiers::default());
    settle_visual(cx);

    assert!(!view.read_with(cx, |view, _| view.file_editor_active()));
    assert_eq!(
        fs::read_to_string(path.join("notes.md")).expect("read edited file"),
        "edited in JayJay\n"
    );
    assert!(view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .files
            .as_ref()
            .is_some_and(|files| files.iter().any(|file| file.path == "while-editing.txt"))
    }));
}

#[gpui::test]
fn binary_working_copy_file_does_not_offer_the_editor(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    fs::write(fixture.path.join("binary.dat"), b"prefix\0suffix").expect("write binary file");
    let (view, cx) = open_fixture(&fixture, cx);
    select_file(&view, "binary.dat", cx);

    assert!(
        cx.debug_bounds("edit-working-copy-file").is_none(),
        "binary files must not expose an editor action"
    );
}

#[gpui::test]
fn placeholder_prefixed_working_copy_text_offers_the_editor(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    fs::write(
        fixture.path.join("literal.txt"),
        "symlink -> literal text\n",
    )
    .expect("write placeholder-prefixed text");
    let (view, cx) = open_fixture(&fixture, cx);
    select_file(&view, "literal.txt", cx);

    assert!(cx.debug_bounds("edit-working-copy-file").is_some());
}
