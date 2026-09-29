use super::LoadedDiff;
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub struct DiffCache {
    pub loaded: HashMap<String, LoadedDiff>,
    pub(super) preloads_in_flight: HashSet<String>,
    pub(super) load_failures: HashSet<String>,
}
