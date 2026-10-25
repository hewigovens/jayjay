use std::io;

use super::persist::ReviewStore;

impl ReviewStore {
    /// Reload, mutate, and persist under the shared store lock; failed mutations leave no staged changes behind.
    pub fn transaction<T>(
        &mut self,
        mutate: impl FnOnce(&mut Self) -> io::Result<T>,
    ) -> io::Result<T> {
        let _lock = self.begin_mutation()?;
        let mut staged = Self::from_state(self.state.clone());
        std::mem::swap(&mut staged.id_source, &mut self.id_source);
        let result = mutate(&mut staged);
        std::mem::swap(&mut staged.id_source, &mut self.id_source);
        let output = result?;
        let previous = std::mem::replace(&mut self.state, staged.state);
        if let Err(error) = self.save_checked() {
            self.state = previous;
            return Err(error);
        }
        Ok(output)
    }

    pub(crate) fn mutate_checked<T>(
        &mut self,
        mutate: impl FnOnce(&mut Self) -> T,
    ) -> io::Result<T> {
        self.transaction(|store| Ok(mutate(store)))
    }

    pub(crate) fn mutate<T>(&mut self, mutate: impl FnOnce(&mut Self) -> T) -> T {
        if !self.persistence.is_persistent() {
            return mutate(self);
        }
        let mut action = Some(mutate);
        let mut output = None;
        let saved = self.transaction(|store| {
            output = Some(action.take().expect("mutation runs once")(store));
            Ok(())
        });
        if let Err(error) = saved {
            eprintln!("[review_store] mutation: {error}");
        }
        // Legacy infallible note APIs still return a value on storage failure; checked callers use transaction.
        output.unwrap_or_else(|| {
            action.expect("mutation has not run")(&mut Self::from_state(self.state.clone()))
        })
    }
}
