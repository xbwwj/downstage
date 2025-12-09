use chromiumoxide_cdp::cdp::browser_protocol::{page, target::TargetId};

use crate::{error::Result, session::CdpSession};

/// Page provides methods to interact with a single tab in browser.
#[derive(Debug, Clone)]
pub struct Page {
    pub session: CdpSession,
    pub target_id: TargetId,
}

impl Page {
    pub async fn close(&self) -> Result<()> {
        self.session.send(page::CloseParams {}).await?;
        Ok(())
    }
}

impl Drop for Page {
    fn drop(&mut self) {
        let session = self.session.clone();
        tokio::spawn(async move {
            _ = session.send(page::CloseParams {}).await;
        });
    }
}
