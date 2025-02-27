use std::collections::HashMap;
use std::collections::VecDeque;
use std::time::Duration;

use anyhow::Result;
use futures_util::stream::SplitSink;
use futures_util::stream::SplitStream;
use futures_util::SinkExt;
use futures_util::StreamExt;
use game_test::action::Response;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use tokio_tungstenite::WebSocketStream;
use tungstenite::protocol::frame::CloseFrame;
use tungstenite::Message;

use super::Action;

pub struct Server {
    pub listener: TcpListener,
    // socket_id, reverse communication channel, action
    pub action_queue: RwLock<VecDeque<(String, Action)>>,
    pub socket_sender: RwLock<HashMap<String, mpsc::Sender<Response>>>,
}

impl Server {
    pub async fn new() -> Result<Self> {
        let action_queue = RwLock::new(VecDeque::new());

        let addr = "0.0.0.0:1351";
        let try_socket = TcpListener::bind(addr).await;
        let listener = try_socket.expect("Failed to bind");

        Ok(Self {
            action_queue,
            socket_sender: RwLock::new(HashMap::new()),
            listener,
        })
    }

    pub async fn send(&self, socket_id: &str, res: Response) -> anyhow::Result<()> {
        if let Some(sender) = self.socket_sender.write().await.get_mut(socket_id) {
            sender.send(res).await?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("channel closed"))
        }
    }

    pub async fn accept_connection(&self, stream: TcpStream) {
        let addr = stream
            .peer_addr()
            .expect("connected streams should have a peer address");
        println!("Peer address: {}", addr);

        let ws_stream = tokio_tungstenite::accept_async(stream)
            .await
            .expect("Error during the websocket handshake occurred");

        println!("New WebSocket connection: {}", addr);

        let socket_id = nanoid::nanoid!();
        let (mut write, mut read) = ws_stream.split();

        let (sendv, mut recv) = mpsc::channel::<Response>(64);
        self.socket_sender
            .write()
            .await
            .insert(socket_id.clone(), sendv);
        // our client loop may throw errors. We don't want to propagate them through
        // into the main network logic so we handle them here
        if let Err(e) = self
            .client_loop(&socket_id, &mut write, &mut read, &mut recv)
            .await
        {
            println!("websocket client loop errored: {:?}", e);
            // we'll cleanup now with the assumption that the connection will be forcibly closed
            self.cleanup_connection(&socket_id, &mut recv).await;

            // be nice and send a close frame, ignore any errors
            let close_frame = Message::Close(Some(CloseFrame {
                code: tungstenite::protocol::frame::coding::CloseCode::Error,
                reason: "internal server error, sorry!".to_string().into(),
            }));
            tokio::time::timeout(Duration::from_millis(500), write.send(close_frame))
                .await
                .ok();

            // close the connection
            if let Err(e) = write.close().await {
                println!("error closing websocket connection: {:?}", e);
            }
        }
    }

    /// Returning Ok indicates the connection has been closed and cleaned up.
    /// Returning Err indicates a logic error and the connection should be cleaned up.
    async fn client_loop(
        &self,
        socket_id: &str,
        write: &mut SplitSink<WebSocketStream<TcpStream>, Message>,
        read: &mut SplitStream<WebSocketStream<TcpStream>>,
        recv: &mut mpsc::Receiver<Response>,
    ) -> anyhow::Result<()> {
        let mut interval = tokio::time::interval(Duration::from_millis(5000));
        loop {
            tokio::select! {
                // we have a response from the game server to give to the client
                res = recv.recv() => {
                    match res {
                        Some(res) => {
                            write.send(Message::binary(bincode::serialize(&res)?)).await?;
                        }
                        None => {
                            // this should be unreachable, but we'll include logic for it
                            // just in case
                            println!("mpsc channel closed");
                            self.cleanup_connection(socket_id, recv).await;
                            break;
                        },
                    }
                }
                // we have a message from the client to the game server
                msg = read.next() => {
                    match msg {
                        Some(msg) => {
                            if let Err(e) = msg {
                                println!("websocket client error: {}", e);
                                self.cleanup_connection(socket_id, recv).await;
                                break;
                            }
                            let msg = msg.unwrap();
                            if msg.is_binary() {
                                let action = bincode::deserialize::<Action>(&msg.clone().into_data())?;
                                println!("{:?}", action);
                                self.action_queue.write().await.push_back((socket_id.to_string(), action));
                            } else if msg.is_close() {
                                self.cleanup_connection(socket_id, recv).await;
                                break;
                            }
                        }
                        // connection is closed
                        None => {
                            self.cleanup_connection(socket_id, recv).await;
                            break;
                        },
                    }
                }
                // we're sending tick/keepalive
                _ = interval.tick() => {
                    println!("sending keepalive");
                    let r = bincode::serialize(&Response::Tick{})?;
                    write.send(Message::binary(r)).await?;
                }
            }
        }
        Ok(())
    }

    async fn cleanup_connection(&self, socket_id: &str, recv: &mut mpsc::Receiver<Response>) {
        self.socket_sender.write().await.remove(socket_id);
        recv.close();
    }
}
