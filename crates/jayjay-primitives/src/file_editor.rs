#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileEditorData {
    pub path: String,
    pub change_id: String,
    pub file_id: String,
    pub content: String,
}
