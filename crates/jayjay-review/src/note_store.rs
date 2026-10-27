use jayjay_primitives::{NoteAnchor, NoteEntry, now_ms};

use super::store::{ReviewStore, StoredNote};

impl ReviewStore {
    pub fn list_notes(&self, change_id: &str, include_resolved: bool) -> Vec<NoteEntry> {
        self.state
            .notes
            .iter()
            .filter_map(StoredNote::parsed)
            .filter(|note| note.change_id == change_id)
            .filter(|note| include_resolved || !note.resolved)
            .cloned()
            .collect()
    }

    pub fn add_note(&mut self, anchor: NoteAnchor, body: &str) -> NoteEntry {
        self.mutate(|store| {
            let now = now_ms();
            if let Some(note) = store
                .state
                .notes
                .iter_mut()
                .filter_map(StoredNote::parsed_mut)
                .find(|note| !note.resolved && note.same_line(&anchor))
            {
                note.update_at_anchor(anchor, body, now);
                return note.clone();
            }
            let note = NoteEntry::new(store.id_source.next_id(), anchor, body, now);
            store.state.notes.push(StoredNote::new(note.clone()));
            note
        })
    }

    pub fn update_note(&mut self, id: &str, body: &str) -> Option<NoteEntry> {
        self.mutate(|store| {
            let now = now_ms();
            let note = store.find_note_mut(id)?;
            note.update_body(body, now);
            Some(note.clone())
        })
    }

    pub fn delete_note(&mut self, id: &str) -> bool {
        self.mutate(|store| {
            let before = store.state.notes.len();
            store
                .state
                .notes
                .retain(|note| note.parsed().is_none_or(|note| note.id != id));
            store.state.notes.len() != before
        })
    }

    pub fn resolve_note(&mut self, id: &str) -> Option<NoteEntry> {
        self.mutate(|store| {
            let now = now_ms();
            let note = store.find_note_mut(id)?;
            note.resolve(now);
            Some(note.clone())
        })
    }

    fn find_note_mut(&mut self, id: &str) -> Option<&mut NoteEntry> {
        self.state
            .notes
            .iter_mut()
            .filter_map(StoredNote::parsed_mut)
            .find(|note| note.id == id)
    }
}
