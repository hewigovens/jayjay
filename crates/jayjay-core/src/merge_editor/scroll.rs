use crate::diff::compute_file_diff_full_plain;
use crate::{MergeEditorHunk, MergePane};

/// Maps each pane through the initial materialized result, retaining conflict identity after edits.
#[derive(Clone, Debug)]
pub struct MergeScrollMap {
    original: String,
    sources: [LineMap; 3],
    result: LineMap,
    hunks: Vec<(u32, f64)>,
}

impl MergeScrollMap {
    pub fn new(
        left: &str,
        base: &str,
        right: &str,
        original: &str,
        hunks: &[MergeEditorHunk],
    ) -> Self {
        Self {
            original: original.to_owned(),
            sources: [
                LineMap::source(original, left, hunks, |h| &h.left),
                LineMap::source(original, base, hunks, |h| &h.base),
                LineMap::source(original, right, hunks, |h| &h.right),
            ],
            result: LineMap::between(original, original),
            hunks: hunks
                .iter()
                .filter_map(|h| {
                    h.occurrence_start(original)
                        .map(|start| (h.index, original[..start].lines().count() as f64))
                })
                .collect(),
        }
    }

    pub fn with_result(&self, result: &str) -> Self {
        Self {
            result: LineMap::between(&self.original, result),
            ..self.clone()
        }
    }

    pub fn map_line(&self, from: MergePane, to: MergePane, line: f64) -> f64 {
        self.pane(to).map(self.pane(from).inverse(line))
    }

    pub fn hunk_line(&self, pane: MergePane, index: u32) -> Option<f64> {
        self.hunks
            .iter()
            .find(|(id, _)| *id == index)
            .map(|(_, line)| self.pane(pane).map(*line))
    }

    fn pane(&self, pane: MergePane) -> &LineMap {
        match pane {
            MergePane::Left => &self.sources[0],
            MergePane::Base => &self.sources[1],
            MergePane::Right => &self.sources[2],
            MergePane::Result => &self.result,
        }
    }
}

#[derive(Clone, Debug)]
struct LineMap(Vec<(f64, f64)>);

impl LineMap {
    fn between(old: &str, new: &str) -> Self {
        let mut anchors = vec![(0.0, 0.0)];
        for line in compute_file_diff_full_plain("", old, new, false).lines {
            if let (Some(a), Some(b)) = (line.old_line_no, line.new_line_no) {
                anchors.push((f64::from(a - 1), f64::from(b - 1)));
                anchors.push((f64::from(a), f64::from(b)));
            }
        }
        anchors.push((old.lines().count() as f64, new.lines().count() as f64));
        Self(anchors)
    }

    fn source(
        original: &str,
        source: &str,
        hunks: &[MergeEditorHunk],
        text: impl Fn(&MergeEditorHunk) -> &str,
    ) -> Self {
        let mut projected = String::new();
        let mut anchors = vec![(0.0, 0.0)];
        let mut offset = 0;
        let mut original_line = 0.0;
        let mut projected_line = 0.0;
        for hunk in hunks {
            let Some(start) = hunk.occurrence_start(original) else {
                continue;
            };
            if start < offset {
                continue;
            }
            for line in original[offset..start].split_inclusive('\n') {
                projected.push_str(line);
                original_line += 1.0;
                projected_line += 1.0;
                anchors.push((original_line, projected_line));
            }
            projected.push_str(text(hunk));
            original_line += hunk.raw.lines().count() as f64;
            projected_line += text(hunk).lines().count() as f64;
            anchors.push((original_line, projected_line));
            offset = start + hunk.raw.len();
        }
        for line in original[offset..].split_inclusive('\n') {
            projected.push_str(line);
            original_line += 1.0;
            projected_line += 1.0;
            anchors.push((original_line, projected_line));
        }
        // Auto-merged regions may contain edits from another side; align the projection to the actual source.
        let actual = Self::between(&projected, source);
        Self(
            anchors
                .into_iter()
                .map(|(a, b)| (a, actual.map(b)))
                .collect(),
        )
    }

    fn map(&self, line: f64) -> f64 {
        self.interpolate(line, false)
    }

