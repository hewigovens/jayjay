use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use jayjay_primitives::hex_sha256;

use super::SideHighlights;

pub(super) struct HighlightCache {
    entries: Mutex<VecDeque<Entry>>,
    max_entries: usize,
    max_bytes: usize,
}

struct Entry {
    source_hash: String,
    language: String,
    highlights: Arc<SideHighlights>,
    bytes: usize,
}

impl HighlightCache {
    pub(super) const fn new(max_entries: usize, max_bytes: usize) -> Self {
        Self {
            entries: Mutex::new(VecDeque::new()),
            max_entries,
            max_bytes,
        }
    }

    pub(super) fn get_or_insert_with(
        &self,
        source: &str,
        language: &str,
        compute: impl FnOnce() -> SideHighlights,
    ) -> Arc<SideHighlights> {
        let source_hash = hex_sha256(source.as_bytes());
        {
            let mut entries = self
                .entries
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            if let Some(index) = entries
                .iter()
                .position(|entry| entry.source_hash == source_hash && entry.language == language)
            {
                let entry = entries.remove(index).expect("matched entry");
                let highlights = entry.highlights.clone();
                entries.push_front(entry);
                return highlights;
            }
        }

        // Parsing runs outside the lock so unrelated files and windows do not serialize on tree-sitter.
        let highlights = Arc::new(compute());
        let bytes = highlights.retained_bytes();
        if self.max_entries == 0 || bytes > self.max_bytes {
            return highlights;
        }
        let mut entries = self
            .entries
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        // Another thread may have finished this source while we were parsing it.
        if let Some(entry) = entries
            .iter()
            .find(|entry| entry.source_hash == source_hash && entry.language == language)
        {
            return entry.highlights.clone();
        }
        let mut retained: usize = entries.iter().map(|entry| entry.bytes).sum();
        while entries.len() >= self.max_entries || retained + bytes > self.max_bytes {
            let Some(entry) = entries.pop_back() else {
                break;
            };
            retained -= entry.bytes;
        }
        entries.push_front(Entry {
            source_hash,
            language: language.to_owned(),
            highlights: highlights.clone(),
            bytes,
        });
        highlights
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::LineIndex;

    fn load(cache: &HighlightCache, source: &str, language: &str) -> Arc<SideHighlights> {
        cache.get_or_insert_with(source, language, || {
            SideHighlights::uncached(source, &LineIndex::from_text(source), language)
        })
    }

    #[test]
    fn reuse_crosses_threads_but_content_and_language_changes_miss() {
        let cache = Arc::new(HighlightCache::new(8, 1024 * 1024));
        let source = "const value: usize = 1;\n";
        let first = load(&cache, source, "rust");
        let other_thread = cache.clone();
        let reused = std::thread::spawn(move || {
            other_thread.get_or_insert_with(source, "rust", || panic!("source parsed again"))
        })
        .join()
        .unwrap();
        assert!(Arc::ptr_eq(&first, &reused));
        assert!(!Arc::ptr_eq(
            &first,
            &load(&cache, "const value: usize = 2;\n", "rust")
        ));
        let plain = load(&cache, source, "plaintext");
        assert!(!Arc::ptr_eq(&first, &plain));
        assert!(plain.spans().is_empty());
    }

    #[test]
    fn eviction_respects_recency_and_memory_without_invalidating_live_highlights() {
        let sample = load(&HighlightCache::new(0, 0), "let a = 1;\n", "rust");
        let bytes = sample.retained_bytes();
        for cache in [
            HighlightCache::new(2, usize::MAX),
            HighlightCache::new(8, bytes * 2),
        ] {
            let first = load(&cache, "let a = 1;\n", "rust");
            let second = load(&cache, "let b = 2;\n", "rust");
            assert!(Arc::ptr_eq(&first, &load(&cache, "let a = 1;\n", "rust")));
            load(&cache, "let c = 3;\n", "rust");
            assert!(Arc::ptr_eq(&first, &load(&cache, "let a = 1;\n", "rust")));
            assert!(!Arc::ptr_eq(&second, &load(&cache, "let b = 2;\n", "rust")));
            assert!(!second.spans().is_empty());
        }
        let cache = HighlightCache::new(8, bytes - 1);
        let first = load(&cache, "let a = 1;\n", "rust");
        assert!(!Arc::ptr_eq(&first, &load(&cache, "let a = 1;\n", "rust")));
    }
}
