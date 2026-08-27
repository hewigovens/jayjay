use jayjay_core::FileTreeEntry;

#[uniffi::export]
fn build_file_tree(paths: Vec<String>) -> Vec<FileTreeEntry> {
    jayjay_core::file_tree::build_file_tree(&paths)
}
