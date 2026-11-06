use super::chord::ShortcutChord;
use jayjay_core::shortcuts::EffectiveEntry;

#[derive(uniffi::Record)]
pub struct ShortcutBinding {
    pub id: String,
    pub title: String,
    pub context_title: String,
    pub locked: bool,
    pub primary: Option<ShortcutChord>,
    pub aliases: Vec<ShortcutChord>,
    pub modified: bool,
}

impl ShortcutBinding {
    pub(super) fn new(entry: &EffectiveEntry, modified: bool) -> Self {
        let command = entry.command;
        Self {
            id: command.id().to_owned(),
            title: command.title(),
            context_title: command.context().title(),
            locked: command.is_locked(),
            primary: entry.chord.map(Into::into),
            aliases: entry.aliases.iter().copied().map(Into::into).collect(),
            modified,
        }
    }
}
