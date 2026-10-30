use super::table::{
    ARCHIVE, CONFIG, DATA, DEFAULT, FILE_EXTENSIONS, FILE_NAMES, GLYPH_METRICS, IMAGE,
    LANGUAGE_IDS, LOCKFILE, NOTEBOOK, SHELL, TABLE,
};
use crate::file_kind::FileKind;

const LANGUAGE_EXTENSIONS: &[(&str, &str)] = &[
    ("ad", "argdown"),
    ("argdown", "argdown"),
    ("bash", "shellscript"),
    ("bat", "bat"),
    ("bicep", "bicep"),
    ("c", "c"),
    ("c++", "cpp"),
    ("cake", "csharp"),
    ("cc", "cpp"),
    ("cjs", "javascript"),
    ("clj", "clojure"),
    ("cljc", "clojure"),
    ("cljs", "clojure"),
    ("cmd", "bat"),
    ("coffee", "coffeescript"),
    ("cpp", "cpp"),
    ("cs", "csharp"),
    ("cshtml", "razor"),
    ("css", "css"),
    ("csx", "csharp"),
    ("cts", "typescript"),
    ("cu", "cuda-cpp"),
    ("cuh", "cuda-cpp"),
    ("cxx", "cpp"),
    ("dart", "dart"),
    ("edn", "clojure"),
    ("elm", "elm"),
    ("erb", "erb"),
    ("ex", "elixir"),
    ("exs", "elixir"),
    ("fs", "fsharp"),
    ("fsi", "fsharp"),
    ("fsx", "fsharp"),
    ("gd", "godot"),
    ("gemspec", "ruby"),
    ("go", "go"),
    ("gradle", "gradle"),
    ("groovy", "groovy"),
    ("h", "c"),
    ("haml", "haml"),
    ("handlebars", "handlebars"),
    ("hbs", "handlebars"),
    ("hh", "cpp"),
    ("hpp", "cpp"),
    ("hs", "haskell"),
    ("htm", "html"),
    ("html", "html"),
    ("hx", "haxe"),
    ("hxx", "cpp"),
    ("j2", "jinja"),
    ("jade", "jade"),
    ("java", "java"),
    ("jinja", "jinja"),
    ("jl", "julia"),
    ("js", "javascript"),
    ("json", "json"),
    ("jsonc", "jsonc"),
    ("jsonl", "jsonl"),
    ("jsx", "javascriptreact"),
    ("kt", "kotlin"),
    ("kts", "kotlin"),
    ("less", "less"),
    ("lhs", "haskell"),
    ("lua", "lua"),
    ("m", "objective-c"),
    ("mak", "makefile"),
    ("markdown", "markdown"),
    ("md", "markdown"),
    ("mjs", "javascript"),
    ("mk", "makefile"),
    ("ml", "ocaml"),
    ("mli", "ocaml"),
    ("mm", "objective-cpp"),
    ("mts", "typescript"),
    ("mustache", "mustache"),
    ("ndjson", "jsonl"),
    ("njk", "nunjucks"),
    ("php", "php"),
    ("phtml", "php"),
    ("pl", "perl"),
    ("pm", "perl"),
    ("properties", "properties"),
    ("ps1", "powershell"),
    ("psd1", "powershell"),
    ("psm1", "powershell"),
    ("pug", "jade"),
    ("py", "python"),
    ("pyi", "python"),
    ("pyw", "python"),
    ("r", "r"),
    ("rake", "ruby"),
    ("razor", "razor"),
    ("rb", "ruby"),
    ("res", "rescript"),
    ("resi", "rescript"),
    ("rs", "rust"),
    ("sass", "sass"),
    ("scss", "scss"),
    ("sh", "shellscript"),
    ("sql", "sql"),
    ("styl", "stylus"),
    ("swift", "swift"),
    ("tex", "tex"),
    ("tf", "terraform"),
    ("tfvars", "terraform"),
    ("ts", "typescript"),
    ("tsx", "typescriptreact"),
    ("vala", "vala"),
    ("vue", "vue"),
    ("xml", "xml"),
    ("yaml", "yaml"),
    ("yml", "yaml"),
    ("zsh", "shellscript"),
];