    fn inverse(&self, line: f64) -> f64 {
        self.interpolate(line, true)
    }

    fn interpolate(&self, line: f64, reverse: bool) -> f64 {
        let point = |&(a, b): &(f64, f64)| if reverse { (b, a) } else { (a, b) };
        let line = if line.is_finite() { line.max(0.0) } else { 0.0 };
        if reverse && self.0.last().unwrap().1 == 0.0 {
            return 0.0;
        }
        let next = self.0.partition_point(|p| point(p).0 <= line);
        let Some(end) = self.0.get(next).map(point) else {
            return point(self.0.last().unwrap()).1;
        };
        if next == 0 {
            return end.1;
        }
        let start = point(&self.0[next - 1]);
        start.1 + (end.1 - start.1) * (line - start.0) / (end.0 - start.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aligns_conflicts_with_repeated_text_and_side_only_edits() {
        let raw = "<<<<<<<\nleft\n=======\nright\n>>>>>>>\n";
        let hunks = (0..2)
            .map(|index| MergeEditorHunk {
                index,
                occurrence: index,
                raw: raw.into(),
                left: "left\n".into(),
                base: "".into(),
                right: "right\nextra\n".into(),
            })
            .collect::<Vec<_>>();
        let original = format!("header\nside-only\n{raw}middle\n{raw}end\n");
        let map = MergeScrollMap::new(
            "header\nside-only\nleft\nmiddle\nleft\nend\n",
            "header\nmiddle\nend\n",
            "header\nright\nextra\nmiddle\nright\nextra\nend\n",
            &original,
            &hunks,
        );
        assert_eq!(map.hunk_line(MergePane::Left, 1), Some(4.0));
        assert_eq!(map.hunk_line(MergePane::Right, 1), Some(4.0));
        assert_eq!(map.hunk_line(MergePane::Base, 1), Some(2.0));
        assert_eq!(map.map_line(MergePane::Result, MergePane::Left, 8.0), 4.0);
        assert_eq!(map.map_line(MergePane::Left, MergePane::Right, 2.5), 2.0);
        let edited = map.with_result(&format!("new\nheader\nside-only\nleft\nmiddle\n{raw}end\n"));
        assert_eq!(
            edited.map_line(MergePane::Left, MergePane::Result, 4.0),
            5.0
        );
        assert_eq!(edited.hunk_line(MergePane::Right, 1), Some(4.0));
    }

    #[test]
    fn handles_empty_files_unicode_and_missing_final_newlines() {
        let map = MergeScrollMap::new("日本語\nlast", "", "日本語\nlast", "日本語\nlast", &[]);
        assert_eq!(map.map_line(MergePane::Left, MergePane::Right, 1.0), 1.0);
        assert_eq!(map.map_line(MergePane::Left, MergePane::Base, 99.0), 0.0);
        assert_eq!(map.map_line(MergePane::Base, MergePane::Left, 0.0), 0.0);
        assert_eq!(
            map.with_result("")
                .map_line(MergePane::Left, MergePane::Result, 1.0),
            0.0
        );
    }

    #[test]
    fn follows_result_edits_at_line_boundaries() {
        let original = "first\nsecond\nthird\n";
        let map = MergeScrollMap::new(original, original, original, original, &[]);
        let edited = map.with_result("inserted\nfirst\nmore\nsecond\nthird\n");
        assert_eq!(
            edited.map_line(MergePane::Left, MergePane::Result, 0.0),
            1.0
        );
        assert_eq!(
            edited.map_line(MergePane::Left, MergePane::Result, 1.0),
            3.0
        );
        assert_eq!(
            edited.map_line(MergePane::Result, MergePane::Right, 3.5),
            1.5
        );
        let deleted = map.with_result("second\nthird\n");
        assert_eq!(
            deleted.map_line(MergePane::Result, MergePane::Left, 0.0),
            1.0
        );
        assert_eq!(
            deleted.map_line(MergePane::Result, MergePane::Left, 0.5),
            1.5
        );
        let replaced = map.with_result("unrelated\ntext\n");
        assert_eq!(
            replaced.map_line(MergePane::Left, MergePane::Result, 1.5),
            1.0
        );
    }
}
