use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::sync::{Mutex, MutexGuard, PoisonError};

static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Sets (`Some`) or unsets (`None`) process-wide environment variables for the guard's lifetime; the guard also serializes the tests within a test binary that take one.
pub fn scoped_env<'a>(
    vars: impl IntoIterator<Item = (&'static str, Option<&'a OsStr>)>,
) -> ScopedEnv {
    let lock = ENV_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
    let previous = vars
        .into_iter()
        .map(|(key, value)| {
            let previous = std::env::var_os(key);
            unsafe {
                match value {
                    Some(value) => std::env::set_var(key, value),
                    None => std::env::remove_var(key),
                }
            }
            (key, previous)
        })
        .collect();
    ScopedEnv {
        previous,
        _lock: lock,
    }
}

pub fn review_store_env(store: &Path) -> ScopedEnv {
    scoped_env([("JAYJAY_REVIEW_STORE_PATH", Some(store.as_os_str()))])
}

pub struct ScopedEnv {
    previous: Vec<(&'static str, Option<OsString>)>,
    _lock: MutexGuard<'static, ()>,
}

impl Drop for ScopedEnv {
    fn drop(&mut self) {
        for (key, previous) in self.previous.iter().rev() {
            unsafe {
                match previous {
                    Some(previous) => std::env::set_var(key, previous),
                    None => std::env::remove_var(key),
                }
            }
        }
    }
}
