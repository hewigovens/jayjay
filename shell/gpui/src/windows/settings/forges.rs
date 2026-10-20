use gpui::{
    AnyElement, AppContext as _, ClipboardItem, Context, InteractiveElement, IntoElement,
    ParentElement, SharedString, StatefulInteractiveElement, Styled, div, px, rgb,
};
use jayjay_core::{HostLogin, JayError, RepoHost, SyncToken};

use super::SettingsView;
use super::shared::form_section;
use super::tools::{ToolState, status_row};
use crate::app::theme::{Theme, ui_font_size};
use crate::ui::icons::glyph;
use crate::ui::primitives::button;

pub(super) struct ForgeSignIn {
    host: RepoHost,
    /// User code and verification page.
    code: Option<(String, String)>,
    error: Option<String>,
    sync: SyncToken,
}

pub(super) fn forge_section(
    view: &SettingsView,
    t: &Theme,
    cx: &mut Context<SettingsView>,
) -> AnyElement {
    let diagnostics = view.cli_diagnostics.as_ref();
    let mut rows = Vec::new();
    for host in RepoHost::TOKEN_HOSTS {
        let login = diagnostics.and_then(|d| d.forge_logins.iter().find(|l| l.host == host));
        rows.push(login_row(host, login, view.forge_sign_in.is_some(), t, cx));
        if let Some(sign_in) = view.forge_sign_in.as_ref().filter(|s| s.host == host) {
            rows.push(sign_in_row(sign_in, t, cx));
        }
    }
    form_section("Forges", rows, t).into_any_element()
}

fn login_row(
    host: RepoHost,
    login: Option<&HostLogin>,
    signing_in: bool,
    t: &Theme,
    cx: &mut Context<SettingsView>,
) -> AnyElement {
    let (detail, state) = match login.map(|login| login.source) {
        Some(Some(source)) => (source.label(), ToolState::Found),
        Some(None) => ("Not signed in".to_owned(), ToolState::Missing),
        None => ("Checking…".to_owned(), ToolState::Checking),
    };
    let glyph_str = match host {
        RepoHost::GitHub => glyph::MARK_GITHUB,
        _ => glyph::GIT_MERGE,
    };
    let row = status_row(host.display_name(), glyph_str, detail, state, t);
    let signed_in = login.is_some_and(|login| login.signed_in);
    let can_sign_in = login.is_some_and(|login| login.can_sign_in) && !signing_in;
    let id = SharedString::from(format!("settings-forge-{}-action", host.display_name()));
    let selector = id.to_string();
    let action = if signed_in {
        button(id, "Sign Out", t, false)
            .on_click(cx.listener(move |view, _, _, cx| view.sign_out_of_forge(host, cx)))
    } else if can_sign_in {
        button(id, "Sign In…", t, false)
            .on_click(cx.listener(move |view, _, _, cx| view.start_forge_sign_in(host, cx)))
    } else {
        return row.into_any_element();
    };
    row.child(action.debug_selector(move || selector.clone()))
        .into_any_element()
}

fn sign_in_row(sign_in: &ForgeSignIn, t: &Theme, cx: &mut Context<SettingsView>) -> AnyElement {
    let message: SharedString = match (&sign_in.error, &sign_in.code) {
        (Some(error), _) => error.clone().into(),
        (None, Some((code, uri))) => format!("Enter {code} at {uri}").into(),
        (None, None) => "Requesting a sign-in code…".into(),
    };
    let mut row = div()
        .debug_selector(|| "settings-forge-sign-in".to_owned())
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.))
        .pl(px(28.))
        .py(px(4.))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .text_size(ui_font_size(11.))
                .text_color(rgb(if sign_in.error.is_some() {
                    t.error_fg
                } else {
                    t.fg_dim
                }))
                .child(message),
        );
    if sign_in.code.is_some() && sign_in.error.is_none() {
        row = row.child(
            button(
                "settings-forge-sign-in-open",
                "Copy Code & Open Browser",
                t,
                true,
            )
            .on_click(cx.listener(|view, _, _, cx| view.open_forge_sign_in(cx))),
        );
    }
    let close = if sign_in.error.is_some() {
        "Close"
    } else {
        "Cancel"
    };
    row.child(
        button("settings-forge-sign-in-cancel", close, t, false)
            .on_click(cx.listener(|view, _, _, cx| view.cancel_forge_sign_in(cx))),
    )
    .into_any_element()
}

