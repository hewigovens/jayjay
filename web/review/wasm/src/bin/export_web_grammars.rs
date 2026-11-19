#[cfg(not(target_family = "wasm"))]
use std::{collections::BTreeSet, env, error::Error, fs, path::PathBuf};

#[cfg(not(target_family = "wasm"))]
use serde_json::{Map, json};

// The language table names native grammar crates, so this build tool only runs natively.
#[cfg(target_family = "wasm")]
fn main() {}

/// Writes `grammars.json`, with the C library functions grammars may import and each language's grammar file, and one finished highlight query per language, then prints the grammar files they need.
#[cfg(not(target_family = "wasm"))]
fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let output = PathBuf::from(args.next().ok_or("expected output directory")?);
    let imports = fs::read_to_string(args.next().ok_or("expected tree-sitter's imports.txt")?)?;
    let imports: Vec<&str> = imports
        .lines()
        .map(|line| line.trim().trim_end_matches(',').trim_matches('"'))
        .filter(|line| !line.is_empty())
        .collect();
    let queries = output.join("queries");
    if queries.exists() {
        fs::remove_dir_all(&queries)?;
    }
    fs::create_dir_all(&queries)?;
    let mut languages = Map::new();
    let mut files = BTreeSet::new();
    for grammar in jj_diff::syntax::web_grammars() {
        fs::write(
            queries.join(format!("{}.scm", grammar.language)),
            &grammar.query,
        )?;
        languages.insert(
            grammar.language.to_owned(),
            json!({ "grammar": grammar.grammar, "requires": grammar.requires }),
        );
        files.insert(grammar.grammar);
    }
    fs::write(
        output.join("grammars.json"),
        serde_json::to_string_pretty(&json!({ "imports": imports, "languages": languages }))?,
    )?;
    for file in files {
        println!("{file}");
    }
    Ok(())
}
