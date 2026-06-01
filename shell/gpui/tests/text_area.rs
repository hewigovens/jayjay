use gpui::{TestAppContext, VisualContext, VisualTestContext};
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

fn install_text_area_test_bindings(cx: &mut TestAppContext) {
    cx.update(|cx| {
        cx.set_global(Theme::light());
        cx.bind_keys(text_area::key_bindings("cmd"));
    });
}
