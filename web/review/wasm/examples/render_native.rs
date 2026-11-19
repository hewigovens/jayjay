//! Reads `{"path": "source", …}` on stdin and prints each file's `render_diff` JSON against an empty base, for scripts/web-grammar-parity.mjs.
use std::io::Read;

fn main() {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).expect("stdin");
    let samples: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(&input).expect("a JSON object of paths and sources");
    let rendered: serde_json::Map<_, _> = samples
        .into_iter()
        .map(|(path, source)| {
            let source = source.as_str().expect("source text");
            let json = jj_diff_web::render_diff(&path, "", source, false, true, false);
            (path, serde_json::Value::String(json))
        })
        .collect();
    println!("{}", serde_json::Value::Object(rendered));
}
