use super::protocol::{Hello, PROTOCOL, Request, Response, read_frame, write_frame};
use crate::{JayError, JayResult, SyncToken};
use std::collections::HashMap;
use std::io::{BufReader, Read, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU64, Ordering},
    mpsc,
};
use std::thread;
use std::time::{Duration, Instant};

pub(crate) struct Connection {
    child: Mutex<Child>,
    input: Mutex<ChildStdin>,
    pending: Mutex<HashMap<u64, mpsc::Sender<JayResult<Response>>>>,
    next: AtomicU64,
    connected: AtomicBool,
    error: Mutex<Option<String>>,
}

pub(crate) fn ssh(machine: &str) -> Command {
    let mut cmd = crate::repo::subprocess_command("ssh");
    cmd.args([
        "-T",
        "-a",
        "-o",
        "ForwardAgent=no",
        "-o",
        "ClearAllForwardings=yes",
        "-o",
        "RemoteCommand=none",
        "-o",
        "BatchMode=yes",
        "-o",
        "StrictHostKeyChecking=yes",
        "-o",
        "ConnectTimeout=15",
        "-o",
        "ServerAliveInterval=15",
        "-o",
        "ServerAliveCountMax=2",
        "--",
        machine,
    ]);
    cmd
}

pub(crate) fn run(machine: &str, command: &str, bytes: &[u8]) -> JayResult<String> {
    let limit = Duration::from_secs(45 + bytes.len() as u64 / 65_536);
    let mut child = ssh(machine)
        .arg(command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(JayError::internal)?;
    let mut input = child.stdin.take().unwrap();
    let bytes = bytes.to_vec();
    let writer = thread::spawn(move || {
        let _ = input.write_all(&bytes);
    });
    let output = child.stdout.take().unwrap();
    let errors = child.stderr.take().unwrap();
    let stdout = thread::spawn(move || read_setup_output(output));
    let stderr = thread::spawn(move || read_setup_output(errors));
    let start = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().map_err(JayError::internal)? {
            break status;
        }
        if start.elapsed() > limit {
            stop(&mut child);
            return Err(JayError::internal(jayjay_i18n::ssh::error_setup_timeout()));
        }
        thread::sleep(Duration::from_millis(25));
    };
    let _ = writer.join();
    let stdout = stdout
        .join()
        .map_err(|_| JayError::internal("SSH setup reader failed."))??;
    let stderr = stderr
        .join()
        .map_err(|_| JayError::internal("SSH setup reader failed."))??;
    if !status.success() {
        return Err(JayError::internal(format!(
            "SSH: {}",
            String::from_utf8_lossy(&stderr).trim()
        )));
    }
    String::from_utf8(stdout).map_err(JayError::internal)
}

fn read_setup_output(mut reader: impl Read) -> JayResult<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut chunk = [0; 8192];
    loop {
        let n = reader.read(&mut chunk).map_err(JayError::internal)?;
        if n == 0 {
            return Ok(bytes);
        }
        if bytes.len() + n > 1024 * 1024 {
            return Err(JayError::refused(
                "SSH setup output exceeded the size limit.",
            ));
        }
        bytes.extend_from_slice(&chunk[..n]);
    }
}

impl Connection {
    pub fn connect(machine: &str, executable: &str) -> JayResult<Arc<Self>> {
        let mut command = ssh(machine);
        command.arg(format!("exec {executable}"));
        Self::start(command)
    }

