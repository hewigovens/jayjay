use std::convert::Infallible;
use std::sync::{Arc, RwLock};

use jj_lib::repo::ReadonlyRepo;

/// Memoizes a value derived from one `ReadonlyRepo` snapshot; the snapshot is immutable, so pointer identity is enough to know the value still matches.
pub(crate) struct RepoCache<T>(RwLock<Option<(Arc<ReadonlyRepo>, Arc<T>)>>);

impl<T> Default for RepoCache<T> {
    fn default() -> Self {
        Self(RwLock::new(None))
    }
}

impl<T> RepoCache<T> {
    pub(crate) fn get_or_init(
        &self,
        repo: &Arc<ReadonlyRepo>,
        build: impl FnOnce() -> T,
    ) -> Arc<T> {
        let Ok(value) = self.get_or_try_init(repo, || Ok::<_, Infallible>(build()));
        value
    }

    /// A failed build is not cached, so the next call retries it.
    pub(crate) fn get_or_try_init<E>(
        &self,
        repo: &Arc<ReadonlyRepo>,
        build: impl FnOnce() -> Result<T, E>,
    ) -> Result<Arc<T>, E> {
        if let Some((cached_repo, value)) = self.0.read().unwrap().as_ref()
            && Arc::ptr_eq(cached_repo, repo)
        {
            return Ok(value.clone());
        }
        let value = Arc::new(build()?);
        *self.0.write().unwrap() = Some((repo.clone(), value.clone()));
        Ok(value)
    }
}
