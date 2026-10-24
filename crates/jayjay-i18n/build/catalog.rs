use std::fs;
use std::path::{Path, PathBuf};

use fluent_bundle::{FluentBundle, FluentResource};
use fluent_syntax::ast::Entry;

use crate::SOURCE_LOCALE;
use crate::identifier::Identifier;
use crate::message::Message;

pub(crate) struct Catalog {
    pub(crate) stem: String,
    pub(crate) path: PathBuf,
    pub(crate) messages: Vec<Message>,
    resource: FluentResource,
}

pub(crate) struct Locale {
    pub(crate) name: String,
    pub(crate) paths: Vec<PathBuf>,
}

impl Catalog {
    pub(crate) fn read_dir(dir: &Path) -> Vec<Self> {
        let mut catalogs: Vec<Self> = sorted_entries(dir)
            .iter()
            .filter(|path| path.extension().is_some_and(|ext| ext == "ftl"))
            .map(|path| Self::read(path))
            .collect();
        Message::resolve_references(catalogs.iter_mut().flat_map(|c| &mut c.messages));
        catalogs
    }

    fn read(path: &Path) -> Self {
        let resource = FluentResource::try_new(fs::read_to_string(path).unwrap())
            .unwrap_or_else(|(_, errors)| panic!("{}: {errors:?}", path.display()));
        let stem = path.file_stem().unwrap().to_str().unwrap().to_owned();
        Identifier::Variable.check(path, &stem);
        let messages = resource
            .entries()
            .filter_map(|entry| match entry {
                Entry::Message(message) => Some(Message::read(path, &stem, message)),
                _ => None,
            })
            .collect();
        Self {
            stem,
            path: path.to_owned(),
            messages,
            resource,
        }
    }

    fn check_translation(&self, locale: &str, source: &[Self]) {
        let original = source
            .iter()
            .find(|c| c.stem == self.stem)
            .unwrap_or_else(|| {
                panic!(
                    "{locale}: {} has no {SOURCE_LOCALE} counterpart",
                    self.path.display()
                )
            });
        for message in &self.messages {
            message.check_translation(locale, &original.messages);
        }
    }
}

impl Locale {
    /// Duplicate messages or terms fail here instead of at launch.
    pub(crate) fn read(dir: &Path, source: &[Catalog]) -> Self {
        let name = dir.file_name().unwrap().to_str().unwrap().to_owned();
        let translated;
        let catalogs = if name == SOURCE_LOCALE {
            source
        } else {
            translated = Catalog::read_dir(dir);
            for catalog in &translated {
                catalog.check_translation(&name, source);
            }
            &translated
        };
        let language = name
            .parse()
            .unwrap_or_else(|_| panic!("locales/{name} is not a language tag"));
        let mut bundle = FluentBundle::new(vec![language]);
        bundle.add_builtins().unwrap();
        for catalog in catalogs {
            bundle
                .add_resource(&catalog.resource)
                .unwrap_or_else(|errors| panic!("{}: {errors:?}", catalog.path.display()));
        }
        let paths = catalogs.iter().map(|c| c.path.clone()).collect();
        Self { name, paths }
    }
}

pub(crate) fn sorted_entries(dir: &Path) -> Vec<PathBuf> {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    entries.sort();
    entries
}
