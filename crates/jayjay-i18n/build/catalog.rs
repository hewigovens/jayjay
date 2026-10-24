use std::fs;
use std::path::{Path, PathBuf};

use fluent_bundle::{FluentBundle, FluentResource};
use fluent_syntax::ast::Entry;

use crate::SOURCE_LOCALE;
use crate::identifier::Identifier;
use crate::message::Message;

pub(crate) struct Catalog {
    pub(crate) module: Vec<String>,
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
        let mut catalogs = Vec::new();
        Self::collect(dir, &[], &mut catalogs);
        catalogs.sort_by(|a, b| a.module.cmp(&b.module));
        Message::resolve_references(catalogs.iter_mut().flat_map(|c| &mut c.messages));
        catalogs
    }

    fn collect(dir: &Path, parent: &[String], catalogs: &mut Vec<Self>) {
        for path in sorted_entries(dir) {
            let mut module = parent.to_vec();
            if path.is_dir() {
                module.push(path.file_name().unwrap().to_str().unwrap().to_owned());
                Self::collect(&path, &module, catalogs);
            } else if path.extension().is_some_and(|ext| ext == "ftl") {
                module.push(path.file_stem().unwrap().to_str().unwrap().to_owned());
                catalogs.push(Self::read(&path, module));
            }
        }
    }

    fn read(path: &Path, module: Vec<String>) -> Self {
        let resource = FluentResource::try_new(fs::read_to_string(path).unwrap())
            .unwrap_or_else(|(_, errors)| panic!("{}: {errors:?}", path.display()));
        for segment in &module {
            Identifier::Variable.check(path, segment);
        }
        let prefix = module.join("-");
        let messages = resource
            .entries()
            .filter_map(|entry| match entry {
                Entry::Message(message) => Some(Message::read(path, &prefix, message)),
                _ => None,
            })
            .collect();
        Self {
            module,
            path: path.to_owned(),
            messages,
            resource,
        }
    }

    fn check_translation(&self, locale: &str, source: &[Self]) {
        let original = source
            .iter()
            .find(|c| c.module == self.module)
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

pub(crate) enum Visit<'a> {
    Open(usize, &'a str),
    Close(usize),
    Catalog(usize, &'a Catalog),
}

/// `settings.ftl` and `settings/fonts.ftl` share the `settings` module, so files become nested modules.
pub(crate) fn walk<'a>(catalogs: &'a [Catalog], mut visit: impl FnMut(Visit<'a>)) {
    let mut stack: Vec<&'a str> = Vec::new();
    for catalog in catalogs {
        while catalog.module.len() < stack.len()
            || catalog.module.iter().zip(&stack).any(|(a, b)| a != b)
        {
            stack.pop();
            visit(Visit::Close(stack.len()));
        }
        for segment in &catalog.module[stack.len()..] {
            visit(Visit::Open(stack.len(), segment));
            stack.push(segment);
        }
        visit(Visit::Catalog(stack.len(), catalog));
    }
    while stack.pop().is_some() {
        visit(Visit::Close(stack.len()));
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
