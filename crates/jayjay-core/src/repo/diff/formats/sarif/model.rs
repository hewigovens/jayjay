use serde::Deserialize;

#[derive(Deserialize)]
pub(super) struct SarifReport {
    pub(super) version: Option<String>,
    pub(super) runs: Option<Vec<SarifRun>>,
}

#[derive(Deserialize)]
pub(super) struct SarifRun {
    pub(super) tool: Option<SarifTool>,
    #[serde(default)]
    pub(super) results: Vec<SarifResult>,
}

#[derive(Deserialize)]
pub(super) struct SarifTool {
    pub(super) driver: Option<SarifDriver>,
}

#[derive(Deserialize)]
pub(super) struct SarifDriver {
    pub(super) name: Option<String>,
    #[serde(default)]
    pub(super) rules: Vec<SarifRule>,
}

#[derive(Deserialize)]
pub(super) struct SarifRule {
    pub(super) id: Option<String>,
    pub(super) name: Option<String>,
    #[serde(rename = "shortDescription")]
    pub(super) short_description: Option<SarifMessage>,
    #[serde(rename = "fullDescription")]
    pub(super) full_description: Option<SarifMessage>,
}

#[derive(Deserialize)]
pub(super) struct SarifResult {
    #[serde(rename = "ruleId")]
    pub(super) rule_id: Option<String>,
    pub(super) level: Option<String>,
    pub(super) message: Option<SarifMessage>,
    #[serde(default)]
    pub(super) locations: Vec<SarifLocation>,
}

#[derive(Deserialize)]
pub(super) struct SarifMessage {
    pub(super) text: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct SarifLocation {
    #[serde(rename = "physicalLocation")]
    pub(super) physical_location: Option<SarifPhysicalLocation>,
}

#[derive(Deserialize)]
pub(super) struct SarifPhysicalLocation {
    #[serde(rename = "artifactLocation")]
    pub(super) artifact_location: Option<SarifArtifactLocation>,
    pub(super) region: Option<SarifRegion>,
}

#[derive(Deserialize)]
pub(super) struct SarifArtifactLocation {
    pub(super) uri: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct SarifRegion {
    #[serde(rename = "startLine")]
    pub(super) start_line: Option<u64>,
    #[serde(rename = "startColumn")]
    pub(super) start_column: Option<u64>,
}
