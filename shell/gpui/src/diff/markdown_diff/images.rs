use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Cursor;
use std::rc::Rc;
use std::sync::{Arc, Weak};

use base64::Engine as _;
use gpui::{Image, ImageFormat, ImageSource, Pixels, Size, px, size};
use jayjay_markdown::{MarkdownDocument, MarkdownImageSource};

pub(crate) type MarkdownImageCacheSlot = Rc<RefCell<MarkdownImageCache>>;

/// Holding a `Weak` keeps the document's allocation alive, so a reloaded document can never reuse the address the entries were keyed on.
#[derive(Default)]
pub(crate) struct MarkdownImageCache {
    document: Weak<MarkdownDocument>,
    entries: HashMap<String, Option<ResolvedImage>>,
}

impl MarkdownImageCache {
    pub(crate) fn clear(&mut self) {
        self.document = Weak::new();
        self.entries.clear();
    }
}

#[derive(Clone)]
pub(super) struct ResolvedImage {
    pub(super) source: ImageSource,
    /// `None` for SVG, which gpui sizes from the file itself.
    pub(super) size: Option<Size<Pixels>>,
}

pub(crate) struct MarkdownImages<'a> {
    pub(crate) repo_path: &'a str,
    pub(crate) document_path: &'a str,
    pub(crate) cache: &'a MarkdownImageCacheSlot,
}

impl MarkdownImages<'_> {
    pub(super) fn sync(&self, document: &Arc<MarkdownDocument>) {
        let mut cache = self.cache.borrow_mut();
        if !std::ptr::eq(cache.document.as_ptr(), Arc::as_ptr(document)) {
            cache.clear();
            cache.document = Arc::downgrade(document);
        }
    }

    pub(super) fn resolve(&self, source: &str) -> Option<ResolvedImage> {
        let mut cache = self.cache.borrow_mut();
        if let Some(resolved) = cache.entries.get(source) {
            return resolved.clone();
        }
        let resolved = resolve_image(self.repo_path, self.document_path, source);
        cache.entries.insert(source.to_owned(), resolved.clone());
        resolved
    }
}

// Same resolution as the SwiftUI preview: relative to the document's directory, contained to the checkout.
fn resolve_image(repo_path: &str, document_path: &str, source: &str) -> Option<ResolvedImage> {
    match MarkdownImageSource::parse(source) {
        MarkdownImageSource::Data { subtype, base64 } => {
            if base64.len() > jayjay_core::MAX_IMAGE_BYTES / 3 * 4 {
                return None;
            }
            let format = match subtype.to_ascii_lowercase().as_str() {
                "png" => ImageFormat::Png,
                "jpeg" | "jpg" => ImageFormat::Jpeg,
                "gif" => ImageFormat::Gif,
                "webp" => ImageFormat::Webp,
                "bmp" => ImageFormat::Bmp,
                _ => return None,
            };
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(base64)
                .ok()?;
            let size = image_size(Cursor::new(&bytes))?;
            Some(ResolvedImage {
                source: ImageSource::Image(Arc::new(Image::from_bytes(format, bytes))),
                size: Some(size),
            })
        }
        MarkdownImageSource::Relative(relative) => {
            let file_path = match document_path.rsplit_once('/') {
                Some((directory, _)) => format!("{directory}/{relative}"),
                None => relative,
            };
            let path = jayjay_core::repo_file_path(repo_path, &file_path)?;
            if std::fs::metadata(&path).ok()?.len() > jayjay_core::MAX_IMAGE_BYTES as u64 {
                return None;
            }
            let size = if path
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("svg"))
            {
                None
            } else {
                Some(image_size(std::io::BufReader::new(
                    std::fs::File::open(&path).ok()?,
                ))?)
            };
            Some(ResolvedImage {
                source: ImageSource::from(path),
                size,
            })
        }
    }
}

fn image_size(reader: impl std::io::BufRead + std::io::Seek) -> Option<Size<Pixels>> {
    let (width, height) = image::ImageReader::new(reader)
        .with_guessed_format()
        .ok()?
        .into_dimensions()
        .ok()?;
    (width > 0 && height > 0).then(|| size(px(width as f32), px(height as f32)))
}
