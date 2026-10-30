use super::machine::{MachineConnection, request};
use super::server::OpenedRepo;
use super::{ConnectionState, HostPlatform, RemoteLocation};
use crate::{JayError, JayResult, SyncToken};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub(crate) struct RemoteRepo {
    pub path: PathBuf,
    pub store: PathBuf,
    pub primary: PathBuf,
    pub workspace: String,
    location: RemoteLocation,
    connection: Arc<MachineConnection>,
    operation: Mutex<String>,
}

impl RemoteRepo {
    pub fn open(location: RemoteLocation) -> JayResult<Self> {
        let (connection, metadata): (_, OpenedRepo) = request(&location, "__open")?;
        let location = RemoteLocation::new(&location.machine, &metadata.root)?;
        Ok(Self {
            path: location.encode().into(),
            store: location.qualify(&metadata.store).into(),
            primary: location.qualify(&metadata.primary).into(),
            workspace: metadata.workspace,
            operation: Mutex::new(metadata.operation),
            location,
            connection,
        })
    }
    pub fn state(&self) -> ConnectionState {
        self.connection.state()
    }
    pub fn platform(&self) -> HostPlatform {
        self.connection.platform()
    }
    pub fn error(&self) -> Option<String> {
        self.connection.snapshot().error()
    }
    pub fn reconnect(&self) -> JayResult<()> {
        request::<OpenedRepo>(&self.location, "__open").map(drop)
    }
    pub fn disconnect(&self) -> JayResult<()> {
        let connection = self.connection.snapshot();
        if connection.busy() {
            return Err(JayError::refused(jayjay_i18n::ssh::error_disconnect_busy()));
        }
        connection.close();
        Ok(())
    }
    pub fn changed(&self, operation: String) -> bool {
        std::mem::replace(&mut *self.operation.lock().unwrap(), operation.clone()) != operation
    }
    pub fn unqualify(&self, location: &str) -> JayResult<String> {
        self.location.unqualify(location)
    }
    pub fn call<T: DeserializeOwned>(
        &self,
        method: &str,
        args: Vec<Value>,
        sync: Option<&SyncToken>,
        qualify: bool,
    ) -> JayResult<T> {
        let connection = self.connection.snapshot();
        let response = connection.request(&self.location.path, method, args, sync)?;
        response.decode(method, &self.location, qualify)
    }
}
