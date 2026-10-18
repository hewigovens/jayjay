use super::table::{FILE_EXTENSIONS, FILE_NAMES, INK_BOUNDS, LANGUAGE_IDS};

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
    pub fn for_path(path: &str) -> Option<Self> {
        let name = path.rsplit('/').next().unwrap_or(path).to_ascii_lowercase();
        lookup(FILE_NAMES, &name)
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
            .map(Self)
    }

    /// Left bearing and width of the ink as fractions of the font size; Seti pads each glyph to a full em.
    pub fn ink(self) -> (f32, f32) {
        INK_BOUNDS
            .binary_search_by_key(&self.0, |(glyph, _, _)| *glyph)
            .map_or((0., 1.), |index| (INK_BOUNDS[index].1, INK_BOUNDS[index].2))
    }
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
    use super::super::table::{FILE_EXTENSIONS, FILE_NAMES, INK_BOUNDS, LANGUAGE_IDS};
    use super::*;

    fn glyph_for_language(language: &str) -> Option<SetiGlyph> {
        lookup(LANGUAGE_IDS, language).map(SetiGlyph)
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
            lookup(FILE_NAMES, "readme.md").map(SetiGlyph)
        );
        assert_eq!(
            SetiGlyph::for_path("infra/main.tf.json"),
            lookup(FILE_EXTENSIONS, "tf.json").map(SetiGlyph)
        );
        assert_ne!(
            SetiGlyph::for_path("infra/main.tf.json"),
            glyph_for_language("json")
        );
        assert_eq!(SetiGlyph::for_path("NOTES"), None);
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
        assert!(INK_BOUNDS.windows(2).all(|pair| pair[0].0 < pair[1].0));
    }

    #[test]
    fn every_mapped_glyph_has_ink_bounds() {
        for (_, glyph) in FILE_NAMES.iter().chain(FILE_EXTENSIONS).chain(LANGUAGE_IDS) {
            assert!(
                INK_BOUNDS.iter().any(|(bounded, _, _)| bounded == glyph),
                "{glyph:?}"
            );
        }
    }

    #[test]
    fn every_language_extension_names_a_seti_language() {
        for (extension, language) in LANGUAGE_EXTENSIONS.iter().chain(LANGUAGE_NAMES) {
            assert!(
                glyph_for_language(language).is_some(),
                "{extension} -> {language}"
            );
        }
    }
}
