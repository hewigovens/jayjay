import type { ReviewBundle } from "@jayjay/review-web/types";

const greeting = {
  path: "src/greeting.rs",
  before: `use std::env;

/// A small greeting for the command line.
fn greeting(name: &str) -> String {
    format!("Hello, {}!", name)
}

fn main() {
    let name = env::args().nth(1);
    let message = greeting(name.as_deref().unwrap_or("world"));
    println!("{}", message);
}
`,
  after: `use std::env;

/// A small greeting, wherever you run it.
fn greeting(name: &str) -> String {
    let name = name.trim();
    if name.is_empty() {
        return "Hello, world!".to_owned();
    }
    format!("Hello, {name}!")
}

fn main() {
    let name = env::args().nth(1);
    let message = greeting(name.as_deref().unwrap_or("world"));
    println!("{message}");
}
`,
};

/** The change the test host opens before anything is imported. */
export const fixture: ReviewBundle = {
  schemaVersion: 1,
  repositoryId: "demo/jayjay",
  changeId: "control-room-preview",
  title: "Bring review into the browser",
  description: "A local review workspace powered by JayJay’s diff engine, fingerprints, and note reconciliation.",
  author: "JayJay",
  files: [
    { path: greeting.path, reviewIdentity: "demo-greeting-v1", before: greeting.before, after: greeting.after },
    { path: "src/web/session.ts", reviewIdentity: "demo-session-v1", before: "export async function load() {\n  const change = await loadChange();\n  const notes = await loadNotes();\n  return { change, notes };\n}\n", after: "export async function load() {\n  const [change, notes] = await Promise.all([\n    loadChange(),\n    loadNotes(),\n  ]);\n  return { change, notes };\n}\n" },
    { path: "config/settings.json", reviewIdentity: "demo-config-v1", before: '{\n  "review": false,\n  "theme": "system"\n}\n', after: '{\n  "review": true,\n  "theme": "system",\n  "offline": true\n}\n' },
    { path: "README.md", reviewIdentity: "demo-readme-v1", before: "# Control room\n\nA workspace for development.\n", after: "# Control room\n\nA workspace for development and review.\n\n- Review changes in your browser.\n- Keep marks and notes on your device.\n" },
    { path: "Cargo.toml", reviewIdentity: "demo-manifest-v1", before: '[dependencies]\njj-diff = "0.1"\n', after: '[dependencies]\njj-diff = "0.1"\njayjay-review = { path = "crates/jayjay-review" }\n' },
  ],
};
