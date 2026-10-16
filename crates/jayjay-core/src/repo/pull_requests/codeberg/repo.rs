use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct ForgejoRepo {
    #[serde(default)]
    default_branch: String,
    #[serde(default)]
    permissions: Permissions,
}

#[derive(Deserialize, Default)]
struct Permissions {
    #[serde(default)]
    push: bool,
}

impl ForgejoRepo {
    pub(crate) fn default_branch(&self) -> &str {
        &self.default_branch
    }

    pub(crate) fn push(&self) -> bool {
        self.permissions.push
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_fields_fail_closed() {
        let repo: ForgejoRepo = serde_json::from_str(r#"{"full_name": "o/r"}"#).unwrap();
        assert!(!repo.push());
        assert_eq!(repo.default_branch(), "");
    }
}
