use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use chromiumoxide_types::{CallId, Command, MethodCall, Response};
use futures::{SinkExt, StreamExt, stream::SplitSink};
use tokio::{
    net::TcpStream,
    sync::{
        Mutex,
        oneshot::{self, Sender},
    },
};
use tokio_websockets::{ClientBuilder, MaybeTlsStream, Message, WebSocketStream};
use tracing::{debug, error, instrument, warn};

use crate::{Error, error::Result};

type CdpSink = Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>;
type ChannelMap = Arc<Mutex<HashMap<CallId, Sender<Response>>>>;

/// The underlying CDP WebSocket connection.
#[derive(Debug, Clone)]
pub struct Connection {
    sink: CdpSink,
    channels: ChannelMap,
    id: Arc<AtomicUsize>,
}

impl Connection {
    #[instrument]
    pub async fn connect(uri: &str) -> Result<Self> {
        let uri = uri.parse()?;
        let (socket, _) = ClientBuilder::from_uri(uri).connect().await?;

        let (sink, mut stream) = socket.split();

        let sink = Arc::new(Mutex::new(sink));
        let channels = ChannelMap::default();

        // response event loop
        {
            let channels = channels.clone();
            tokio::spawn(async move {
                loop {
                    // next message
                    let Some(message) = stream.next().await else {
                        debug!("CDP WS stream ends");
                        break;
                    };
                    let message = match message {
                        Ok(message) => message,
                        Err(err) => {
                            error!(%err, "Unable to read CDP WS stream");
                            continue;
                        }
                    };

                    // only text message
                    let text = match message.as_text() {
                        Some(text) => text,
                        None => {
                            error!(?message, "CDP WS message is not text");
                            continue;
                        }
                    };

                    // deserialize ws message
                    // TODO: event or response, use `Message`
                    let response = match serde_json::from_str::<Response>(text) {
                        Ok(response) => response,
                        Err(err) => {
                            error!(%err, "Unable to deserialize CDP response");
                            continue;
                        }
                    };

                    // retrieve tx
                    let Some(tx) = channels.lock().await.remove(&response.id) else {
                        warn!(id = %response.id, "tx missing for response");
                        continue;
                    };

                    // tx send response back
                    if let Err(response) = tx.send(response) {
                        error!(?response, "Unable to send response back");
                    }
                }
            });
        }

        Ok(Self {
            sink,
            channels,
            id: Default::default(),
        })
    }

    pub async fn send<T: Command>(
        &self,
        session_id: Option<String>,
        command: T,
    ) -> Result<T::Response> {
        // inc id
        let id = CallId::new(self.id.fetch_add(1, Ordering::SeqCst));

        let method_call = MethodCall {
            id,
            session_id,
            method: command.identifier(),
            params: serde_json::to_value(command)?,
        };
        let text = serde_json::to_string(&method_call)?;

        let (tx, rx) = oneshot::channel::<Response>();

        // prepare channel
        self.channels.lock().await.insert(id, tx);

        // start call
        self.sink.lock().await.send(Message::text(text)).await?;

        // wait for result and deserialize
        let response = rx.await?;
        match response.result {
            Some(response) => Ok(serde_json::from_value::<T::Response>(response)?),
            None => Err(Error::Response(response.error)),
        }
    }
}
