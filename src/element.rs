use chromiumoxide_cdp::cdp::browser_protocol::dom::{
    NodeId, QuerySelectorParams, QuerySelectorReturns,
};

use crate::{CdpSession, error::Result};

#[derive(Debug, Clone)]
pub struct ElementHandle {
    pub(crate) session: CdpSession,
    pub(crate) node_id: NodeId,
}

impl ElementHandle {
    #[doc(alias("$", "dollar", "find_element"))]
    pub async fn query_selector(&self, selector: &str) -> Result<Self> {
        let QuerySelectorReturns { node_id } = self
            .session
            .send(QuerySelectorParams {
                node_id: self.node_id,
                selector: selector.to_string(),
            })
            .await?;

        Ok(Self {
            session: self.session.clone(),
            node_id,
        })
    }
}
