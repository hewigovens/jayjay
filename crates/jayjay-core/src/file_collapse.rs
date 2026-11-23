use std::collections::HashSet;

#[derive(Debug, Default)]
pub struct FileCollapse {
    collapsed: HashSet<String>,
}

impl FileCollapse {
    pub fn contains(&self, path: &str) -> bool {
        self.collapsed.contains(path)
    }

    pub fn toggle(&mut self, path: &str) {
        if !self.collapsed.remove(path) {
            self.collapsed.insert(path.to_owned());
        }
    }

    /// False when the file was already in that state.
    pub fn set(&mut self, path: &str, collapsed: bool) -> bool {
        if collapsed {
            self.collapsed.insert(path.to_owned())
        } else {
            self.collapsed.remove(path)
        }
    }

    pub fn collapse_all(&mut self, paths: impl IntoIterator<Item = String>) {
        self.collapsed = paths.into_iter().collect();
    }

    pub fn expand_all(&mut self) {
        self.collapsed.clear();
    }
}
