use jj_diff::{FileDiff, compute_file_diff_full};

/// Shared text for the font settings preview, so both shells show the same specimen.
#[derive(Debug, Clone)]
pub struct FontSpecimen {
    pub summary: String,
    pub change_id: String,
    pub path: String,
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
        Self {
            summary: "Teach the jay to land".to_owned(),
            change_id: "zoomzoom".to_owned(),
            path: PATH.to_owned(),
            diff: compute_file_diff_full(PATH, OLD, NEW, false),
        }
    }
}
