use bevy::prelude::*;
use bevy::tasks::futures_lite::StreamExt;
use bevy::tasks::futures_lite::stream::block_on;
use futures_util::SinkExt;
use gloo_timers::future::TimeoutFuture;
use wasm_bindgen_futures::future_to_promise;
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
    connected_rx: flume::Receiver<anyhow::Result<()>>,
}

impl NetworkConnection {
    pub fn is_closed(&self) -> bool {
        !self.close_rx.is_empty()
    }

    pub fn is_open(&self) -> anyhow::Result<bool> {
        if self.connected_rx.is_empty() {
            return Ok(false);
        }
        let msg = self.connected_rx.recv().unwrap();
        if let Err(e) = msg {
            Err(anyhow::format_err!(e))
        } else {
            Ok(true)
        }
    }

    pub fn attempt_connection(url: String) -> Self {
        let url_clone = url.clone();
        let (send_tx, send_rx) = flume::unbounded::<Action>();
        let (receive_tx, receive_rx) = flume::unbounded::<Response>();
        let (close_tx, close_rx) = flume::bounded::<()>(1);
        let (connected_tx, connected_rx) = flume::unbounded::<anyhow::Result<()>>();
        spawn_local(async move {
            let connection = WsMeta::connect(url_clone, None).await;
            if let Err(e) = connection {
                web_sys::console::log_1(&"Connection errored".into());
                web_sys::console::log_2(&"err:".into(), &e.to_string().into());
                connected_tx.send(Err(anyhow::format_err!(e))).ok();
                return; // thread ends
            }
            if let Ok((ws, mut wsio)) = connection {
                web_sys::console::log_1(&"Connection succeeded".into());
                if let Err(_) = connected_tx.send(Ok(())) {
                    println!("WARNING: No receivers for network connection attempt!");
                    println!("halting connection thread");
                    return; // thread ends
                }
                loop {
                    while let Ok(action) = send_rx.try_recv() {
                        if let Err(e) = wsio
                            .send(WsMessage::Binary(bincode::serialize(&action).unwrap()))
                            .await
                        {
                            println!("Error sending ws message {:?}", e);
                        }
                    }
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
                    TimeoutFuture::new(17).await;
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
            connected_rx,
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
