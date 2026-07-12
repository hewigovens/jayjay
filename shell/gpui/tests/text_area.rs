use gpui::{EntityInputHandler, TestAppContext, VisualContext, VisualTestContext};
use jayjay_gpui::app::theme::Theme;
use jayjay_gpui::ui::text_area::{self, TextArea};

#[gpui::test]
fn text_area_supports_readline_style_line_editing(cx: &mut TestAppContext) {
    install_text_area_test_bindings(cx);
    let (input, cx) = cx.add_window_view(|_, cx| TextArea::new("", "Message", true, 80., cx));
    let cx: &mut VisualTestContext = cx;

    cx.focus(&input);
    cx.simulate_input("alpha\nbeta");
    cx.simulate_keystrokes("ctrl-a");
    cx.simulate_input(">");
    cx.simulate_keystrokes("ctrl-e");
    cx.simulate_input("<");
    cx.simulate_keystrokes("cmd-backspace");

    input.read_with(cx, |input, _| {
        assert_eq!(input.text(), "alpha\n");
    });
}

#[gpui::test]
fn text_area_supports_alt_word_navigation(cx: &mut TestAppContext) {
    install_text_area_test_bindings(cx);
    let (input, cx) = cx.add_window_view(|_, cx| TextArea::new("", "Message", true, 80., cx));
    let cx: &mut VisualTestContext = cx;

    cx.focus(&input);
    cx.simulate_input("alpha beta_gamma, delta");
    cx.simulate_keystrokes("home alt-right");
    cx.simulate_input("|");

    input.read_with(cx, |input, _| {
        assert_eq!(input.text(), "alpha| beta_gamma, delta");
    });

    input.update_in(cx, |input, _, cx| {
        input.set_text("alpha beta_gamma, delta", cx);
    });
    cx.simulate_keystrokes("home alt-right alt-right");
    cx.simulate_input("|");

    input.read_with(cx, |input, _| {
        assert_eq!(input.text(), "alpha beta_gamma|, delta");
    });

    input.update_in(cx, |input, _, cx| {
        input.set_text("alpha beta_gamma, delta", cx);
    });
    cx.simulate_keystrokes("alt-left");
    cx.simulate_input("|");

    input.read_with(cx, |input, _| {
        assert_eq!(input.text(), "alpha beta_gamma, |delta");
    });

    input.update_in(cx, |input, _, cx| {
        input.set_text("alpha beta_gamma, delta", cx);
    });
    cx.simulate_keystrokes("alt-left alt-left");
    cx.simulate_input("|");

    input.read_with(cx, |input, _| {
        assert_eq!(input.text(), "alpha |beta_gamma, delta");
    });
}

#[gpui::test]
fn text_area_supports_alt_delete_previous_word(cx: &mut TestAppContext) {
    install_text_area_test_bindings(cx);
    let (input, cx) = cx.add_window_view(|_, cx| TextArea::new("", "Message", true, 80., cx));
    let cx: &mut VisualTestContext = cx;

    cx.focus(&input);
    cx.simulate_input("alpha beta gamma");
    cx.simulate_keystrokes("alt-backspace");

    input.read_with(cx, |input, _| {
        assert_eq!(input.text(), "alpha beta ");
    });

    cx.simulate_keystrokes("alt-delete");

    input.read_with(cx, |input, _| {
        assert_eq!(input.text(), "alpha ");
    });
}

#[gpui::test]
fn text_area_supports_vertical_navigation(cx: &mut TestAppContext) {
    install_text_area_test_bindings(cx);
    let (input, cx) = cx.add_window_view(|_, cx| TextArea::new("", "Message", true, 80., cx));
    let cx: &mut VisualTestContext = cx;

    cx.focus(&input);
    cx.simulate_input("abcd\nefgh\nij");
    cx.simulate_keystrokes("up");
    cx.simulate_input("|");

    input.read_with(cx, |input, _| {
        assert_eq!(input.text(), "abcd\nef|gh\nij");
    });

    input.update_in(cx, |input, _, cx| {
        input.set_text("abcd\nefgh\nij", cx);
    });
    cx.simulate_keystrokes("up down");
    cx.simulate_input("|");

    input.read_with(cx, |input, _| {
        assert_eq!(input.text(), "abcd\nefgh\nij|");
    });
}

