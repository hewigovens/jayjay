#![cfg(unix)]

use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use chrono::DateTime;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tempfile::TempDir;
use ureq::Agent;

const APPCAST: &[u8] = br#"<?xml version="1.0"?><rss><channel>JayJay</channel></rss>"#;
const IP: &str = "203.0.113.1";
const WRANGLER_ENV: [(&str, &str); 3] = [
    ("CI", "1"),
    ("WRANGLER_SEND_METRICS", "false"),
    ("NO_COLOR", "1"),
];

#[derive(Default)]
struct Origin {
    status: AtomicU16,
    disconnect: AtomicBool,
}

impl Origin {
    fn serve() -> (Arc<Self>, u16) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let origin = Arc::new(Self::default());
        let shared = origin.clone();
        thread::spawn(move || {
            for mut stream in listener.incoming().flatten() {
                let mut request = [0; 4096];
                let _ = stream.read(&mut request);
                if shared.disconnect.load(Ordering::SeqCst) {
                    continue;
                }
                let head = format!(
                    "HTTP/1.1 {} Origin\r\nContent-Type: text/plain\r\nX-Origin-Header: not-forwarded\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    shared.status.load(Ordering::SeqCst),
                    APPCAST.len()
                );
                let _ = stream.write_all(&[head.as_bytes(), APPCAST].concat());
            }
        });
        (origin, port)
    }
}

struct Response {
    status: u16,
    content_type: Option<String>,
    origin_header: bool,
    body: Vec<u8>,
}

struct LocalWorker {
    url: String,
    state: TempDir,
    origin: Arc<Origin>,
    agent: Agent,
    process: Child,
}

impl LocalWorker {
    fn start() -> Self {
        let state = TempDir::with_prefix("jayjay-worker-test-").unwrap();
        wrangler(
            state.path(),
            &["d1", "migrations", "apply", "DB", "--local"],
        );
        let (origin, origin_port) = Origin::serve();
        let port = TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let log_path = state.path().join("wrangler.log");
        let log = File::create(&log_path).unwrap();
        let process = Command::new("wrangler")
            .args([
                "dev",
                "--local",
                "--ip",
                "127.0.0.1",
                "--port",
                &port.to_string(),
                "--inspector-port",
                "0",
            ])
            .args(["--persist-to".as_ref(), state.path().as_os_str()])
            .args([
                "--show-interactive-dev-session=false",
                "--var",
                "HASH_SECRET:secret",
                "--var",
            ])
            .arg(format!(
                "APPCAST_ORIGIN:http://127.0.0.1:{origin_port}/appcast.xml"
            ))
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .envs(WRANGLER_ENV)
            .stdin(Stdio::null())
            .stdout(log.try_clone().unwrap())
            .stderr(log)
            .process_group(0)
            .spawn()
            .unwrap();
        let agent = Agent::new_with_config(
            Agent::config_builder()
                .http_status_as_error(false)
                .timeout_global(Some(Duration::from_secs(10)))
                .build(),
        );
        let mut worker = Self {
            url: format!("http://127.0.0.1:{port}"),
            state,
            origin,
            agent,
            process,
        };
        let deadline = Instant::now() + Duration::from_secs(300);
        while worker.process.try_wait().unwrap().is_none() && Instant::now() < deadline {
            if worker
                .agent
                .get(format!("{}/ping?probe=1", worker.url))
                .call()
                .is_ok_and(|response| response.status() == 200)
            {
                return worker;
            }
            thread::sleep(Duration::from_millis(200));
        }
        panic!(
            "local Worker failed to start:\n{}",
            std::fs::read_to_string(log_path).unwrap_or_default()
        );
    }

    fn sql(&self, query: &str) -> Value {
        let output = wrangler(
            self.state.path(),
            &[
                "d1",
                "execute",
                "DB",
                "--local",
                "--json",
                "--command",
                query,
            ],
        );
        serde_json::from_str::<Value>(&output).unwrap()[0]["results"].take()
    }

    fn request(&self, method: &str, path: &str) -> Response {
        let request = ureq::http::Request::builder()
            .method(method)
            .uri(format!("{}{path}", self.url))
            .header("CF-Connecting-IP", IP)
            .header("Accept-Encoding", "identity")
            .body(())
            .unwrap();
        let mut response = self.agent.run(request).unwrap();
        let header = |name| {
            response
                .headers()
                .get(name)
                .map(|value| value.to_str().unwrap().to_owned())
        };
        Response {
            status: response.status().as_u16(),
            content_type: header("content-type"),
            origin_header: header("x-origin-header").is_some(),
            body: response.body_mut().read_to_vec().unwrap(),
        }
    }

    fn assert_response(&self, method: &str, path: &str, status: u16, body: &[u8]) {
        let response = self.request(method, path);
        assert_eq!(
            (response.status, response.body.as_slice()),
            (status, body),
            "{method} {path}"
        );
    }
}

impl Drop for LocalWorker {
    fn drop(&mut self) {
        let group = -(self.process.id() as i32);
        unsafe { libc::kill(group, libc::SIGTERM) };
        let deadline = Instant::now() + Duration::from_secs(10);
        while self.process.try_wait().ok().flatten().is_none() {
            if Instant::now() > deadline {
                unsafe { libc::kill(group, libc::SIGKILL) };
                let _ = self.process.wait();
                break;
            }
            thread::sleep(Duration::from_millis(100));
        }
    }
}