impl SettingsView {
    fn start_forge_sign_in(&mut self, host: RepoHost, cx: &mut Context<Self>) {
        let sync = SyncToken::standalone();
        self.forge_sign_in = Some(ForgeSignIn {
            host,
            code: None,
            error: None,
            sync: sync.clone(),
        });
        cx.notify();
        cx.spawn(async move |this, cx| {
            let started = cx
                .background_spawn(async move { host.start_sign_in() })
                .await;
            let sign_in = match started {
                Ok(sign_in) => sign_in,
                Err(error) => {
                    let _ = this.update(cx, |view, cx| view.fail_forge_sign_in(&sync, error, cx));
                    return;
                }
            };
            let code = (sign_in.user_code.clone(), sign_in.verification_uri.clone());
            let _ = this.update(cx, |view, cx| {
                if let Some(current) = view.current_sign_in(&sync) {
                    current.code = Some(code);
                    cx.notify();
                }
            });
            let wait_sync = sync.clone();
            let finished = cx
                .background_spawn(async move { sign_in.wait(&wait_sync) })
                .await;
            let _ = this.update(cx, |view, cx| match finished {
                Ok(()) => {
                    if view.current_sign_in(&sync).is_some() {
                        view.forge_sign_in = None;
                    }
                    view.reload_forge_logins(cx);
                }
                Err(error) => view.fail_forge_sign_in(&sync, error, cx),
            });
        })
        .detach();
    }

    /// None once that sign-in was canceled or replaced.
    fn current_sign_in(&mut self, sync: &SyncToken) -> Option<&mut ForgeSignIn> {
        self.forge_sign_in
            .as_mut()
            .filter(|sign_in| sign_in.sync.is(sync))
    }

    fn fail_forge_sign_in(&mut self, sync: &SyncToken, error: JayError, cx: &mut Context<Self>) {
        if matches!(error, JayError::Canceled) {
            return;
        }
        if let Some(current) = self.current_sign_in(sync) {
            current.error = Some(error.to_string());
            cx.notify();
        }
    }

    fn open_forge_sign_in(&mut self, cx: &mut Context<Self>) {
        let Some((code, uri)) = self.forge_sign_in.as_ref().and_then(|s| s.code.clone()) else {
            return;
        };
        cx.write_to_clipboard(ClipboardItem::new_string(code));
        crate::platform::open_url(&uri);
    }

    fn cancel_forge_sign_in(&mut self, cx: &mut Context<Self>) {
        if let Some(sign_in) = self.forge_sign_in.take() {
            sign_in.sync.cancel();
        }
        cx.notify();
    }

    fn sign_out_of_forge(&mut self, host: RepoHost, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            let result = cx.background_spawn(async move { host.sign_out() }).await;
            let _ = this.update(cx, |view, cx| {
                if let Err(error) = result {
                    view.forge_sign_in = Some(ForgeSignIn {
                        host,
                        code: None,
                        error: Some(error.to_string()),
                        sync: SyncToken::standalone(),
                    });
                }
                view.reload_forge_logins(cx);
            });
        })
        .detach();
    }

    fn reload_forge_logins(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            let logins = cx
                .background_spawn(async { RepoHost::TOKEN_HOSTS.map(RepoHost::login).to_vec() })
                .await;
            let _ = this.update(cx, |view, cx| {
                if let Some(diagnostics) = view.cli_diagnostics.as_mut() {
                    diagnostics.forge_logins = logins;
                }
                cx.notify();
            });
        })
        .detach();
    }
}
