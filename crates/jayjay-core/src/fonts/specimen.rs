use jayjay_primitives::{HunkType, ShortId};
use jj_diff::{FileDiff, compute_file_diff_full};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecimenChange {
    pub change_id: ShortId,
    pub summary: String,
    pub bookmark: Option<String>,
    pub is_working_copy: bool,
    pub is_immutable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecimenFile {
    pub path: String,
    pub hunk_type: HunkType,
}

/// Shared text for the font and theme settings previews, so both shells show the same specimen.
#[derive(Debug, Clone)]
pub struct FontSpecimen {
    pub summary: String,
    pub change_id: String,
    pub path: String,
    pub changes: Vec<SpecimenChange>,
    pub files: Vec<SpecimenFile>,
    pub diff: FileDiff,
}

const PATH: &str = "src/jay/land.rs";

const OLD: &str = "fn land(jay: &Jay) -> Result<Perch> {
    if jay.speed >= 10 && !jay.tired {
        return Err(Sky::TooFast);
    }
    Ok(Branch)
}
";

const NEW: &str = "fn land(jay: &Jay) -> Result<Perch> {
    if jay.speed >= 10 && !jay.tired {
        return Err(Sky::TooFast);
    }
    // 0O 1Il: a jay never forgets a seed
    match jay.seeds[..] {
        [] => Ok(Branch),
        [s, ..] if s <= 3 || s != 7 => Ok(Feeder),
        _ => Ok(Nest),
    }
}
";

impl Default for FontSpecimen {
    fn default() -> Self {
        let short_id = |id: &str, short_len: u32| ShortId {
            id: id.to_owned(),
            short_len,
        };
        Self {
            summary: "Teach the jay to land".to_owned(),
            change_id: "zoomzoom".to_owned(),
            path: PATH.to_owned(),
            changes: vec![
                SpecimenChange {
                    change_id: short_id("zoomzoom", 4),
                    summary: "Teach the jay to land".to_owned(),
                    bookmark: None,
                    is_working_copy: true,
                    is_immutable: false,
                },
                SpecimenChange {
                    change_id: short_id("mvwtqkpz", 3),
                    summary: "Count seeds in flight".to_owned(),
                    bookmark: Some("perch".to_owned()),
                    is_working_copy: false,
                    is_immutable: false,
                },
                SpecimenChange {
                    change_id: short_id("kqrxvsln", 3),
                    summary: "Build the feeder".to_owned(),
                    bookmark: Some("main".to_owned()),
                    is_working_copy: false,
                    is_immutable: true,
                },
            ],
            files: vec![
                SpecimenFile {
                    path: PATH.to_owned(),
                    hunk_type: HunkType::Modified,
                },
                SpecimenFile {
                    path: "src/jay/perch.rs".to_owned(),
                    hunk_type: HunkType::Added,
                },
                SpecimenFile {
                    path: "src/jay/glide.rs".to_owned(),
                    hunk_type: HunkType::Removed,
                },
            ],
            diff: compute_file_diff_full(PATH, OLD, NEW, false),
        }
    }
}
