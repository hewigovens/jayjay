use super::view::{FlockView, Sheet};
use gpui::{AppContext, AsyncApp, Context, WeakEntity};
use jayjay_core::flock::{FlockSignIn, FlockStore, FlockVerification};
use jayjay_core::{JayError, JayResult, SyncToken};
use jayjay_i18n::settings;

const SPONSORABLE: &str = "hewigovens";

pub(super) struct Activation {
    sync: SyncToken,
    pub(super) busy: bool,
    pub(super) sign_in: Option<(String, String)>,
    pub(super) message: Option<String>,
    pub(super) error: Option<String>,
}

impl Drop for Activation {
    fn drop(&mut self) {
        self.sync.cancel();
    }
}

impl FlockView {
    pub(super) fn open_activation(&mut self, cx: &mut Context<Self>) {
        self.sheet = Some(Sheet::Activation(Activation {
            sync: SyncToken::standalone(),
            busy: false,
            sign_in: None,
            message: None,
            error: None,
        }));
        self.verify(false, cx);
    }

    fn activation(&mut self, sync: Option<&SyncToken>) -> Option<&mut Activation> {
        match &mut self.sheet {
            Some(Sheet::Activation(activation))
                if sync.is_none_or(|sync| activation.sync.is(sync)) =>
            {
                Some(activation)
            }
            _ => None,
        }
    }

    pub(super) fn verify(&mut self, another_account: bool, cx: &mut Context<Self>) {
        let Some(activation) = self.activation(None) else {
            return;
        };
        if activation.busy {
            return;
        }
        activation.sync.cancel();
        activation.sync = SyncToken::standalone();
        activation.busy = true;
        activation.sign_in = None;
        activation.message = None;
        activation.error = None;
        let sync = activation.sync.clone();
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result = Self::check_sponsorship(another_account, &sync, &this, cx).await;
            let _ = this.update(cx, |view, cx| view.finish_verification(&sync, result, cx));
        })
        .detach();
    }

    async fn check_sponsorship(
        another_account: bool,
        sync: &SyncToken,
        this: &WeakEntity<Self>,
        cx: &mut AsyncApp,
    ) -> JayResult<FlockVerification> {
        if !another_account {
            let check_sync = sync.clone();
            let result = cx
                .background_spawn(
                    async move { FlockStore::load().verify(SPONSORABLE, &check_sync) },
                )
                .await?;
            if !matches!(result, FlockVerification::SignInRequired) {
                return Ok(result);
            }
        }
        let sign_in = cx
            .background_spawn(async { FlockSignIn::start(SPONSORABLE.into()) })
            .await?;
        let code = (
            sign_in.user_code().to_owned(),
            sign_in.verification_uri().to_owned(),
        );
        this.update(cx, |view, cx| {
            let activation = view.activation(Some(sync)).ok_or(JayError::Canceled)?;
            activation.sign_in = Some(code);
            cx.notify();
            Ok(())
        })
        .map_err(|_| JayError::Canceled)??;
        let wait_sync = sync.clone();
        cx.background_spawn(async move { sign_in.wait(&wait_sync) })
            .await
    }

    fn finish_verification(
        &mut self,
        sync: &SyncToken,
        result: JayResult<FlockVerification>,
        cx: &mut Context<Self>,
    ) {
        // Core has already saved an unlock, so it applies even after the sheet closed.
        if let Ok(FlockVerification::Unlocked { member }) = result {
            self.member = Some(member);
            if self.activation(Some(sync)).is_some() {
                self.sheet = None;
            }
            cx.notify();
            return;
        }
        let Some(activation) = self.activation(Some(sync)) else {
            return;
        };
        activation.busy = false;
        activation.sign_in = None;
        match result {
            Ok(FlockVerification::NotSponsored { login }) => {
                activation.message = Some(settings::flock_not_sponsored(&login))
            }
            Ok(FlockVerification::SignInRequired) => {
                activation.error = Some(settings::flock_sign_in_failed())
            }
            Err(JayError::Canceled) | Ok(FlockVerification::Unlocked { .. }) => {}
            Err(error) => activation.error = Some(error.to_string()),
        }
        cx.notify();
    }

    pub(super) fn redeem(&mut self, cx: &mut Context<Self>) {
        if self.busy || !self.code_ready(cx) {
            return;
        }
        let code = self.code.read(cx).text();
        let name = self.name.read(cx).text();
        self.busy = true;
        self.message = None;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { FlockStore::load().redeem(&code, &name) })
                .await;
            let _ = this.update(cx, |view, cx| {
                view.busy = false;
                match result {
                    Ok(Some(member)) => {
                        view.member = Some(member);
                        view.close_sheet(cx);
                    }
                    Ok(None) => view.message = Some(settings::flock_code_invalid()),
                    Err(error) => view.message = Some(error.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(super) fn forget(&mut self, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        self.busy = true;
        self.message = None;
        cx.notify();
        cx.spawn(async |this, cx| {
            let result = cx
                .background_spawn(async { FlockStore::load().forget() })
                .await;
            let _ = this.update(cx, |view, cx| {
                view.busy = false;
                match result {
                    Ok(()) => view.member = None,
                    Err(error) => view.message = Some(error.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }
}
