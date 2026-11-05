use super::icon::FlockIcon;
use gpui::{AnyElement, App, IntoElement, RenderImage, Styled, img, px};
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
                    let (light, dark) = icon.svgs()?;
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
