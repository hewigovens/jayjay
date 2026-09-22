use gpui::{Context, px};

use super::{LineLayout, TextArea};

impl TextArea {
    pub fn center_logical_line(&self) -> f64 {
        let (Some(layout), Some(bounds)) = (&self.last_layout, &self.last_bounds) else {
            return 0.;
        };
        if layout.lines.is_empty() {
            return 0.;
        }
        let center_y = f32::from(self.scroll_y) + f32::from(bounds.size.height) / 2.;
        center_to_logical(
            &rows(&layout.lines),
            f32::from(layout.line_height),
            center_y,
        )
    }

    /// Never emits `TextAreaScrolled`, so followers cannot echo back.
    pub fn scroll_to_logical_line(&mut self, line: f64, cx: &mut Context<Self>) -> bool {
        let (Some(layout), Some(bounds)) = (&self.last_layout, &self.last_bounds) else {
            return false;
        };
        if layout.lines.is_empty() {
            return false;
        }
        let line_height = f32::from(layout.line_height);
        let content_height = line_height * layout.lines.len() as f32;
        let y = logical_to_scroll_y(
            &rows(&layout.lines),
            line_height,
            line,
            f32::from(bounds.size.height),
            content_height,
        );
        let y = px(y);
        if y != self.scroll_y {
            self.scroll_y = y;
            cx.notify();
        }
        true
    }
}

fn rows(lines: &[LineLayout]) -> Vec<(usize, f32)> {
    lines
        .iter()
        .map(|line| (line.logical_line, f32::from(line.top)))
        .collect()
}

fn center_to_logical(rows: &[(usize, f32)], line_height: f32, center_y: f32) -> f64 {
    let row = (center_y / line_height)
        .floor()
        .clamp(0., rows.len() as f32 - 1.) as usize;
    let logical = rows[row].0;
    let group_start = rows[..=row]
        .iter()
        .rposition(|(line, _)| *line != logical)
        .map_or(0, |ix| ix + 1);
    let group_end = rows[row..]
        .iter()
        .position(|(line, _)| *line != logical)
        .map_or(rows.len(), |ix| row + ix);
    let group_top = rows[group_start].1;
    let group_height = (group_end - group_start) as f32 * line_height;
    let fraction = ((center_y - group_top) / group_height).clamp(0., 1.);
    logical as f64 + f64::from(fraction)
}

fn logical_to_scroll_y(
    rows: &[(usize, f32)],
    line_height: f32,
    line: f64,
    viewport_height: f32,
    content_height: f32,
) -> f32 {
    let max_line = rows
        .last()
        .map(|(logical, _)| *logical as f64)
        .unwrap_or(0.)
        + 1.
        - 1e-6;
    let clamped = line.clamp(0., max_line);
    let logical = clamped.floor() as usize;
    let fraction = (clamped - logical as f64) as f32;
    let group_start = rows.partition_point(|(line, _)| *line < logical);
    let group_end = rows.partition_point(|(line, _)| *line <= logical);
    let group_top = rows[group_start].1;
    let group_height = (group_end - group_start) as f32 * line_height;
    let y = group_top + fraction * group_height - viewport_height / 2.;
    y.clamp(0., (content_height - viewport_height).max(0.))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uniform_rows(logical_count: usize) -> Vec<(usize, f32)> {
        (0..logical_count)
            .map(|logical| (logical, logical as f32 * 18.))
            .collect()
    }

    #[test]
    fn center_line_round_trips_through_scroll_to_line() {
        let rows = uniform_rows(100);
        for line in [12.5, 45.25, 90.] {
            let y = logical_to_scroll_y(&rows, 18., line, 180., 1800.);
            assert_eq!(center_to_logical(&rows, 18., y + 90.), line, "line {line}");
        }
    }

    #[test]
    fn wrapped_lines_share_the_logical_fraction_space() {
        let rows = vec![(0, 0.), (1, 18.), (1, 36.), (1, 54.), (2, 72.)];
        assert_eq!(
            center_to_logical(&rows, 18., 27.),
            1. + f64::from(9. / 54_f32)
        );
        assert_eq!(center_to_logical(&rows, 18., 45.), 1.5);
        assert_eq!(
            center_to_logical(&rows, 18., 80.),
            2. + f64::from(8. / 18_f32)
        );
        let y = logical_to_scroll_y(&rows, 18., 1.5, 90., 1000.);
        assert_eq!(y, 0.);
        assert_eq!(center_to_logical(&rows, 18., y + 45.), 1.5);
    }

    #[test]
    fn scroll_to_line_clamps_to_the_scrollable_range() {
        let rows = uniform_rows(10);
        assert_eq!(logical_to_scroll_y(&rows, 18., -5., 180., 180.), 0.);
        assert_eq!(logical_to_scroll_y(&rows, 18., 99., 180., 180.), 0.);
        assert_eq!(center_to_logical(&rows, 18., 90.), 5.);
    }
}
