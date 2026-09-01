use std::ffi::OsString;
use std::path::Path;
use std::sync::{Mutex, MutexGuard, PoisonError};

static STORE_ENV_LOCK: Mutex<()> = Mutex::new(());

/// Points `JAYJAY_REVIEW_STORE_PATH` at `store` for the guard's lifetime; the env var is process-wide, so the guard also serializes tests within a test binary.
pub fn review_store_env(store: &Path) -> ReviewStoreEnv {
    let lock = STORE_ENV_LOCK
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    let previous = std::env::var_os("JAYJAY_REVIEW_STORE_PATH");
    unsafe {
        std::env::set_var("JAYJAY_REVIEW_STORE_PATH", store);
    }
    ReviewStoreEnv {
        previous,
        _lock: lock,
    }
}

pub struct ReviewStoreEnv {
    previous: Option<OsString>,
    _lock: MutexGuard<'static, ()>,
}

impl Drop for ReviewStoreEnv {
    fn drop(&mut self) {
        unsafe {
            match &self.previous {
                Some(previous) => std::env::set_var("JAYJAY_REVIEW_STORE_PATH", previous),
                None => std::env::remove_var("JAYJAY_REVIEW_STORE_PATH"),
            }
        }
    }
}
