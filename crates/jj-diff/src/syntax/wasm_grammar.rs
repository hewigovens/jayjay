use std::io::Read;
use std::sync::{LazyLock, Mutex, OnceLock, PoisonError};

use flate2::read::GzDecoder;
use tree_sitter::wasmtime::{Config, Engine};
use tree_sitter::{Language, Parser, WasmStore};
use tree_sitter_highlight::Highlighter;

use super::Grammar;

// Pulley interprets instead of compiling to machine code, which the hardened runtime only allows with the allow-unsigned-executable-memory entitlement.
static ENGINE: LazyLock<Option<Engine>> = LazyLock::new(|| {
    let mut config = Config::new();
    config.target("pulley64").ok()?;
    Engine::new(&config).ok()
});

// A store instantiates each grammar once, so parsers borrow idle stores instead of paying for that on every parse.
static IDLE_STORES: Mutex<Vec<WasmStore>> = Mutex::new(Vec::new());

pub(crate) struct WasmGrammar {
    name: &'static str,
    gzipped: &'static [u8],
    language: OnceLock<Option<Language>>,
}

impl WasmGrammar {
    pub(crate) const fn new(name: &'static str, gzipped: &'static [u8]) -> Self {
        Self {
            name,
            gzipped,
            language: OnceLock::new(),
        }
    }

    fn load(&self) -> Option<Language> {
        let mut wasm = Vec::new();
        GzDecoder::new(self.gzipped).read_to_end(&mut wasm).ok()?;
        let mut store = borrow_store()?;
        let language = store.load_language(self.name, &wasm).ok();
        return_store(store);
        language
    }
}

impl Grammar for WasmGrammar {
    fn language(&self) -> Option<Language> {
        self.language.get_or_init(|| self.load()).clone()
    }
}

pub(crate) trait ParserOwner {
    fn parser_mut(&mut self) -> &mut Parser;
}

impl ParserOwner for Parser {
    fn parser_mut(&mut self) -> &mut Parser {
        self
    }
}

impl ParserOwner for Highlighter {
    fn parser_mut(&mut self) -> &mut Parser {
        &mut self.parser
    }
}

/// A parser cannot use a WebAssembly language without a store, so this lends one for the duration of `run`.
pub(crate) fn with_wasm_store<T: ParserOwner, R>(
    owner: &mut T,
    language: &Language,
    run: impl FnOnce(&mut T) -> R,
) -> R {
    let Some(store) = language.is_wasm().then(borrow_store).flatten() else {
        return run(owner);
    };
    if owner.parser_mut().set_wasm_store(store).is_err() {
        return run(owner);
    }
    let result = run(owner);
    if let Some(store) = owner.parser_mut().take_wasm_store() {
        return_store(store);
    }
    result
}

fn borrow_store() -> Option<WasmStore> {
    let idle = IDLE_STORES
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .pop();
    idle.or_else(|| WasmStore::new(ENGINE.as_ref()?).ok())
}

fn return_store(store: WasmStore) {
    IDLE_STORES
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .push(store);
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::path::Path;

    use flate2::{Compression, write::GzEncoder};

    use super::*;

    #[test]
    fn a_corrupt_grammar_loads_as_none() {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(b"\0asm not a grammar").unwrap();
        let not_wasm: &'static [u8] = encoder.finish().unwrap().leak();
        for gzipped in [b"not gzip".as_slice(), not_wasm] {
            assert!(WasmGrammar::new("kotlin", gzipped).language().is_none());
        }
    }

    #[test]
    fn bundled_grammars_match_their_pinned_sources() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let lock = std::fs::read_to_string(root.join("../../Cargo.lock")).unwrap();
        let locked: Vec<_> = lock.lines().zip(lock.lines().skip(1)).collect();
        let script = std::fs::read_to_string(root.join("../../scripts/wasm-grammars.sh")).unwrap();
        let versions = std::fs::read_to_string(root.join("grammars/wasm/versions")).unwrap();
        for line in versions.lines() {
            let (source, version) = line.split_once(' ').unwrap();
            let pinned = if source.starts_with("https://") {
                script.contains(&format!(" {source}@{version} "))
            } else {
                let entry = (
                    format!("name = \"{source}\""),
                    format!("version = \"{version}\""),
                );
                locked.contains(&(entry.0.as_str(), entry.1.as_str()))
            };
            assert!(
                pinned,
                "{source} {version} is not the pinned version; run `just wasm-grammars`"
            );
        }
    }
}
