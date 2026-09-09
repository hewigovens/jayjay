use std::time::Duration;

use crate::harness::{install_test_globals, settle_visual};
use gpui::{Modifiers, TestAppContext, VisualTestContext, px, size};
use jayjay_core::repositories::normalize_repository_path;
use jayjay_gpui::app::config;
use jayjay_gpui::windows::repo_list::RepoListWindow;

#[gpui::test]
fn recent_panel_toggles_and_keeps_the_pinned_column_at_welcome_width(cx: &mut TestAppContext) {
    let pinned = tempfile::tempdir().expect("pinned repository");
    let recent = tempfile::tempdir().expect("recent repository");
    let path = |dir: &tempfile::TempDir| {
        normalize_repository_path(dir.path())
            .to_string_lossy()
            .into_owned()
    };
    install_test_globals(cx);
    cx.update(RepoListWindow::open);
    let window = cx.windows().last().copied().expect("repo list window");
    let mut visual = VisualTestContext::from_window(window, cx);
    settle_visual(&mut visual);
    assert!(
        visual.debug_bounds("repo-list-recent-panel").is_none(),
        "nothing recent, so the panel starts hidden"
    );
    assert!(
        visual.debug_bounds("repo-list-toggle-recent").is_some(),
        "the toggle is there even with nothing listed"
    );

    visual.cx.update(|cx| {
        config::update(cx, |cfg| {
            cfg.recent_repos = vec![path(&pinned), path(&recent)]
        })
    });
    settle_slide(&mut visual);
    let hero = visual
        .debug_bounds("repo-list-detail")
        .expect("detail column");
    let height = visual.update(|window, _| window.viewport_size().height);
    assert!(
        (hero.center().y - (px(38.) + height) / 2.).abs() < px(2.),
        "with nothing pinned the hero centers below the toolbar: {hero:?}"
    );
    let pin = visual.debug_bounds("repo-list-pin-0").expect("pin button");
    visual.simulate_click(pin.center(), Modifiers::default());
    settle_visual(&mut visual);

    let panel = visual
        .debug_bounds("repo-list-recent-panel")
        .expect("recent panel shows by default");
    let detail = visual
        .debug_bounds("repo-list-detail")
        .expect("detail column");
    assert!(
        visual.debug_bounds("repo-list-row-0").is_some(),
        "recent row lives in the panel"
    );
    assert!(detail.origin.x >= panel.origin.x + panel.size.width);
    assert!(detail.size.width <= px(480.), "{detail:?}");

    let toggle = visual
        .debug_bounds("repo-list-toggle-recent")
        .expect("panel toggle");
    assert!(
        toggle.origin.x < panel.origin.x + px(100.),
        "toggle leads the window like the sidebar button: {toggle:?}"
    );
    visual.simulate_click(toggle.center(), Modifiers::default());
    settle_visual(&mut visual);
    visual
        .cx
        .executor()
        .advance_clock(Duration::from_millis(90));
    visual.update(|window, cx| window.simulate_next_frame(cx));
    settle_visual(&mut visual);
    let sliding = visual
        .debug_bounds("repo-list-recent-panel")
        .expect("the panel slides out before it leaves the tree");
    assert!(
        sliding.origin.x < px(0.) && sliding.origin.x > px(-270.),
        "mid-slide the panel is partly off the left edge: {sliding:?}"
    );
    settle_slide(&mut visual);
    assert!(
        visual.debug_bounds("repo-list-recent-panel").is_none(),
        "toggle hides the panel"
    );
    assert!(
        !visual
            .cx
            .update(|cx| config::current(cx).layout.recent_repos_panel)
    );
    let pinned_row = visual
        .debug_bounds("repo-list-pinned-row-0")
        .expect("pinned row stays");
    assert!(
        pinned_row.size.width <= px(480.),
        "pinned cards keep the welcome width: {pinned_row:?}"
    );
    let window_width = visual.update(|window, _| window.viewport_size().width);
    assert!(
        (pinned_row.center().x - window_width / 2.).abs() < px(2.),
        "pinned column is centered once the panel is hidden"
    );

    let toggle = visual
        .debug_bounds("repo-list-toggle-recent")
        .expect("panel toggle");
    visual.simulate_click(toggle.center(), Modifiers::default());
    settle_slide(&mut visual);
    assert!(
        visual.debug_bounds("repo-list-recent-panel").is_some(),
        "toggle shows the panel again"
    );
    assert!(
        visual
            .cx
            .update(|cx| config::current(cx).layout.recent_repos_panel)
    );

    visual.simulate_resize(size(px(600.), px(600.)));
    settle_visual(&mut visual);
    let panel = visual
        .debug_bounds("repo-list-recent-panel")
        .expect("a narrow window keeps the panel reachable");
    let detail = visual
        .debug_bounds("repo-list-detail")
        .expect("detail column");
    assert!(detail.origin.x >= panel.origin.x + panel.size.width);
    assert!(
        detail.size.width < px(480.),
        "detail compresses: {detail:?}"
    );

    let clear = visual
        .debug_bounds("repo-list-clear")
        .expect("clear button");
    visual.simulate_click(clear.center(), Modifiers::default());
    settle_slide(&mut visual);
    assert!(
        visual.debug_bounds("repo-list-recent-panel").is_none(),
        "clearing the last recent hides the panel despite the earlier choice"
    );
    visual
        .cx
        .update(|cx| config::update(cx, |cfg| cfg.record_opened_repo(recent.path())));
    settle_slide(&mut visual);
    assert!(
        visual.debug_bounds("repo-list-recent-panel").is_some(),
        "the first recent shows the panel again"
    );
}

fn settle_slide(visual: &mut VisualTestContext) {
    settle_visual(visual);
    visual
        .cx
        .executor()
        .advance_clock(Duration::from_millis(200));
    visual.update(|window, cx| window.simulate_next_frame(cx));
    settle_visual(visual);
}