    fn start(mut command: Command) -> JayResult<Arc<Self>> {
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(JayError::internal)?;
        let input = child.stdin.take().unwrap();
        let mut output = BufReader::new(child.stdout.take().unwrap());
        let (tx, rx) = mpsc::sync_channel(1);
        let reader = thread::spawn(move || {
            let hello = read_frame(&mut output).and_then(|frame| {
                serde_json::from_slice::<Hello>(
                    &frame.ok_or_else(|| JayError::internal("The SSH helper did not start."))?,
                )
                .map_err(JayError::internal)
            });
            let _ = tx.send((hello, output));
        });
        let mut abort = |error| {
            stop(&mut child);
            Err(error)
        };
        let (hello, mut output) = match rx.recv_timeout(Duration::from_secs(30)) {
            Ok((Ok(hello), output)) => (hello, output),
            Ok((Err(error), _)) => return abort(error),
            Err(_) => return abort(JayError::internal("The SSH helper did not respond.")),
        };
        let _ = reader.join();
        if hello.protocol != PROTOCOL || !Path::new(&hello.home).is_absolute() {
            return abort(JayError::refused(
                jayjay_i18n::ssh::error_incompatible_helper(),
            ));
        }
        let this = Arc::new(Self {
            child: Mutex::new(child),
            input: Mutex::new(input),
            pending: Mutex::default(),
            next: AtomicU64::new(1),
            connected: AtomicBool::new(true),
            error: Mutex::new(None),
        });
        let weak = Arc::downgrade(&this);
        thread::spawn(move || {
            loop {
                let result = read_frame(&mut output).and_then(|frame| {
                    serde_json::from_slice::<Response>(
                        &frame.ok_or_else(|| JayError::internal("SSH connection closed."))?,
                    )
                    .map_err(JayError::internal)
                });
                let Some(connection) = weak.upgrade() else {
                    break;
                };
                match result {
                    Ok(response) => {
                        if let Some(sender) =
                            connection.pending.lock().unwrap().remove(&response.id)
                        {
                            let _ = sender.send(Ok(response));
                        }
                    }
                    Err(error) => {
                        connection.fail(&error.to_string());
                        break;
                    }
                }
            }
        });
        Ok(this)
    }

    pub fn connected(&self) -> bool {
        self.connected.load(Ordering::Acquire)
    }
    pub fn error(&self) -> Option<String> {
        self.error.lock().unwrap().clone()
    }
    pub fn busy(&self) -> bool {
        !self.pending.lock().unwrap().is_empty()
    }
    pub fn close(&self) {
        self.fail(&jayjay_i18n::ssh::error_disconnected());
    }
    fn fail(&self, message: &str) {
        if !self.connected.swap(false, Ordering::AcqRel) {
            return;
        }
        let message = if self.busy() {
            jayjay_i18n::ssh::error_write_uncertain(message)
        } else {
            message.into()
        };
        *self.error.lock().unwrap() = Some(message.clone());
        stop(&mut self.child.lock().unwrap());
        for (_, sender) in self.pending.lock().unwrap().drain() {
            let _ = sender.send(Err(JayError::internal(&message)));
        }
    }

    pub fn request(
        &self,
        path: &str,
        method: &str,
        args: Vec<serde_json::Value>,
        sync: Option<&SyncToken>,
    ) -> JayResult<Response> {
        if let Some(sync) = sync {
            sync.check()?;
        }
        let id = self.next.fetch_add(1, Ordering::Relaxed);
        let (sender, receiver) = mpsc::channel();
        {
            let mut pending = self.pending.lock().unwrap();
            if !self.connected() {
                return Err(JayError::refused(
                    self.error()
                        .unwrap_or_else(jayjay_i18n::ssh::error_disconnected),
                ));
            }
            if pending.len() >= 128 {
                return Err(JayError::refused(jayjay_i18n::ssh::error_busy()));
            }
            pending.insert(id, sender);
        }
        let request = Request {
            id,
            path: path.into(),
            method: method.into(),
            args,
        };
        if let Err(error) = write_frame(&mut *self.input.lock().unwrap(), &request) {
            self.fail(&jayjay_i18n::ssh::error_send(&error.to_string()));
        }
        let mut canceled = false;
        loop {
            match receiver.recv_timeout(Duration::from_millis(50)) {
                Ok(response) => return response,
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return Err(JayError::internal(jayjay_i18n::ssh::error_response_lost()));
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
            }
            if !canceled && sync.is_some_and(|sync| sync.check().is_err()) {
                canceled = true;
                let _ = write_frame(
                    &mut *self.input.lock().unwrap(),
                    &Request {
                        id,
                        path: String::new(),
                        method: "__cancel".into(),
                        args: vec![],
                    },
                );
            }
        }
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        stop(self.child.get_mut().unwrap());
    }
}

fn stop(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

#[cfg(all(test, unix))]
mod tests;