#[gpui::test]
fn text_area_supports_readline_delete_and_shift_enter(cx: &mut TestAppContext) {
    install_text_area_test_bindings(cx);
    let (input, cx) = cx.add_window_view(|_, cx| TextArea::new("", "Message", true, 80., cx));
    let cx: &mut VisualTestContext = cx;

    cx.focus(&input);
    cx.simulate_input("alpha");
    cx.simulate_keystrokes("shift-enter");
    cx.simulate_input("beta");

    input.read_with(cx, |input, _| {
        assert_eq!(input.text(), "alpha\nbeta");
    });

    cx.simulate_keystrokes("ctrl-u");

    input.read_with(cx, |input, _| {
        assert_eq!(input.text(), "alpha\n");
    });

    input.update_in(cx, |input, _, cx| {
        input.set_text("alpha beta\ngamma", cx);
    });
    cx.simulate_keystrokes("ctrl-a ctrl-k");

    input.read_with(cx, |input, _| {
        assert_eq!(input.text(), "alpha beta\n");
    });
}

#[gpui::test]
fn text_area_ime_marked_selection_after_multibyte_text_does_not_panic(cx: &mut TestAppContext) {
    install_text_area_test_bindings(cx);
    let (input, cx) = cx.add_window_view(|_, cx| TextArea::new("", "Message", true, 80., cx));
    let cx: &mut VisualTestContext = cx;

    cx.focus(&input);
    cx.simulate_input("é"); // pre-existing non-ASCII text (2 UTF-8 bytes) before composition

    // AppKit's marked-text selection is UTF-16 and relative to the marked run; mapping it against
    // the whole content used to land the end offset mid-character (inside the first あ).
    input.update_in(cx, |input, window, cx| {
        input.replace_and_mark_text_in_range(None, "ああ", Some(0..1), window, cx);
    });

    // Copy byte-slices the selection; a non-char-boundary range panics.
    cx.simulate_keystrokes("cmd-c");

    cx.read_from_clipboard()
        .and_then(|item| item.text())
        .map(|text| assert_eq!(text, "あ"))
        .expect("copy should place the first composed glyph on the clipboard");
    input.read_with(cx, |input, _| {
        assert_eq!(input.text(), "éああ");
    });
}

fn install_text_area_test_bindings(cx: &mut TestAppContext) {
    cx.update(|cx| {
        cx.set_global(Theme::light());
        cx.bind_keys(text_area::key_bindings("cmd"));
    });
}

#[gpui::test]
fn long_content_scrolls_caret_into_view_and_wheel_scrolls(cx: &mut TestAppContext) {
    use gpui::{ScrollDelta, ScrollWheelEvent, TouchPhase, point, px, size};

    install_text_area_test_bindings(cx);
    let (input, cx) = cx.add_window_view(|_, cx| TextArea::new("", "Message", true, 80., cx));
    let cx: &mut VisualTestContext = cx;
    cx.simulate_resize(size(px(300.), px(120.)));

    cx.focus(&input);
    // 10 lines at 18px in a 68px viewport (80 minus 12 vertical padding).
    cx.simulate_input("l0\nl1\nl2\nl3\nl4\nl5\nl6\nl7\nl8\nl9");
    cx.run_until_parked();
    input.read_with(cx, |input, _| {
        assert_eq!(
            input.scroll_offset_y(),
            px(112.),
            "caret at the end must scroll the tail into view"
        );
    });

    let wheel = |cx: &mut VisualTestContext, dy: f32| {
        cx.simulate_event(ScrollWheelEvent {
            position: point(px(50.), px(40.)),
            delta: ScrollDelta::Pixels(point(px(0.), px(dy))),
            modifiers: Default::default(),
            touch_phase: TouchPhase::Moved,
        });
        cx.run_until_parked();
    };

    wheel(cx, 40.);
    input.read_with(cx, |input, _| {
        assert_eq!(input.scroll_offset_y(), px(72.), "wheel up scrolls back");
    });
    wheel(cx, 1000.);
    input.read_with(cx, |input, _| {
        assert_eq!(input.scroll_offset_y(), px(0.), "clamped at the top");
    });
    wheel(cx, -1000.);
    input.read_with(cx, |input, _| {
        assert_eq!(input.scroll_offset_y(), px(112.), "clamped at the bottom");
    });

    cx.simulate_keystrokes("cmd-up");
    cx.run_until_parked();
    input.read_with(cx, |input, _| {
        assert_eq!(
            input.scroll_offset_y(),
            px(0.),
            "moving the caret to the document start must scroll it into view"
        );
    });
}
