mod catalog;
mod identifier;
mod message;
mod rust_source;
mod swift_source;

use std::fs;
use std::path::{Path, PathBuf};

use catalog::{Catalog, Locale};

const SOURCE_LOCALE: &str = "en-US";

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("locales");
    println!("cargo:rerun-if-changed={}", root.display());

    let source = Catalog::read_dir(&root.join(SOURCE_LOCALE));
    let locales: Vec<Locale> = catalog::sorted_entries(&root)
        .iter()
        .map(|dir| Locale::read(dir, &source))
        .collect();

    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    fs::write(
        out.join("messages.rs"),
        rust_source::render(&source, &locales),
    )
    .unwrap();
    fs::write(out.join("Localized.swift"), swift_source::render(&source)).unwrap();
}
