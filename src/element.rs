use chromiumoxide_cdp::cdp::browser_protocol::{
    dom::{GetBoxModelParams, NodeId, QuerySelectorParams, QuerySelectorReturns},
    input::{DispatchMouseEventParams, DispatchMouseEventType},
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

    pub async fn bounding_box(&self) -> Result<(f64, f64, f64, f64)> {
        let chromiumoxide_cdp::cdp::browser_protocol::dom::GetBoxModelReturns { model } = self
            .session
            .send(GetBoxModelParams {
                node_id: Some(self.node_id),
                backend_node_id: None,
                // TODO: playwright use objectid
                object_id: None,
            })
            .await?;
        let quad = model.border.inner();
        // TODO: handle quad out of bound
        let x = quad[0].min(quad[2]).min(quad[4]).min(quad[6]);
        let y = quad[1].min(quad[3]).min(quad[5]).min(quad[7]);
        let width = quad[0].max(quad[2]).max(quad[4]).max(quad[6]) - x;
        let height = quad[1].max(quad[3]).max(quad[5]).max(quad[7]) - y;
        // TODO: frame position relative
        Ok((x, y, width, height))
    }

    /// TODO:playwright have better way implementing this, involving injected script.
    pub async fn click(&self) -> Result<()> {
        let bbox = self.bounding_box().await?;
        let center = (bbox.0 + bbox.2 / 2., bbox.1 + bbox.3 / 2.);

        self.session
            .send(DispatchMouseEventParams::new(
                DispatchMouseEventType::MousePressed,
                center.0,
                center.1,
            ))
            .await?;
        Ok(())
    }
}
