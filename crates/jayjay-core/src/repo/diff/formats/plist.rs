use std::io::Cursor;

use plist::{Dictionary, Value as PlistValue};

use crate::types::*;

use super::{
    FormatInput, ProjectionPair,
    types::{DiffFormatPlugin, has_extension, project_text_pair, projection_error},
};

pub(super) struct PlistPlugin;

impl DiffFormatPlugin for PlistPlugin {
    fn id(&self) -> &'static str {
        "plist"
    }

    fn version(&self) -> u32 {
        1
    }

    fn label(&self) -> &'static str {
        "Property list"
    }

    fn render_kind(&self) -> DiffRenderKind {
        DiffRenderKind::Text
    }

    fn matches_path(&self, path: &str) -> bool {
        has_extension(path, &["plist"])
    }

    fn matches_input(&self, input: FormatInput<'_>) -> bool {
        self.matches_path(input.path)
            && [input.old, input.new]
                .into_iter()
                .flatten()
                .any(is_binary_plist)
    }

    fn content_gated(&self) -> bool {
        true
    }

    fn virtual_path(&self, path: &str) -> String {
        format!("{path}.xml")
    }

    fn project(&self, input: FormatInput<'_>) -> CoreResult<ProjectionPair> {
        project_text_pair(
            input,
            self.projection(input.path, DiffProjectionMode::Processed),
            project_plist,
        )
    }
}

fn is_binary_plist(bytes: &[u8]) -> bool {
    bytes.starts_with(b"bplist")
}

fn project_plist(bytes: &[u8]) -> CoreResult<String> {
    let value = PlistValue::from_reader(Cursor::new(bytes))
        .map_err(|err| projection_error(format!("parse plist: {err}")))?;
    let sorted = sort_plist_value(value);
    let mut out = Vec::new();
    plist::to_writer_xml(&mut out, &sorted)
        .map_err(|err| projection_error(format!("serialize plist XML: {err}")))?;
    String::from_utf8(out)
        .map_err(|err| projection_error(format!("plist projection is not UTF-8: {err}")))
}

fn sort_plist_value(value: PlistValue) -> PlistValue {
    match value {
        PlistValue::Array(values) => {
            PlistValue::Array(values.into_iter().map(sort_plist_value).collect())
        }
        PlistValue::Dictionary(dictionary) => {
            let mut keys: Vec<String> = dictionary.keys().cloned().collect();
            keys.sort();
            let mut sorted = Dictionary::new();
            for key in keys {
                if let Some(value) = dictionary.get(&key) {
                    sorted.insert(key, sort_plist_value(value.clone()));
                }
            }
            PlistValue::Dictionary(sorted)
        }
        value => value,
    }
}

#[cfg(test)]
mod tests {
    use super::{is_binary_plist, project_plist};
    use plist::{Dictionary, Value as PlistValue};

    #[test]
    fn detects_binary_plist_magic() {
        assert!(is_binary_plist(b"bplist00..."));
        assert!(!is_binary_plist(
            br#"<?xml version="1.0"?><plist version="1.0"/>"#
        ));
    }

    #[test]
    fn projects_binary_plist_as_sorted_xml() {
        let mut dictionary = Dictionary::new();
        dictionary.insert("z".to_owned(), PlistValue::String("last".to_owned()));
        dictionary.insert("a".to_owned(), PlistValue::Integer(1.into()));
        let mut binary = Vec::new();
        plist::to_writer_binary(&mut binary, &PlistValue::Dictionary(dictionary))
            .expect("write binary plist");
        let projected = project_plist(&binary).expect("project plist");

        let a_index = projected.find("<key>a</key>").expect("a key");
        let z_index = projected.find("<key>z</key>").expect("z key");
        assert!(a_index < z_index);
        assert!(projected.contains("<integer>1</integer>"));
    }
}
