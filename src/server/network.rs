use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::RwLock;
use std::time::Duration;

use anyhow::Result;
use futures_util::SinkExt;
use futures_util::StreamExt;
use game_test::action::Response;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
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

        let addr = "127.0.0.1:1351";
        let try_socket = TcpListener::bind(addr).await;
        let listener = try_socket.expect("Failed to bind");

        Ok(Self {
            action_queue,
            socket_sender: RwLock::new(HashMap::new()),
            listener,
        })
    }

    pub async fn send(&self, socket_id: &String, res: Response) -> anyhow::Result<()> {
        if let Some(sender) = self.socket_sender.write().unwrap().get_mut(socket_id) {
            sender.send(res).await?;
        }
        Ok(())
    }

    pub async fn accept_connection(&self, stream: TcpStream) -> anyhow::Result<()> {
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
        let mut interval = tokio::time::interval(Duration::from_millis(5000));
        let (sendv, mut recv) = mpsc::channel::<Response>(64);
        self.socket_sender
            .write()
            .unwrap()
            .insert(socket_id.clone(), sendv);
        loop {
            tokio::select! {
                // we have a response from the game server to give to the client
                res = recv.recv() => {
                    match res {
                        Some(res) => {
                            write.send(Message::binary(bincode::serialize(&res)?)).await?;
                        }
                        None => break,
                    }
                }
                // we have a message from the client to the game server
                msg = read.next() => {
                    match msg {
                        Some(msg) => {
                            let msg = msg?;
                            if msg.is_binary() {
                                let action = bincode::deserialize::<Action>(&msg.clone().into_data())?;
                                self.action_queue.write().unwrap().push_back((socket_id.clone(), action));
                            } else if msg.is_close() {
                                break;
                            }
                        }
                        None => break,
                    }
                }
                // we're sending tick/keepalive
                _ = interval.tick() => {
                    let r = bincode::serialize(&Response::Tick{})?;
                    write.send(Message::binary(r)).await?;
                    println!("sending");
                }
            }
        }
        Ok(())
    }
}