const LANGUAGE_NAMES: &[(&str, &str)] = &[
    ("commit_editmsg", "git-commit"),
    ("compose.yaml", "dockercompose"),
    ("compose.yml", "dockercompose"),
    ("containerfile", "dockerfile"),
    ("docker-compose.yaml", "dockercompose"),
    ("docker-compose.yml", "dockercompose"),
    ("dockerfile", "dockerfile"),
    ("gnumakefile", "makefile"),
    ("makefile", "makefile"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetiGlyph(pub char);

impl SetiGlyph {
    pub fn for_path(path: &str) -> Self {
        let name = path.rsplit('/').next().unwrap_or(path).to_ascii_lowercase();
        let glyph = lookup(FILE_NAMES, &name)
            .or_else(|| {
                name.match_indices('.')
                    .find_map(|(index, _)| lookup(FILE_EXTENSIONS, &name[index + 1..]))
            })
            .or_else(|| {
                let extension = name.rsplit_once('.').map(|(_, extension)| extension);
                let language = lookup(LANGUAGE_NAMES, &name)
                    .or_else(|| dotfile_language(&name))
                    .or_else(|| {
                        extension.and_then(|extension| lookup(LANGUAGE_EXTENSIONS, extension))
                    })?;
                lookup(LANGUAGE_IDS, language)
            })
            .unwrap_or_else(|| match FileKind::for_path(&name) {
                FileKind::Shell => SHELL,
                FileKind::Data => DATA,
                FileKind::Config => CONFIG,
                FileKind::Lockfile => LOCKFILE,
                FileKind::Image => IMAGE,
                FileKind::Table => TABLE,
                FileKind::Notebook => NOTEBOOK,
                FileKind::Archive => ARCHIVE,
                FileKind::Other => DEFAULT,
            });
        Self(glyph)
    }

    pub fn metrics(self) -> SetiMetrics {
        GLYPH_METRICS
            .binary_search_by_key(&self.0, |(glyph, ..)| *glyph)
            .map_or(SetiMetrics::DEFAULT, |index| {
                let (_, ink_left, ink_width, ink_center, scale) = GLYPH_METRICS[index];
                SetiMetrics {
                    scale,
                    ink_left,
                    ink_width,
                    ink_center,
                }
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SetiMetrics {
    pub scale: f32,
    pub ink_left: f32,
    pub ink_width: f32,
    pub ink_center: f32,
}

impl SetiMetrics {
    const DEFAULT: Self = Self {
        scale: 1.,
        ink_left: 0.,
        ink_width: 1.,
        ink_center: 0.5,
    };
}

fn dotfile_language(name: &str) -> Option<&'static str> {
    if !name.starts_with('.') {
        return None;
    }
    if name == ".env" || name.starts_with(".env.") {
        Some("dotenv")
    } else if name.ends_with("ignore") {
        Some("ignore")
    } else {
        None
    }
}

fn lookup<T: Copy>(table: &[(&str, T)], key: &str) -> Option<T> {
    table
        .binary_search_by(|(candidate, _)| (*candidate).cmp(key))
        .ok()
        .map(|index| table[index].1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn glyph_for_language(language: &str) -> SetiGlyph {
        SetiGlyph(lookup(LANGUAGE_IDS, language).expect("Seti language"))
    }

    #[test]
    fn for_path_prefers_names_then_longest_extensions_then_languages() {
        assert_eq!(
            SetiGlyph::for_path("crates/core/src/lib.rs"),
            glyph_for_language("rust")
        );
        assert_eq!(
            SetiGlyph::for_path("App/View.SWIFT"),
            glyph_for_language("swift")
        );
        assert_eq!(
            SetiGlyph::for_path("Makefile"),
            glyph_for_language("makefile")
        );
        assert_eq!(
            SetiGlyph::for_path(".gitignore"),
            glyph_for_language("ignore")
        );
        assert_eq!(
            SetiGlyph::for_path(".env.local"),
            glyph_for_language("dotenv")
        );
        assert_eq!(
            SetiGlyph::for_path("docs/README.md"),
            SetiGlyph(lookup(FILE_NAMES, "readme.md").expect("readme glyph"))
        );
        assert_eq!(
            SetiGlyph::for_path("infra/main.tf.json"),
            SetiGlyph(lookup(FILE_EXTENSIONS, "tf.json").expect("tf.json glyph"))
        );
        assert_ne!(
            SetiGlyph::for_path("infra/main.tf.json"),
            glyph_for_language("json")
        );
        assert_eq!(SetiGlyph::for_path("Cargo.lock"), SetiGlyph(LOCKFILE));
        assert_eq!(SetiGlyph::for_path("hooks/setup.fish"), SetiGlyph(SHELL));
        assert_eq!(SetiGlyph::for_path("NOTES"), SetiGlyph(DEFAULT));
    }

    #[test]
    fn tables_are_sorted_for_binary_search() {
        for keys in [
            FILE_NAMES.iter().map(|(key, _)| *key).collect::<Vec<_>>(),
            FILE_EXTENSIONS.iter().map(|(key, _)| *key).collect(),
            LANGUAGE_IDS.iter().map(|(key, _)| *key).collect(),
            LANGUAGE_EXTENSIONS.iter().map(|(key, _)| *key).collect(),
            LANGUAGE_NAMES.iter().map(|(key, _)| *key).collect(),
        ] {
            assert!(keys.windows(2).all(|pair| pair[0] < pair[1]), "{keys:?}");
        }
        assert!(GLYPH_METRICS.windows(2).all(|pair| pair[0].0 < pair[1].0));
    }

    #[test]
    fn every_mapped_glyph_has_metrics() {
        let kinds = [
            DEFAULT, SHELL, DATA, CONFIG, LOCKFILE, IMAGE, TABLE, NOTEBOOK, ARCHIVE,
        ];
        let mapped = FILE_NAMES.iter().chain(FILE_EXTENSIONS).chain(LANGUAGE_IDS);
        for glyph in mapped.map(|(_, glyph)| glyph).chain(&kinds) {
            assert!(
                GLYPH_METRICS.iter().any(|(measured, ..)| measured == glyph),
                "{glyph:?}"
            );
        }
    }

    #[test]
    fn every_language_extension_names_a_seti_language() {
        for (extension, language) in LANGUAGE_EXTENSIONS.iter().chain(LANGUAGE_NAMES) {
            assert!(
                lookup(LANGUAGE_IDS, language).is_some(),
                "{extension} -> {language}"
            );
        }
    }
}
