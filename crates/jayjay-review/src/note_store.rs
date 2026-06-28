use jayjay_primitives::{NoteAnchor, NoteEntry};

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
        let now = self.clock.now_ms();
        if let Some(note) = self
            .state
            .notes
            .iter_mut()
            .filter_map(StoredNote::parsed_mut)
            .find(|note| !note.resolved && note.same_line(&anchor))
        {
            note.update_at_anchor(anchor, body, now);
            let out = note.clone();
            self.save();
            return out;
        }
        let note = NoteEntry::new(self.id_source.next_id(), anchor, body, now);
        self.state.notes.push(StoredNote::new(note.clone()));
        self.save();
        note
    }

    pub fn update_note(&mut self, id: &str, body: &str) -> Option<NoteEntry> {
        let now = self.clock.now_ms();
        let note = self.find_note_mut(id)?;
        note.update_body(body, now);
        let out = note.clone();
        self.save();
        Some(out)
    }

    pub fn delete_note(&mut self, id: &str) -> bool {
        let before = self.state.notes.len();
        self.state
            .notes
            .retain(|note| note.parsed().is_none_or(|note| note.id != id));
        let deleted = self.state.notes.len() != before;
        if deleted {
            self.save();
        }
        deleted
    }

    pub fn resolve_note(&mut self, id: &str) -> Option<NoteEntry> {
        let now = self.clock.now_ms();
        let note = self.find_note_mut(id)?;
        let changed = note.resolve(now);
        let out = note.clone();
        if changed {
            self.save();
        }
        Some(out)
    }

    fn find_note_mut(&mut self, id: &str) -> Option<&mut NoteEntry> {
        self.state
            .notes
            .iter_mut()
            .filter_map(StoredNote::parsed_mut)
            .find(|note| note.id == id)
    }
}
