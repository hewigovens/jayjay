// Renders a sample of every language natively and in the web engine with run-time grammars, and fails on any difference.
// Run after scripts/build-diff-web.sh: node scripts/web-grammar-parity.mjs
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { join } from "node:path";

import { compileGrammar, linkGrammar } from "../web/review/src/local/linkGrammar.ts";

const root = join(import.meta.dirname, "..");
const wasm = join(root, "web/review/src/wasm");
const samples = {
  "src/lib.rs": '// note\npub fn answer(name: &str) -> String {\n    format!("hi {name}")\n}\n',
  "Sources/App.swift":
    '// note\nstruct App { let name = "hi"\n  func run() -> Int { return 1 } }\n',
  "web/app.js": "// note\nexport const greet = (name) => `hi ${name}`;\nclass A extends B {}\n",
  "web/app.jsx": 'export default function App() {\n  return <div className="x">{1}</div>;\n}\n',
  "web/app.ts": "// note\ninterface User { name: string }\nexport function f(u: User): void {}\n",
  "web/App.tsx":
    'export function App(): JSX.Element {\n  return <main className="app">{1}</main>;\n}\n',
  "tool.py": '# note\ndef greet(name: str) -> str:\n    return f"hi {name}"\n',
  "main.go": '// note\npackage main\nfunc main() { fmt.Println("hi", 42) }\n',
  "Main.java": '// note\npublic class Main { static int x = 42; String s = "hi"; }\n',
  "Main.kt": '// note\nfun greet(name: String) = "Hello $name"\nval raw = """a ${1 + 2}"""\n',
  "main.c": '// note\n#include <stdio.h>\nint main(void) { printf("hi %d", 42); return 0; }\n',
  "Program.cs": '// note\npublic class Greeter { string Name = $"Hello {1}"; }\n',
  "main.cpp": "// note\ntemplate <typename T> class Box { T value; };\nint main() { return 0; }\n",
  "greet.rb": "# note\ndef greet(name)\n  <<~TEXT\n    Hello #{name}\n  TEXT\nend\n",
  "run.sh": '# note\nfor f in *.txt; do\n  echo "file $f" && exit 0\ndone\n',
  "config.fish": '# note\nfunction greet\n    echo "hi $argv"\nend\n',
  "app.ftl":
    "# note\nhello = Hello, { $name }!\nitems = { $count ->\n    [one] One\n   *[other] Many\n}\n",
  "index.html": '<!-- note -->\n<div class="x"><p>hi</p></div>\n',
  "style.css": "/* note */\n.a > b { color: #fff; margin: 0 4px; }\n",
  "data.json": '{ "name": "hi", "count": 42, "ok": true }\n',
  "index.php": "<?php\n// note\nfunction greet($name) { return <<<EOT\nHello {$name}\nEOT; }\n",
  "Cargo.toml": '# note\n[package]\nname = "x"\nversion = 1\n',
  "ci.yml": "# note\non: push\njobs: { build: { runs-on: ubuntu } }\n",
  "README.md":
    "# Title\n\nSome *emphasis*, `code`, and [a link](https://example.com).\n\n```rust\nfn x() {}\n```\n",
  "query.sql": "-- note\nSELECT name FROM users WHERE id = 'a' LIMIT 10;\n",
  "Info.plist": '<?xml version="1.0"?>\n<!-- note -->\n<plist><key>Name</key></plist>\n',
  "build.ps1": '# note\nfunction Get-Hi { param($Name) Write-Host "hi $Name" }\n',
  "build.bat": "@echo off\nREM note\nset NAME=hi\necho %NAME%\n",
  "settings.ini": "; note\n[section]\nkey = value\n",
  "CMakeLists.txt": "# note\ncmake_minimum_required(VERSION 3.20)\nproject(x LANGUAGES C)\n",
  "Token.sol": "// note\ncontract Token { uint256 public total = 42; function f() public {} }\n",
  "main.zig": "// note\npub fn main() void { const x: u32 = 42; _ = x; }\n",
  "flake.nix": '# note\n{ pkgs, ... }: { name = "x"; value = 42; }\n',
  "init.lua": '-- note\nlocal function greet(name) return "hi " .. name end\n',
  Makefile: "# note\nbuild: main.o\n\tcc -o app main.o\n",
  "main.dart": '// note\nclass Greeter { final String name = "Hello"; }\n',
  Dockerfile: '# note\nFROM rust:1.90 AS build\nENTRYPOINT ["/app"]\n',
  justfile: '# note\nset shell := ["bash", "-c"]\nbuild:\n  cargo build\n',
  "api.proto": '// note\nsyntax = "proto3";\nmessage User { string name = 1; }\n',
  "analysis.R": '# note\nscore <- function(x) if (x > 0) "up" else "down"\n',
};

const native = JSON.parse(
  execFileSync(
    "cargo",
    ["run", "--locked", "--quiet", "-p", "jj-diff-web", "--example", "render_native"],
    {
      cwd: root,
      input: JSON.stringify(samples),
      maxBuffer: 1 << 26,
      stdio: ["pipe", "pipe", "inherit"],
    },
  ).toString(),
);

const engine = await import(join(wasm, "engine/jj_diff.js"));
await engine.default({
  module_or_path: readFileSync(join(wasm, "engine/jj_diff_bg.wasm")),
});
const manifest = JSON.parse(readFileSync(join(wasm, "grammars.json"), "utf8"));
const linked = new Map();
const register = async (language) => {
  const { grammar, requires } = manifest.languages[language];
  for (const required of requires) await register(required);
  if (!linked.has(grammar)) {
    const bytes = readFileSync(join(wasm, "grammars", `${grammar}.wasm`));
    linked.set(grammar, linkGrammar(engine, await compileGrammar(bytes), manifest.imports));
  }
  const query = readFileSync(join(wasm, "queries", `${language}.scm`), "utf8");
  engine.register_grammar(language, linked.get(grammar), query);
};

const covered = new Set();
let failures = 0;
for (const [path, source] of Object.entries(samples)) {
  const language = engine.language_for(path, source);
  if (manifest.languages[language]) await register(language);
  const web = engine.render_diff(path, "", source, false, true, false);
  const highlighted = JSON.parse(web).rows.some((row) =>
    row.cells.some((cell) => cell?.spans.some((span) => span.token !== "Plain")),
  );
  covered.add(language);
  const same = web === native[path] && highlighted;
  if (!same) failures++;
  console.log(`${same ? "same" : "DIFFERENT"}  ${language.padEnd(12)} ${path}`);
}
const missing = Object.keys(manifest.languages).filter(
  (language) => !covered.has(language) && !["jsx", "bash", "markdown_inline"].includes(language),
);
if (missing.length) {
  failures++;
  console.log(`no sample for: ${missing.join(", ")}`);
}
console.log(failures ? `${failures} failed` : `all ${covered.size} languages match native output`);
process.exitCode = failures ? 1 : 0;