fn wrangler(state: &Path, args: &[&str]) -> String {
    let output = Command::new("wrangler")
        .args(args)
        .args(["--persist-to".as_ref(), state.as_os_str()])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .envs(WRANGLER_ENV)
        .stdin(Stdio::null())
        .output()
        .expect("wrangler must be on PATH");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        output.status.success(),
        "wrangler {args:?} failed:\n{stdout}{}",
        String::from_utf8_lossy(&output.stderr)
    );
    stdout
}

fn client_ping_writes_d1(worker: &LocalWorker) {
    let response = worker.request(
        "GET",
        &format!("/ping?version=0.3.1&build=42&platform=gpui&os=linux&osver=6.12&arch=x86_64&daily_id={}&monthly_id={}", "a".repeat(64), "b".repeat(64)),
    );
    assert_eq!(
        (
            response.status,
            response.content_type,
            response.body.as_slice()
        ),
        (200, None, b"ok".as_slice())
    );
    assert_eq!(
        worker.sql("SELECT unique_key, monthly_key, identity_kind, channel, version, build, os_version FROM pings"),
        json!([{
            "unique_key": "a".repeat(64), "monthly_key": "b".repeat(64), "identity_kind": "client",
            "channel": "gpui", "version": "0.3.1", "build": "42", "os_version": "6.12",
        }])
    );
}

fn legacy_appcast_routes_and_hashes(worker: &LocalWorker) {
    for path in ["/appcast.xml", "/", "/older-client-path"] {
        let response = worker.request(
            "GET",
            &format!("{path}?appVersionShort=0.3.1&osVersion=15.5&cputype=16777228"),
        );
        assert_eq!(
            (response.status, response.body.as_slice()),
            (200, APPCAST),
            "{path}"
        );
        assert_eq!(
            response.content_type.as_deref(),
            Some("application/xml; charset=utf-8")
        );
        assert!(!response.origin_header);
    }
    let rows = worker.sql("SELECT day, ts, unique_key, monthly_key, identity_kind, channel, platform, os, os_version, arch FROM pings");
    let rows = rows.as_array().unwrap();
    assert_eq!(rows.len(), 3);
    let key = |period: String| hex::encode(&Sha256::digest(format!("{IP}|{period}|secret"))[..12]);
    for row in rows {
        let month = DateTime::from_timestamp(row["ts"].as_i64().unwrap(), 0)
            .unwrap()
            .format("%Y-%m");
        assert_eq!(row["unique_key"], key(row["day"].to_string()));
        assert_eq!(row["monthly_key"], key(month.to_string()));
        assert_eq!(
            [
                &row["identity_kind"],
                &row["channel"],
                &row["platform"],
                &row["os"],
                &row["os_version"],
                &row["arch"]
            ],
            ["network", "swiftui", "macos", "macos", "15.5", "arm64"]
        );
    }
}

fn probes_and_non_releases_do_not_write(worker: &LocalWorker) {
    for query in [
        "probe=1&version=0.3.1",
        "version=0.3.1-dev",
        "",
        "version=unknown",
    ] {
        assert_eq!(worker.request("GET", &format!("/ping?{query}")).status, 200);
        assert_eq!(
            worker.request("GET", &format!("/appcast.xml?{query}")).body,
            APPCAST
        );
    }
    assert_eq!(
        worker.sql("SELECT COUNT(*) AS count FROM pings"),
        json!([{ "count": 0 }])
    );
}

fn head_and_post_keep_existing_routes(worker: &LocalWorker) {
    for (path, body) in [
        ("/ping?probe=1", b"ok".as_slice()),
        ("/appcast.xml?probe=1", APPCAST),
    ] {
        worker.assert_response("HEAD", path, 200, b"");
        worker.assert_response("POST", path, 200, body);
    }
}

fn upstream_error_status_keeps_legacy_proxy_response(worker: &LocalWorker) {
    worker.origin.status.store(503, Ordering::SeqCst);
    worker.assert_response("GET", "/appcast.xml?probe=1", 200, APPCAST);
}

fn upstream_connection_failure_returns_bad_gateway(worker: &LocalWorker) {
    worker.origin.disconnect.store(true, Ordering::SeqCst);
    worker.assert_response("GET", "/appcast.xml?probe=1", 502, b"");
}

fn telemetry_failure_does_not_break_responses(worker: &LocalWorker) {
    worker.sql("ALTER TABLE pings RENAME TO unavailable_pings");
    worker.assert_response("GET", "/ping?version=0.3.1", 200, b"ok");
    worker.assert_response("GET", "/appcast.xml?appVersionShort=0.3.1", 200, APPCAST);
    worker.sql("ALTER TABLE unavailable_pings RENAME TO pings");
}

#[test]
#[ignore = "builds the Worker and serves it with wrangler dev"]
fn local_worker_and_d1() {
    let worker = LocalWorker::start();
    let cases: [fn(&LocalWorker); 7] = [
        client_ping_writes_d1,
        legacy_appcast_routes_and_hashes,
        probes_and_non_releases_do_not_write,
        head_and_post_keep_existing_routes,
        upstream_error_status_keeps_legacy_proxy_response,
        upstream_connection_failure_returns_bad_gateway,
        telemetry_failure_does_not_break_responses,
    ];
    for case in cases {
        worker.sql("DELETE FROM pings");
        worker.origin.status.store(200, Ordering::SeqCst);
        worker.origin.disconnect.store(false, Ordering::SeqCst);
        case(&worker);
    }
}
