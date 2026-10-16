//! Counts moved code over a git history: move_history <git-dir> [revision] [count].
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::time::Instant;

use jj_diff::{DiffSpanStyle, compute_file_diff_full};

#[derive(Default)]
struct Tally {
    files: usize,
    changed_rows: usize,
    moved_rows: usize,
    blocks: usize,
    millis: Vec<f64>,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let repo = args
        .get(1)
        .expect("usage: move_history <git-dir> [revision] [count]");
    let revision = args.get(2).map_or("HEAD", String::as_str);
    let count = args.get(3).map_or("300", String::as_str);
    let commits = git(
        repo,
        &["log", "--no-merges", "--format=%H", "-n", count, revision],
    );
    let mut blobs = Blobs::new(repo);
    let mut tallies: BTreeMap<String, Tally> = BTreeMap::new();
    for commit in commits.lines() {
        let changes = git(
            repo,
            &[
                "diff-tree",
                "-r",
                "-M",
                "--no-commit-id",
                "--name-status",
                commit,
            ],
        );
        for change in changes.lines() {
            let fields: Vec<&str> = change.split('\t').collect();
            let (old_path, path) = match fields.as_slice() {
                [status, path] if status.starts_with('M') => (*path, *path),
                [status, old, new] if status.starts_with('R') && !status.ends_with("100") => {
                    (*old, *new)
                }
                _ => continue,
            };
            let (Some(old), Some(new)) = (
                blobs.read(&format!("{commit}^:{old_path}")),
                blobs.read(&format!("{commit}:{path}")),
            ) else {
                continue;
            };
            let mut diff = compute_file_diff_full(path, &old, &new, false);
            let start = Instant::now();
            diff.detect_moves(&old, &new);
            let tally = tallies.entry(diff.language.clone()).or_default();
            tally.millis.push(start.elapsed().as_secs_f64() * 1000.);
            tally.files += 1;
            for line in diff.lines.iter().filter(|line| line.is_changed()) {
                tally.changed_rows += 1;
                if let Some(moved) = line.moved.filter(|_| line.style == DiffSpanStyle::Removed) {
                    tally.moved_rows += 1;
                    tally.blocks += usize::from(moved.starts_block);
                }
            }
        }
    }
    println!(
        "{:<11} {:>6} {:>8} {:>7} {:>7} {:>7} {:>7}",
        "language", "files", "changed", "moved", "blocks", "p50_ms", "max_ms"
    );
    let mut total = Tally::default();
    for (language, tally) in &tallies {
        row(language, tally);
        total.files += tally.files;
        total.changed_rows += tally.changed_rows;
        total.moved_rows += tally.moved_rows;
        total.blocks += tally.blocks;
        total.millis.extend(&tally.millis);
    }
    row("TOTAL", &total);
}

fn row(language: &str, tally: &Tally) {
    let mut millis = tally.millis.clone();
    millis.sort_by(f64::total_cmp);
    let p50 = millis.get(millis.len() / 2).copied().unwrap_or(0.);
    let max = millis.last().copied().unwrap_or(0.);
    println!(
        "{language:<11} {:>6} {:>8} {:>7} {:>7} {p50:>7.2} {max:>7.1}",
        tally.files, tally.changed_rows, tally.moved_rows, tally.blocks
    );
}

fn git(repo: &str, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .expect("git");
    String::from_utf8_lossy(&output.stdout).into_owned()
}

struct Blobs {
    child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
}

impl Drop for Blobs {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Blobs {
    fn new(repo: &str) -> Self {
        let mut child = Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(["cat-file", "--batch"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("git cat-file");
        Self {
            input: child.stdin.take().unwrap(),
            output: BufReader::new(child.stdout.take().unwrap()),
            child,
        }
    }

    fn read(&mut self, object: &str) -> Option<String> {
        writeln!(self.input, "{object}").ok()?;
        self.input.flush().ok()?;
        let mut header = String::new();
        self.output.read_line(&mut header).ok()?;
        let size: usize = header.split_whitespace().nth(2)?.parse().ok()?;
        let mut body = vec![0; size + 1];
        self.output.read_exact(&mut body).ok()?;
        body.pop();
        String::from_utf8(body)
            .ok()
            .filter(|text| !text.contains('\0'))
    }
}
