use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, LazyLock},
};

use gpui::{
    AnyElement, App, DevicePixels, InteractiveElement, IntoElement, ObjectFit, ParentElement,
    RenderImage, SharedString, Styled, Task, Window, canvas, div, size,
};
use resvg::{tiny_skia, usvg};

use super::media_diff::media_image;
use crate::app::theme::Theme;

#[derive(Clone, PartialEq, Eq)]
struct SvgRequest {
    content: Arc<str>,
    color: u32,
    width: u32,
    height: u32,
}

struct SvgRaster {
    image: Arc<RenderImage>,
    intrinsic_size: gpui::Size<DevicePixels>,
}

struct PreviewState {
    request: SvgRequest,
    result: Rc<RefCell<PreviewResult>>,
    _task: Task<()>,
}

struct PreviewResult {
    raster: Option<Arc<SvgRaster>>,
    loaded: bool,
}

pub(super) fn svg_preview(content: &str, theme: &Theme) -> impl IntoElement {
    let content: Arc<str> = content.into();
    let color = theme.fg;
    let dim = theme.fg_dim;
    canvas(
        move |bounds, window, cx| {
            let scale = window.scale_factor();
            // Bucket resize requests so dragging a pane does not rasterize every intermediate pixel size.
            let dimension = |value: gpui::Pixels| {
                ((f32::from(value) * scale).ceil() as u32)
                    .clamp(1, 2048)
                    .div_ceil(64)
                    * 64
            };
            let request = SvgRequest {
                content,
                color,
                width: dimension(bounds.size.width),
                height: dimension(bounds.size.height),
            };
            let (raster, loaded) = window.with_global_id("svg-raster".into(), |id, window| {
                window.with_element_state::<PreviewState, _>(id, |state, window| {
                    let state = match state {
                        Some(state) if state.request == request => state,
                        previous => {
                            let raster = previous
                                .as_ref()
                                .filter(|state| {
                                    state.request.content == request.content
                                        && state.request.color == request.color
                                })
                                .and_then(|state| state.result.borrow().raster.clone());
                            PreviewState::new(request, raster, window, cx)
                        }
                    };
                    let result = {
                        let result = state.result.borrow();
                        (result.raster.clone(), result.loaded)
                    };
                    (result, state)
                })
            });
            let (mut element, fitted): (AnyElement, _) = match raster {
                Some(raster)
                    if bounds.size.width > gpui::Pixels::ZERO
                        && bounds.size.height > gpui::Pixels::ZERO =>
                {
                    let fitted = ObjectFit::ScaleDown.get_bounds(bounds, raster.intrinsic_size);
                    (
                        media_image(raster.image.clone())
                            .w(fitted.size.width)
                            .h(fitted.size.height)
                            .aspect_ratio(fitted.size.width / fitted.size.height)
                            .debug_selector(|| "svg-preview-image".to_owned())
                            .into_any_element(),
                        fitted,
                    )
                }
                _ => (
                    div()
                        .text_color(gpui::rgb(dim))
                        .child(SharedString::from(if loaded {
                            "(preview unavailable)"
                        } else {
                            ""
                        }))
                        .into_any_element(),
                    bounds,
                ),
            };
            element.layout_as_root(fitted.size.map(gpui::AvailableSpace::Definite), window, cx);
            element.prepaint_at(fitted.origin, window, cx);
            element
        },
        |_, mut element, window, cx| element.paint(window, cx),
    )
    .size_full()
}

impl PreviewState {
    fn new(request: SvgRequest, raster: Option<Arc<SvgRaster>>, window: &Window, cx: &App) -> Self {
        let result = Rc::new(RefCell::new(PreviewResult {
            raster,
            loaded: false,
        }));
        let render = cx.background_executor().spawn({
            let request = request.clone();
            async move { request.render().map(Arc::new) }
        });
        let task = window.spawn(cx, {
            let result = Rc::downgrade(&result);
            async move |cx| {
                let raster = render.await;
                if let Some(result) = result.upgrade() {
                    *result.borrow_mut() = PreviewResult {
                        raster,
                        loaded: true,
                    };
                    cx.update(|window, _| window.refresh()).ok();
                }
            }
        });
        Self {
            request,
            result,
            _task: task,
        }
    }
}

