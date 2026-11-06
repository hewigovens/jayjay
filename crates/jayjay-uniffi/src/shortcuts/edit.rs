#[derive(uniffi::Enum)]
pub enum ShortcutEdit {
    Assign { chord: String },
    Unbind,
    Reset,
    ResetAll,
}
