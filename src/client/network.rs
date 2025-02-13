use std::sync::mpsc;
use std::thread;

use game_test::action::Action;
use game_test::action::Response;

use websocket::client::ClientBuilder;
use websocket::ws::dataframe::DataFrame;
use websocket::OwnedMessage;

pub struct Connection {
    sender: websocket::sender::Writer<std::net::TcpStream>,
    receiver: mpsc::Receiver<OwnedMessage>,
    _read_thread: thread::JoinHandle<()>,
}

impl Connection {
    pub fn open(url: &str) -> anyhow::Result<Self> {
        let client = ClientBuilder::new(url)?.connect_insecure()?;

        let (mut receiver, sender) = client.split()?;
        let (msg_tx, msg_rx) = mpsc::channel();

        // Spawn thread to handle incoming messages
        let read_thread = thread::spawn(move || {
            for message in receiver.incoming_messages() {
                match message {
                    Ok(msg) => {
                        if let Err(e) = msg_tx.send(msg) {
                            println!("websocket send error: {e}");
                            break;
                        }
                    }
                    Err(e) => {
                        println!("websocket incoming message error: {:?}", e);
                        break;
                    }
                }
            }
        });

        Ok(Self {
            sender,
            receiver: msg_rx,
            _read_thread: read_thread,
        })
    }

    pub fn send(&mut self, action: &Action) -> anyhow::Result<()> {
        self.sender
            .send_message(&OwnedMessage::Binary(bincode::serialize(action)?))?;
        Ok(())
    }

    fn parse_response(msg: OwnedMessage) -> anyhow::Result<Response> {
        if msg.is_data() {
            let res = bincode::deserialize::<Response>(&msg.take_payload())?;
            Ok(res)
        } else {
            anyhow::bail!("Received non-data message")
        }
    }

    pub fn receive(&self) -> anyhow::Result<Response> {
        match self.receiver.recv() {
            Ok(msg) => Self::parse_response(msg),
            Err(_) => anyhow::bail!("Read thread terminated"),
        }
    }

    pub fn try_receive(&self) -> anyhow::Result<Option<Response>> {
        match self.receiver.try_recv() {
            Ok(msg) => Ok(Some(Self::parse_response(msg)?)),
            Err(mpsc::TryRecvError::Empty) => Ok(None),
            Err(mpsc::TryRecvError::Disconnected) => anyhow::bail!("Read thread terminated"),
        }
    }
}
