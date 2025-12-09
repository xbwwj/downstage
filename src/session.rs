use chromiumoxide_types::Command;

use crate::{Connection, Result};

#[derive(Debug, Clone)]
pub struct CdpSession {
    pub(crate) connection: Connection,
    pub(crate) session_id: Option<String>,
}

impl CdpSession {
    pub(crate) fn with_session_id(&self, session_id: Option<String>) -> Self {
        Self {
            connection: self.connection.clone(),
            session_id,
        }
    }

    pub fn session_id(&self) -> Option<&String> {
        self.session_id.as_ref()
    }

    pub async fn send<T: Command>(&self, command: T) -> Result<T::Response> {
        self.connection.send(self.session_id.clone(), command).await
    }
}
