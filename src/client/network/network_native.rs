use bevy::prelude::*;
use futures_util::SinkExt;
use futures_util::StreamExt;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;

use game_test::action::Action;
use game_test::action::Response;

#[derive(Component)]
pub struct NetworkConnection {
    url: String,
    send_tx: flume::Sender<Action>,
    receive_rx: flume::Receiver<Response>,
    worker_thread: std::thread::JoinHandle<()>,
}

impl NetworkConnection {
    pub fn is_closed(&self) -> bool {
        self.worker_thread.is_finished()
    }

    pub fn attempt_connection(url: String) -> Self {
        let url_clone = url.clone();
        let (send_tx, send_rx) = flume::unbounded::<Action>();
        let (receive_tx, receive_rx) = flume::unbounded::<Response>();
        Self {
            url,
            send_tx,
            receive_rx,
            worker_thread: std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let connection = connect_async(url_clone).await;
                    if let Ok((ws_stream, _)) = connection {
                        let (mut write, mut read) = ws_stream.split();
                        tokio::spawn(async move {
                            while let Some(Ok(msg)) = read.next().await {
                                if msg.is_binary() {
                                    if let Ok(r) =
                                        bincode::deserialize::<Response>(&msg.into_data())
                                    {
                                        if let Err(e) = receive_tx.send(r) {
                                            println!("receive err {:?}", e);
                                            break;
                                        }
                                    } else {
                                        println!("failed to deserialize response");
                                    }
                                } else {
                                    println!("non-binary message");
                                }
                            }
                        });
                        while let Ok(action) = send_rx.recv_async().await {
                            if let Ok(serialized) = bincode::serialize(&action) {
                                if let Err(e) = write.send(Message::binary(serialized)).await {
                                    println!("error sending {:?}", e);
                                    break;
                                }
                            }
                        }
                    } else {
                        println!(
                            "Error connecting to server: {:?}",
                            connection.err().unwrap()
                        );
                    }
                });
            }),
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
