use jayjay_core::ShortId;

pub(crate) struct DagRebaseRequest {
    pub(crate) source_rev: String,
    pub(crate) source_change_id: ShortId,
    pub(crate) source_commit_id: ShortId,
    pub(crate) source_label: String,
    pub(crate) dest_rev: String,
    pub(crate) dest_change_id: ShortId,
    pub(crate) dest_commit_id: ShortId,
    pub(crate) dest_label: String,
}
