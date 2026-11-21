use std::ops::Range;

/// Where each block of a stack sits, from the heights of the blocks above it.
pub(crate) struct StackLayout {
    tops: Vec<f32>,
}

impl StackLayout {
    pub(super) fn new(heights: &[f32]) -> Self {
        let mut tops = Vec::with_capacity(heights.len() + 1);
        let mut top = 0.;
        tops.push(top);
        for height in heights {
            top += height;
            tops.push(top);
        }
        Self { tops }
    }

    pub(super) fn total(&self) -> f32 {
        self.tops.last().copied().unwrap_or(0.)
    }

    pub(super) fn top(&self, block: usize) -> f32 {
        self.tops
            .get(block)
            .copied()
            .unwrap_or_else(|| self.total())
    }

    fn block_at(&self, y: f32) -> usize {
        self.tops
            .partition_point(|top| *top <= y)
            .saturating_sub(1)
            .min(self.tops.len().saturating_sub(2))
    }

    /// The block under `y` and how far into it `y` is.
    pub(super) fn locate(&self, y: f32) -> Option<(usize, f32)> {
        if self.tops.len() < 2 {
            return None;
        }
        let block = self.block_at(y.max(0.));
        Some((block, (y - self.tops[block]).max(0.)))
    }

    pub(super) fn blocks_between(&self, top: f32, bottom: f32) -> Range<usize> {
        if self.tops.len() < 2 {
            return 0..0;
        }
        self.block_at(top.max(0.))..self.block_at(bottom.max(0.)) + 1
    }
}

#[cfg(test)]
mod tests {
    use super::StackLayout;

    #[test]
    fn a_block_resizing_above_keeps_the_located_block_in_place() {
        let before = StackLayout::new(&[100., 200., 300.]);
        let (block, offset) = before.locate(350.).unwrap();
        assert_eq!((block, offset), (2, 50.));

        let after = StackLayout::new(&[100., 900., 300.]);
        assert_eq!(after.top(block) + offset, 1050.);
    }

    #[test]
    fn blocks_between_covers_partly_visible_blocks_only() {
        let layout = StackLayout::new(&[100., 200., 300.]);
        assert_eq!(layout.blocks_between(0., 99.), 0..1);
        assert_eq!(layout.blocks_between(150., 320.), 1..3);
        assert_eq!(layout.blocks_between(900., 2000.), 2..3);
        assert_eq!(StackLayout::new(&[]).blocks_between(0., 100.), 0..0);
    }
}
