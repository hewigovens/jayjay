use gpui::{
    App, AppContext, Bounds, Point, Size, TitlebarOptions, WindowBounds, WindowOptions, px,
};

use crate::app::theme::{Theme, observe_window_appearance};

use super::ExternalToolInvocation;
use super::view::ExternalToolWindow;

pub fn open_external_tool(invocation: ExternalToolInvocation, cx: &mut App) -> Result<(), String> {
    let title = invocation.title();
    let bounds = Bounds::centered(
        None,
        Size {
            width: px(1120.),
            height: px(760.),
        },
        cx,
    );
    let handle = cx
        .open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some(title.into()),
                    appears_transparent: true,
                    traffic_light_position: Some(Point {
                        x: px(12.),
                        y: px(14.),
                    }),
                }),
                ..crate::app::window_options()
            },
            move |_, cx| {
                cx.new(|cx| {
                    cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
                    cx.observe_global::<crate::app::config::AppConfigStore>(|_, cx| cx.notify())
                        .detach();
                    ExternalToolWindow::new(invocation.clone(), cx)
                })
            },
        )
        .map_err(|error| error.to_string())?;
    handle
        .update(cx, |view, window, cx| {
            observe_window_appearance(window, cx);
            window.focus(&view.focus_handle, cx);
            let exit_state = view.exit_state.clone();
            let cancel_exit_code = view.invocation.cancel_exit_code();
            let exit = view.exit.clone();
            window.on_window_should_close(cx, move |_, _| {
                exit(exit_state.code(cancel_exit_code));
                false
            });
        })
        .map_err(|error| error.to_string())?;
    Ok(())
}
