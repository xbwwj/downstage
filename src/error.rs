use http::uri::InvalidUri;
use thiserror::Error;
use tokio::{io, sync::oneshot::error::RecvError, time::error::Elapsed};

pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("the child fail to spawn")]
    Spawn(#[from] io::Error),
    #[error("the child has no stderr")]
    NoStderr,
    #[error("reach timeout limit")]
    Timeout(#[from] Elapsed),
    #[error("given uri is invalid")]
    InvalidUri(#[from] InvalidUri),
    #[error("{0}")]
    Socket(#[from] tokio_websockets::Error),
    #[error("{0}")]
    Serialize(#[from] serde_json::Error),
    #[error("{0}")]
    Recv(#[from] RecvError),
    #[error("respond with a error: {0:?}")]
    Response(Option<chromiumoxide_types::Error>),
}
