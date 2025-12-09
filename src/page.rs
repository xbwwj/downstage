use chromiumoxide_cdp::cdp::browser_protocol::target::{CloseTargetParams, TargetId};

use crate::{error::Result, session::CdpSession};

/// Page provides methods to interact with a single tab in browser.
#[derive(Debug, Clone)]
pub struct Page {
    pub session: CdpSession,
    pub target_id: TargetId,
}

impl Page {
    pub async fn close(&self) -> Result<()> {
        self.session
            .send(CloseTargetParams {
                target_id: self.target_id.clone(),
            })
            .await?;
        Ok(())
    }
}

impl Drop for Page {
    fn drop(&mut self) {
        let session = self.session.clone();
        let target_id = self.target_id.clone();
        tokio::spawn(async move {
            _ = session.send(CloseTargetParams { target_id }).await;
        });
    }
}
