use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

use keyring_core::{CredentialStore, Entry, Result};

static SYSTEM: LazyLock<Result<SignInStore>> = LazyLock::new(|| {
    if cfg!(feature = "mock") {
        Ok(SignInStore {
            store: keyring_core::mock::Store::new()?,
            modifiers: None,
        })
    } else {
        SignInStore::platform()
    }
});

pub(super) struct SignInStore {
    store: Arc<CredentialStore>,
    modifiers: Option<HashMap<&'static str, &'static str>>,
}

impl SignInStore {
    pub(super) fn system() -> &'static Result<Self> {
        &SYSTEM
    }

    pub(super) fn entry(&self, service: &str, account: &str) -> Result<Entry> {
        self.store.build(service, account, self.modifiers.as_ref())
    }

    #[cfg(target_os = "macos")]
    fn platform() -> Result<Self> {
        use apple_native_keyring_store::{keychain, protected};

        // Scoped to team and bundle ID, so a differently signed build never prompts.
        let protected = Self {
            store: protected::Store::new()?,
            modifiers: Some(HashMap::from([(
                "access-policy",
                "after-first-unlock-this-device-only",
            )])),
        };
        // Without the entitlement reads just find nothing, so only a write tells.
        let probe = protected.entry("JayJay entitlement probe", "probe")?;
        match probe.set_secret(b"probe") {
            Err(keyring_core::Error::PlatformFailure(error)) if is_missing_entitlement(&*error) => {
                Ok(Self {
                    store: keychain::Store::new()?,
                    modifiers: None,
                })
            }
            _ => {
                let _ = probe.delete_credential();
                Ok(protected)
            }
        }
    }

    #[cfg(windows)]
    fn platform() -> Result<Self> {
        Ok(Self {
            store: windows_native_keyring_store::Store::new()?,
            modifiers: None,
        })
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    fn platform() -> Result<Self> {
        Ok(Self {
            store: zbus_secret_service_keyring_store::Store::new()?,
            modifiers: None,
        })
    }
}

/// Ad-hoc builds (Debug, tests, the CLI) lack the application identifier.
#[cfg(target_os = "macos")]
fn is_missing_entitlement(error: &(dyn std::error::Error + Send + Sync + 'static)) -> bool {
    const ERR_SEC_MISSING_ENTITLEMENT: i32 = -34018;
    error
        .downcast_ref::<security_framework::base::Error>()
        .is_some_and(|error| error.code() == ERR_SEC_MISSING_ENTITLEMENT)
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;

    #[test]
    fn an_unprovisioned_process_falls_back_to_the_login_keychain() {
        let store = SignInStore::platform().unwrap();
        assert!(store.store.vendor().starts_with("macOS Keychain Store"));
    }
}
