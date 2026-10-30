use super::host_path::{to_native, to_wire};
use super::protocol::{Hello, PROTOCOL, Request, Response, read_frame, write_frame};
use super::{RemoteDirectory, RemoteFolder};
use crate::repo::Repo;
use crate::{JayError, JayResult, SyncToken};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Serialize, Deserialize)]
pub(crate) struct OpenedRepo {
    pub root: String,
    pub primary: String,
    pub store: String,
    pub workspace: String,
    pub operation: String,
}

pub fn serve() -> JayResult<()> {
    serve_stream(BufReader::new(std::io::stdin()), std::io::stdout())
}

fn serve_stream(
    reader: impl std::io::BufRead,
    writer: impl Write + Send + 'static,
) -> JayResult<()> {
    let home = crate::home_dir()
        .ok_or_else(|| JayError::internal("The SSH user's home directory is unavailable."))?;
    let writer = Arc::new(Mutex::new(writer));
    write_frame(
        &mut *writer.lock().unwrap(),
        &Hello {
            protocol: PROTOCOL,
            home: to_wire(&home),
        },
    )?;
    let repositories = Arc::new(Mutex::new(HashMap::<PathBuf, Arc<Repo>>::new()));
    let active = Arc::new(Mutex::new(HashMap::<u64, Arc<Mutex<SyncToken>>>::new()));
    let mut reader = reader;
    while let Some(frame) = read_frame(&mut reader)? {
        let request: Request = serde_json::from_slice(&frame).map_err(JayError::internal)?;
        if request.method == "__cancel" {
            if let Some(token) = active.lock().unwrap().get(&request.id) {
                token.lock().unwrap().cancel();
            }
            continue;
        }
        let sync = Arc::new(Mutex::new(SyncToken::standalone()));
        {
            let mut active = active.lock().unwrap();
            if active.len() >= 128 || active.contains_key(&request.id) {
                return Err(JayError::refused("Too many or duplicate SSH requests."));
            }
            active.insert(request.id, sync.clone());
        }
        let (writer, repositories, active, home) = (
            writer.clone(),
            repositories.clone(),
            active.clone(),
            home.clone(),
        );
        thread::spawn(move || {
            let mut images = HashMap::new();
            let result = execute(&request, &home, &repositories, &sync).and_then(|mut value| {
                super::images::collect(&mut value, &mut images)?;
                Ok(value)
            });
            active.lock().unwrap().remove(&request.id);
            let mut writer = writer.lock().unwrap();
            if let Err(error) = write_frame(
                &mut *writer,
                &Response {
                    id: request.id,
                    result,
                    images,
                },
            ) {
                let _ = write_frame(
                    &mut *writer,
                    &Response {
                        id: request.id,
                        result: Err(error),
                        images: HashMap::new(),
                    },
                );
            }
        });
    }
    for sync in active.lock().unwrap().values() {
        sync.lock().unwrap().cancel();
    }
    while !active.lock().unwrap().is_empty() {
        thread::sleep(std::time::Duration::from_millis(25));
    }
    Ok(())
}

fn execute(
    request: &Request,
    home: &Path,
    repositories: &Mutex<HashMap<PathBuf, Arc<Repo>>>,
    active_sync: &Mutex<SyncToken>,
) -> JayResult<Value> {
    let path = if request.path == "~" {
        home.to_owned()
    } else if let Some(rest) = request.path.strip_prefix("~/") {
        home.join(rest)
    } else {
        to_native(&request.path)
    };
    if !path.is_absolute() || request.path.contains(['\0', '\r', '\n']) {
        return Err(JayError::refused("Choose an absolute remote path."));
    }
    if request.method == "__browse" {
        return serde_json::to_value(browse(&path)?).map_err(JayError::internal);
    }
    if request.method == "__init" {
        crate::init_jj_git_repo(&path)?;
        return Ok(Value::Null);
    }
    let repo = {
        let mut repos = repositories.lock().unwrap();
        if let Some(repo) = repos.get(&path) {
            repo.clone()
        } else {
            if repos.len() >= 256 {
                return Err(JayError::refused(
                    "Too many SSH repository sessions. Reconnect the machine.",
                ));
            }
            let repo = Arc::new(Repo::open(&path)?);
            repos.insert(path, repo.clone());
            repo
        }
    };
    let sync = repo.sync_token();
    {
        let mut previous = active_sync.lock().unwrap();
        if previous.check().is_err() {
            sync.cancel();
        }
        *previous = sync.clone();
    }
    sync.check()?;
    if request.method == "__open" {
        return serde_json::to_value(OpenedRepo {
            root: to_wire(repo.path()),
            primary: to_wire(&repo.primary_root()),
            store: to_wire(repo.repository_store_path()),
            workspace: repo.workspace_name().into(),
            operation: repo.snapshot_operation().unwrap_or_default(),
        })
        .map_err(JayError::internal);
    }
    crate::repository::dispatch(&repo, &request.method, request.args.clone(), &sync)
}

fn browse(path: &Path) -> JayResult<RemoteDirectory> {
    let path = dunce::canonicalize(path).map_err(JayError::internal)?;
    let mut folders = Vec::new();
    for entry in std::fs::read_dir(&path)
        .map_err(JayError::internal)?
        .take(5000)
    {
        let entry = entry.map_err(JayError::internal)?;
        if entry.path().is_dir()
            && let Some(name) = entry.file_name().to_str().filter(|s| !s.starts_with('.'))
        {
            folders.push(RemoteFolder {
                name: name.into(),
                path: to_wire(&entry.path()),
            });
        }
    }
    folders.sort_by_key(|folder| folder.name.to_lowercase());
    Ok(RemoteDirectory {
        parent: path.parent().map(to_wire),
        path: to_wire(&path),
        folders,
    })
}
