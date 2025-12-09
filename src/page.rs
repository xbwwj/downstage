use chromiumoxide_cdp::cdp::browser_protocol::{
    dom::{GetDocumentParams, GetDocumentReturns},
    page::{self, NavigateParams},
    target::TargetId,
};

use crate::{element::ElementHandle, error::Result, session::CdpSession};

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

    pub async fn goto(&self, url: &str) -> Result<()> {
        self.session
            .send(NavigateParams {
                url: url.to_string(),
                referrer: None,
                transition_type: None,
                frame_id: None,
                referrer_policy: None,
            })
            .await?;
        Ok(())
    }

    pub async fn document(&self) -> Result<ElementHandle> {
        let GetDocumentReturns { root } = self
            .session
            .send(GetDocumentParams {
                depth: Some(-1),
                pierce: Some(true),
            })
            .await?;
        Ok(ElementHandle {
            session: self.session.clone(),
            node_id: root.node_id,
        })
    }

    #[doc(alias("$", "dollar", "find_element"))]
    pub async fn query_selector(&self, selector: &str) -> Result<ElementHandle> {
        self.document().await?.query_selector(selector).await
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
