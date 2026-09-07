use std::borrow::Cow;

use gpui::{AssetSource, SharedString};

use crate::ui::icons::SVG_ASSETS;

pub(super) struct GpuiAssets;

impl AssetSource for GpuiAssets {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        Ok(SVG_ASSETS
            .iter()
            .find(|(name, _)| *name == path)
            .map(|(_, bytes)| Cow::Borrowed(*bytes)))
    }

    fn list(&self, _path: &str) -> gpui::Result<Vec<SharedString>> {
        Ok(Vec::new())
    }
}
