use std::time::Duration;

use anyhow::Result;
use dashmap::DashMap;
use futures_util::SinkExt;
use futures_util::StreamExt;
use futures_util::stream::SplitSink;
use futures_util::stream::SplitStream;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::protocol::frame::CloseFrame;

use game_common::network::Response;

use super::Action;

pub struct Server {
    pub listener: TcpListener,
    // socket_id, reverse communication channel, action
    pub pending_actions: (
        flume::Sender<(String, Action)>,
        flume::Receiver<(String, Action)>,
    ),
    pub socket_sender: DashMap<String, mpsc::Sender<Response>>,
    // player id keyed to socket
    // and socket keyed to player id
    player_socket_map: DashMap<String, String>,
}

impl Server {
    pub async fn new() -> Result<Self> {
        let addr = "0.0.0.0:1351";
        let try_socket = TcpListener::bind(addr).await;
        let listener = try_socket.expect("Failed to bind");

        Ok(Self {
            pending_actions: flume::unbounded(),
            socket_sender: DashMap::new(),
            listener,
            player_socket_map: DashMap::new(),
        })
    }

    /// Send to a player id
    /// eat any error that occurs
    pub async fn send_to_player(&self, player_id: &str, res: Response) {
        if let Some(socket_id) = self.socket_by_player_id(player_id).await {
            if let Err(e) = self.send(&socket_id, res.clone()).await {
                println!("Error sending to player {player_id}: {:?}", e);
                println!("message: {:?}", res);
                if e.to_string() == "channel closed" {
                    println!("player disconnected");
                    self.logout_socket(&socket_id).await;
                }
            }
        }
    }

    /// Send to a socket id
    /// This can be invoked from any thread
    pub async fn send(&self, socket_id: &str, res: Response) -> anyhow::Result<()> {
        if let Some(mut sender) = self.socket_sender.get_mut(socket_id) {
            let sender = sender.value_mut();
            sender.send(res).await?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("channel closed"))
        }
    }

    /// This will be invoked from a non-main thread
    pub async fn accept_connection(&self, stream: TcpStream) {
        let addr = stream
            .peer_addr()
            .expect("connected streams should have a peer address");
        println!("Peer address: {}", addr);

        let ws_stream = tokio_tungstenite::accept_async(stream).await;
        if ws_stream.is_err() {
            println!("WARNING: error accepting connection, aborting");
            return;
        }
        let ws_stream = ws_stream.unwrap();

        println!("New WebSocket connection: {}", addr);

        let socket_id = nanoid::nanoid!();
        let (mut write, mut read) = ws_stream.split();

        let (sendv, mut recv) = mpsc::channel::<Response>(64);
        self.socket_sender.insert(socket_id.clone(), sendv);
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
                                // println!("{:?}", action);
                                self.pending_actions.0.send((socket_id.to_string(), action)).unwrap();
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
        self.socket_sender.remove(socket_id);
        recv.close();
    }

    pub async fn player_by_socket_id(&self, socket_id: &str) -> Option<String> {
        if let Some(player_id) = self.player_socket_map.get(socket_id) {
            let player_id = player_id.value();
            if let Some(socket_id_internal) = self.player_socket_map.get(player_id) {
                let socket_id_internal = socket_id_internal.value();
                if socket_id == socket_id_internal {
                    return Some(player_id.clone());
                }
            }
        }
        None
    }

    pub async fn socket_by_player_id(&self, player_id: &str) -> Option<String> {
        if let Some(v) = self.player_socket_map.get(player_id) {
            Some(v.value().clone())
        } else {
            None
        }
    }

    pub async fn logout_socket(&self, socket_id: &str) -> Option<String> {
        let player_id = self.player_socket_map.remove(socket_id);
        if let Some((_, player_id)) = player_id.as_ref() {
            self.player_socket_map.remove(player_id);
        }
        player_id.and_then(|(_, v)| Some(v))
    }

    pub async fn register_player(&self, socket_id: String, player_id: String) {
        self.player_socket_map
            .insert(socket_id.clone(), player_id.clone());
        self.player_socket_map.insert(player_id, socket_id);
    }
}