impl SvgRequest {
    fn render(&self) -> Option<SvgRaster> {
        static FONTS: LazyLock<Arc<usvg::fontdb::Database>> = LazyLock::new(|| {
            let mut fonts = usvg::fontdb::Database::new();
            fonts.load_system_fonts();
            Arc::new(fonts)
        });
        let xml = usvg::roxmltree::Document::parse_with_options(
            &self.content,
            usvg::roxmltree::ParsingOptions {
                allow_dtd: true,
                ..Default::default()
            },
        )
        .ok()?;
        let root = xml.root_element();
        // A presentation attribute supplies inheritance without overriding the document's own CSS or colors.
        let source = if root.attribute("color").is_none() {
            let start = root.range().start + 1;
            let end = start
                + self.content[start..]
                    .find(|c: char| c.is_whitespace() || c == '/' || c == '>')?;
            format!(
                "{} color=\"#{:06x}\"{}",
                &self.content[..end],
                self.color,
                &self.content[end..]
            )
        } else {
            self.content.to_string()
        };
        let options = usvg::Options {
            fontdb: FONTS.clone(),
            ..Default::default()
        };
        let tree = usvg::Tree::from_str(&source, &options).ok()?;
        let intrinsic = tree.size();
        let scale = (self.width as f32 / intrinsic.width())
            .min(self.height as f32 / intrinsic.height())
            .min(2.0);
        let mut pixmap = tiny_skia::Pixmap::new(
            (intrinsic.width() * scale).floor().max(1.) as u32,
            (intrinsic.height() * scale).floor().max(1.) as u32,
        )?;
        resvg::render(
            &tree,
            tiny_skia::Transform::from_scale(scale, scale),
            &mut pixmap.as_mut(),
        );
        for pixel in pixmap.data_mut().as_chunks_mut::<4>().0 {
            gpui::swap_rgba_pa_to_bgra(pixel);
        }
        let buffer = image::RgbaImage::from_raw(pixmap.width(), pixmap.height(), pixmap.take())?;
        Some(SvgRaster {
            image: Arc::new(RenderImage::new([image::Frame::new(buffer)])),
            intrinsic_size: size(
                DevicePixels(intrinsic.width().ceil().min(i32::MAX as f32) as i32),
                DevicePixels(intrinsic.height().ceil().min(i32::MAX as f32) as i32),
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[gpui::test]
    fn dropping_preview_releases_pending_and_completed_renders(cx: &mut gpui::TestAppContext) {
        let cx = cx.add_empty_window();
        for complete in [false, true] {
            let content: Arc<str> = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24"><rect width="24" height="24"/></svg>"#.into();
            let source = Arc::downgrade(&content);
            let state = cx.update(|window, cx| {
                PreviewState::new(
                    SvgRequest {
                        content,
                        color: 0xffffff,
                        width: 64,
                        height: 64,
                    },
                    None,
                    window,
                    cx,
                )
            });
            let result = Rc::downgrade(&state.result);
            let raster = if complete {
                cx.run_until_parked();
                let result = state.result.borrow();
                assert!(result.loaded);
                Some(Arc::downgrade(result.raster.as_ref().unwrap()))
            } else {
                assert!(!state.result.borrow().loaded);
                None
            };
            drop(state);
            cx.run_until_parked();
            assert!(source.upgrade().is_none());
            assert!(result.upgrade().is_none());
            if let Some(raster) = raster {
                assert!(raster.upgrade().is_none());
            }
        }
    }

    #[test]
    fn rasterization_fits_the_budget_before_allocating() {
        for (width, height) in [(100_000, 100_000), (100_000, 1), (1, 100_000), (24, 24)] {
            let request = SvgRequest {
                content: format!(r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}"><rect width="100%" height="100%" fill="red"/></svg>"#).into(),
                color: 0xffffff, width: 320, height: 180,
            };
            let raster = request.render().unwrap();
            let actual = raster.image.size(0);
            assert!(actual.width.0 > 0 && actual.width.0 <= 320);
            assert!(actual.height.0 > 0 && actual.height.0 <= 180);
            assert!(raster.image.as_bytes(0).unwrap().len() <= 320 * 180 * 4);
        }
    }

    #[test]
    fn theme_fallback_preserves_explicit_colors_and_alpha() {
        for (attributes, children, color, expected) in [
            ("", "", 0xffffff, [255, 255, 255, 255]),
            ("", "", 0x123456, [0x56, 0x34, 0x12, 255]),
            (r#"color="red""#, "", 0xffffff, [0, 0, 255, 255]),
            (r#"style="color:blue""#, "", 0xffffff, [255, 0, 0, 255]),
            (
                "",
                "<style>rect { fill: blue }</style>",
                0xffffff,
                [255, 0, 0, 255],
            ),
            (
                "",
                "<style>svg { color: red }</style>",
                0xffffff,
                [0, 0, 255, 255],
            ),
            (
                r#"opacity="0.5" color="red""#,
                "",
                0xffffff,
                [0, 0, 255, 128],
            ),
        ] {
            let request = SvgRequest {
                content: format!(r#"<?xml version="1.0"?><svg xmlns="http://www.w3.org/2000/svg" width="2" height="2" {attributes}>{children}<rect width="2" height="2" fill="currentColor"/></svg>"#).into(),
                color, width: 2, height: 2,
            };
            let raster = request.render().unwrap();
            for (actual, expected) in raster.image.as_bytes(0).unwrap()[..4].iter().zip(expected) {
                assert!(
                    actual.abs_diff(expected) <= 1,
                    "{attributes} {children}: {actual} != {expected}"
                );
            }
        }
    }
}
