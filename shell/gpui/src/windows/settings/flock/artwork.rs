use gpui::{AnyElement, App, IntoElement, RenderImage, Styled, img, px};
use jayjay_core::flock::FlockIcon;
use std::sync::Arc;

pub(super) struct Artwork {
    pub feather: Arc<RenderImage>,
    birds: Vec<(FlockIcon, Arc<RenderImage>, Arc<RenderImage>)>,
}

impl Artwork {
    pub fn load(cx: &App) -> Self {
        let render = |bytes: &[u8]| {
            cx.svg_renderer()
                .render_single_frame(bytes, 0.25)
                .expect("bundled Flock SVG")
        };
        Self {
            feather: render(include_bytes!(
                "../../../../../../assets/flock/flock-feather.svg"
            )),
            birds: FlockIcon::ALL
                .into_iter()
                .filter_map(|icon| {
                    let (light, dark) = svgs(icon)?;
                    Some((icon, render(light), render(dark)))
                })
                .collect(),
        }
    }

    pub fn bird(&self, icon: FlockIcon, dark: bool, size: f32) -> Option<AnyElement> {
        let (_, light_image, dark_image) = self.birds.iter().find(|(bird, ..)| *bird == icon)?;
        Some(
            img(if dark { dark_image } else { light_image }.clone())
                .size(px(size))
                .into_any_element(),
        )
    }
}

fn svgs(icon: FlockIcon) -> Option<(&'static [u8], &'static [u8])> {
    Some(match icon {
        FlockIcon::Classic => return None,
        FlockIcon::FishOwl => (
            include_bytes!("../../../../../../assets/flock/flock-fish-owl-light.svg"),
            include_bytes!("../../../../../../assets/flock/flock-fish-owl-dark.svg"),
        ),
        FlockIcon::GoldenPheasant => (
            include_bytes!("../../../../../../assets/flock/flock-golden-pheasant-light.svg"),
            include_bytes!("../../../../../../assets/flock/flock-golden-pheasant-dark.svg"),
        ),
        FlockIcon::Kingfisher => (
            include_bytes!("../../../../../../assets/flock/flock-kingfisher-light.svg"),
            include_bytes!("../../../../../../assets/flock/flock-kingfisher-dark.svg"),
        ),
        FlockIcon::Flycatcher => (
            include_bytes!("../../../../../../assets/flock/flock-flycatcher-light.svg"),
            include_bytes!("../../../../../../assets/flock/flock-flycatcher-dark.svg"),
        ),
    })
}
