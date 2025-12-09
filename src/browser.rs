use std::{process::Stdio, sync::Arc, time::Duration};

use chromiumoxide_cdp::cdp::browser_protocol::{
    browser::{CloseParams, GetVersionParams, GetVersionReturns},
    target::{
        AttachToTargetParams, AttachToTargetReturns, CreateTargetParams, CreateTargetReturns,
    },
};
use regex::Regex;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Child,
    time::timeout,
};

use crate::{
    connection::Connection,
    error::{Error, Result},
    page::Page,
    session::CdpSession,
};

/// Launch or connect to a browser for automation.
#[derive(Debug, Clone)]
pub struct Browser {
    pub child: Option<Arc<Child>>,
    pub session: CdpSession,
}

impl Browser {
    pub async fn launch() -> Result<Browser> {
        let (child, uri) = launch_chromium().await?;
        let connection = Connection::connect(&uri).await?;
        let session = CdpSession {
            connection,
            session_id: None,
        };
        Ok(Self {
            child: Some(Arc::new(child)),
            session,
        })
    }

    pub async fn connect(uri: &str) -> Result<Self> {
        let connection = Connection::connect(uri).await?;
        let session = CdpSession {
            connection,
            session_id: None,
        };
        Ok(Self {
            child: None,
            session,
        })
    }

    pub async fn close(&self) -> Result<()> {
        self.session.send(CloseParams {}).await?;
        Ok(())
    }

    pub async fn version(&self) -> Result<GetVersionReturns> {
        self.session.send(GetVersionParams {}).await
    }

    pub async fn new_page(&self) -> Result<Page> {
        let CreateTargetReturns { target_id } = self
            .session
            .send(CreateTargetParams {
                url: "".into(),
                ..Default::default()
            })
            .await?;
        let AttachToTargetReturns { session_id } = self
            .session
            .send(AttachToTargetParams {
                target_id: target_id.clone(),
                flatten: Some(true),
            })
            .await?;
        Ok(Page {
            target_id,
            session: self.session.with_session_id(Some(session_id.into())),
        })
    }
}

impl Drop for Browser {
    fn drop(&mut self) {
        let session = self.session.clone();
        tokio::spawn(async move {
            _ = session.send(CloseParams {}).await;
        });
    }
}

/// Launch chromium and retrieve ws endpoint from stderr.
async fn launch_chromium() -> Result<(Child, String)> {
    let mut child = tokio::process::Command::new("chromium")
        .arg("--enable-automation")
        .arg("--remote-debugging-port=0")
        // TODO: tempdir
        // .arg("--user-data-dir=/tmp/chrome-dev")
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()?;

    let Some(stderr) = child.stderr.as_mut() else {
        return Err(Error::NoStderr);
    };

    let reader = BufReader::new(stderr);
    let mut lines = reader.lines();

    let re = Regex::new("ws://.+$").expect("invalid regex");
    let ws_url = timeout(Duration::from_secs(30), async move {
        loop {
            let Ok(Some(line)) = lines.next_line().await else {
                continue;
            };
            if let Some(capture) = re.captures(&line) {
                let ws = capture.get(0).expect("should have captured");
                return ws.as_str().to_string();
            }
        }
    })
    .await?;

    Ok((child, ws_url))
}
