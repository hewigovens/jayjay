use std::cmp::Ordering;

use gpui::{Pixels, ScrollHandle, point, px};

#[derive(Default)]
pub(crate) struct MergeHunkListScroll {
    pub(crate) handle: ScrollHandle,
    last_offset: Pixels,
    pending_reveal: Option<u32>,
}

impl MergeHunkListScroll {
    pub(crate) fn reveal(&mut self, hunk: u32) -> bool {
        self.pending_reveal = Some(hunk);
        self.apply_pending_reveal()
    }

    pub(crate) fn cancel_reveal(&mut self) {
        self.pending_reveal = None;
    }

    pub(crate) fn apply_pending_reveal(&mut self) -> bool {
        let Some(hunk) = self.pending_reveal else {
            return true;
        };
        let viewport = self.handle.bounds();
        let Some(card) = self.handle.bounds_for_item(hunk as usize) else {
            return false;
        };
        if viewport.size.height <= px(0.) {
            return false;
        }
        let offset = reveal_card_offset(
            self.handle.offset().y,
            card.top() - viewport.top(),
            card.size.height,
            viewport.size.height,
            self.handle.max_offset().y,
        );
        self.handle
            .set_offset(point(self.handle.offset().x, offset));
        self.last_offset = offset;
        self.pending_reveal = None;
        true
    }

    pub(crate) fn user_scrolled(&mut self) -> Option<u32> {
        let offset = self.handle.offset().y;
        if offset == self.last_offset {
            return None;
        }
        self.last_offset = offset;
        let viewport = self.handle.bounds();
        let cards = (0..self.handle.children_count())
            .filter_map(|index| {
                self.handle.bounds_for_item(index).map(|bounds| {
                    (
                        f32::from(bounds.top() - viewport.top()),
                        f32::from(bounds.bottom() - viewport.top()),
                    )
                })
            })
            .collect::<Vec<_>>();
        nearest_card(&cards, f32::from(viewport.size.height) / 2.).map(|index| index as u32)
    }
}

/// Distance is 0 while the viewport center is inside the card, so a tall visible hunk beats a nearer small one.
fn nearest_card(cards: &[(f32, f32)], viewport_mid: f32) -> Option<usize> {
    cards
        .iter()
        .enumerate()
        .min_by(|(_, (top, bottom)), (_, (other_top, other_bottom))| {
            let distance =
                |top: f32, bottom: f32| (top - viewport_mid).max(viewport_mid - bottom).max(0.);
            distance(*top, *bottom)
                .partial_cmp(&distance(*other_top, *other_bottom))
                .unwrap_or(Ordering::Equal)
        })
        .map(|(index, _)| index)
}

/// Offsets are negative-down, as `ScrollHandle` stores them.
fn reveal_card_offset(
    current: Pixels,
    card_top: Pixels,
    card_height: Pixels,
    viewport_height: Pixels,
    max_scroll: Pixels,
) -> Pixels {
    let shift = if card_height > viewport_height {
        -card_top
    } else {
        viewport_height / 2. - (card_top + card_height / 2.)
    };
    (current + shift).max(-max_scroll).min(px(0.))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tall_visible_card_beats_a_nearer_small_one() {
        let cards = [(0., 2000.), (2012., 2112.)];
        for mid in [1690., 1890.] {
            assert_eq!(nearest_card(&cards, mid), Some(0));
        }
        assert_eq!(nearest_card(&cards, 2102.), Some(1));
    }

    #[test]
    fn reveal_centers_short_cards_and_tops_tall_cards() {
        assert_eq!(
            reveal_card_offset(px(-100.), px(500.), px(100.), px(400.), px(2000.)),
            px(-450.)
        );
        assert_eq!(
            reveal_card_offset(px(-100.), px(500.), px(900.), px(400.), px(2000.)),
            px(-600.)
        );
        assert_eq!(
            reveal_card_offset(px(0.), px(100.), px(100.), px(400.), px(2000.)),
            px(0.)
        );
        assert_eq!(
            reveal_card_offset(px(-1900.), px(500.), px(100.), px(400.), px(2000.)),
            px(-2000.)
        );
    }
}
