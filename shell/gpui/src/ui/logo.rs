use std::sync::Arc;

use gpui::{AnyElement, App, Global, IntoElement, RenderImage, Styled, div, img, px};

const LOGO_SVG: &[u8] = include_bytes!("../../assets/icons/logo.svg");
// GPUI rasterizes at 2x, keeping the largest 88pt logo sharp on high-density displays.
const LOGO_SCALE: f32 = 256. / 1024.;

#[derive(Clone)]
pub(crate) struct Logo {
    image: Option<Arc<RenderImage>>,
}

struct SharedLogo(Logo);

impl Global for SharedLogo {}

impl Logo {
    pub(crate) fn load(cx: &mut App) -> Self {
        if let Some(shared) = cx.try_global::<SharedLogo>() {
            return shared.0.clone();
        }
        let image = cx
            .svg_renderer()
            .render_single_frame(LOGO_SVG, LOGO_SCALE)
            .inspect_err(|err| eprintln!("[jayjay-gpui] failed to render logo: {err}"))
            .ok();
        let logo = Self { image };
        cx.set_global(SharedLogo(logo.clone()));
        logo
    }

    pub(crate) fn image(&self, size: f32) -> AnyElement {
        match &self.image {
            Some(image) => img(image.clone())
                .w(px(size))
                .h(px(size))
                .rounded_lg()
                .into_any_element(),
            None => div().w(px(size)).h(px(size)).into_any_element(),
        }
    }
}

#[cfg(test)]
mod tests {
    use gpui::TestAppContext;

    use super::*;

    #[gpui::test]
    fn bundled_logo_is_decoded_once_and_shared(cx: &mut TestAppContext) {
        cx.update(|cx| {
            let logo = Logo::load(cx);
            let image = logo.image.expect("bundled JayJay logo SVG should render");
            assert_eq!(image.frame_count(), 1);
            assert!(image.as_bytes(0).is_some());

            let again = Logo::load(cx);
            assert!(Arc::ptr_eq(
                &image,
                again.image.as_ref().expect("cached logo present")
            ));
        });
    }
}
