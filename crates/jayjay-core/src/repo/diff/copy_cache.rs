use std::collections::VecDeque;
use std::sync::{Arc, Mutex, MutexGuard};

use jj_lib::backend::{CommitId, CopyRecord};

use crate::types::JayResult;

type Endpoints = (CommitId, CommitId);
type Slot = Arc<Mutex<Option<Arc<Vec<CopyRecord>>>>>;

/// Rename detection compares every added file with every removed or modified one, so a change's records are computed once; commit ids name their content, so an entry never goes stale.
#[derive(Default)]
pub(crate) struct CopyRecordsCache(Mutex<VecDeque<(Endpoints, Slot)>>);

impl CopyRecordsCache {
    const CAPACITY: usize = 16;

    pub(super) fn contains(&self, endpoints: &Endpoints) -> bool {
        let slot = self
            .slots()
            .iter()
            .find(|(key, _)| key == endpoints)
            .map(|(_, slot)| slot.clone());
        // A slot still being loaded is locked, and counts as missing.
        slot.is_some_and(|slot| slot.try_lock().is_ok_and(|records| records.is_some()))
    }

    /// Callers asking for the same endpoints wait for the first load rather than repeating it.
    pub(super) fn get_or_load(
        &self,
        endpoints: &Endpoints,
        load: impl FnOnce() -> JayResult<Vec<CopyRecord>>,
    ) -> JayResult<Arc<Vec<CopyRecord>>> {
        let slot = {
            let mut slots = self.slots();
            match slots.iter().find(|(key, _)| key == endpoints) {
                Some((_, slot)) => slot.clone(),
                None => {
                    if slots.len() == Self::CAPACITY {
                        slots.pop_front();
                    }
                    let slot = Slot::default();
                    slots.push_back((endpoints.clone(), slot.clone()));
                    slot
                }
            }
        };
        let mut records = slot.lock().unwrap_or_else(|error| error.into_inner());
        if let Some(records) = records.as_ref() {
            return Ok(records.clone());
        }
        let loaded = Arc::new(load()?);
        *records = Some(loaded.clone());
        Ok(loaded)
    }

    fn slots(&self) -> MutexGuard<'_, VecDeque<(Endpoints, Slot)>> {
        self.0.lock().unwrap_or_else(|error| error.into_inner())
    }
}
