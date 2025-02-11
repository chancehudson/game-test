use websocket::client::ClientBuilder;
use websocket::OwnedMessage;
use std::sync::mpsc;
use std::thread;

pub struct Connection {
    sender: websocket::sender::Writer<std::net::TcpStream>,
    receiver: mpsc::Receiver<OwnedMessage>,
    _read_thread: thread::JoinHandle<()>,
}

impl Connection {
    pub fn open(url: &str) -> anyhow::Result<Self> {
        let client = ClientBuilder::new(url)?
            .connect_insecure()?;

        let (mut receiver, sender) = client.split()?;
        let (msg_tx, msg_rx) = mpsc::channel();

        // Spawn thread to handle incoming messages
        let read_thread = thread::spawn(move || {
            for message in receiver.incoming_messages() {
                match message {
                    Ok(msg) => {
                        if msg_tx.send(msg).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(Self {
            sender,
            receiver: msg_rx,
            _read_thread: read_thread,
        })
    }

    pub fn send(&mut self, message: String) -> anyhow::Result<()> {
        self.sender.send_message(&OwnedMessage::Text(message))?;
        Ok(())
    }

    pub fn receive(&self) -> anyhow::Result<OwnedMessage> {
        match self.receiver.recv() {
            Ok(msg) => Ok(msg),
            Err(_) => anyhow::bail!("Read thread terminated")
        }
    }

    pub fn try_receive(&self) -> anyhow::Result<Option<OwnedMessage>> {
        match self.receiver.try_recv() {
            Ok(msg) => Ok(Some(msg)),
            Err(mpsc::TryRecvError::Empty) => Ok(None),
            Err(mpsc::TryRecvError::Disconnected) => anyhow::bail!("Read thread terminated")
        }
    }
}
