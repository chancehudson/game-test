use bevy::prelude::*;
use bevy::tasks::futures_lite::stream::block_on;
use bevy::tasks::futures_lite::StreamExt;
use futures_util::SinkExt;
use gloo_timers::future::TimeoutFuture;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::future_to_promise;
use wasm_bindgen_futures::js_sys::Promise;
use wasm_bindgen_futures::spawn_local;
use ws_stream_wasm::*;

use game_test::action::Action;
use game_test::action::Response;

#[derive(Component)]
pub struct NetworkConnection {
    url: String,
    send_tx: flume::Sender<Action>,
    receive_rx: flume::Receiver<Response>,
    close_rx: flume::Receiver<()>,
}

impl NetworkConnection {
    pub fn is_closed(&self) -> bool {
        !self.close_rx.is_empty()
    }

    pub fn attempt_connection(url: String) -> Self {
        let url_clone = url.clone();
        let (send_tx, send_rx) = flume::unbounded::<Action>();
        let (receive_tx, receive_rx) = flume::unbounded::<Response>();
        let (close_tx, close_rx) = flume::bounded::<()>(1);
        spawn_local(async move {
            if let Ok((ws, mut wsio)) = WsMeta::connect(url_clone, None).await {
                loop {
                    println!("read");
                    while let Ok(action) = send_rx.try_recv() {
                        if let Err(e) = wsio
                            .send(WsMessage::Binary(bincode::serialize(&action).unwrap()))
                            .await
                        {
                            println!("Error sending ws message {:?}", e);
                        }
                    }
                    println!("write");
                    for msg in block_on(wsio.drain()) {
                        match msg {
                            WsMessage::Binary(bytes) => {
                                if let Ok(r) = bincode::deserialize::<Response>(&bytes) {
                                    if let Err(e) = receive_tx.send(r) {
                                        println!("receive err {:?}", e);
                                        break;
                                    }
                                } else {
                                    println!("failed to deserialize response");
                                }
                            }
                            _ => {}
                        }
                    }
                    TimeoutFuture::new(10).await;
                    if ws.ready_state() != WsState::Open {
                        println!("breaking");
                        break;
                    }
                }
                close_tx.send(()).ok();
            }
        });
        Self {
            url,
            send_tx,
            receive_rx,
            close_rx,
        }
    }

    pub fn read_connection(&self) -> Vec<Response> {
        let mut responses = vec![];
        while let Ok(r) = self.receive_rx.try_recv() {
            responses.push(r);
        }
        responses
    }

    pub fn write_connection(&self, action: Action) {
        if let Err(e) = self.send_tx.send(action) {
            println!("error writing to network connection (native): {:?}", e);
        }
    }
}
