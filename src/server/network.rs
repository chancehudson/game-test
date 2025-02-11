use std::net::TcpListener;
use std::thread::spawn;

use tungstenite::accept;

pub struct Server {
    // pub server: TcpListener,
}

impl Server {
    pub async fn new() -> anyhow::Result<Self> {
        let server = TcpListener::bind("127.0.0.1:1351").unwrap();
        spawn(move || -> anyhow::Result<()> {
            for stream in server.incoming() {
                let mut websocket = accept(stream?)?;
                loop {
                    let msg = websocket.read()?;
                    // We do not want to send back ping/pong messages.
                    if msg.is_text() {
                        websocket.send(msg)?;
                    }
                }
            }
            Ok(())
        });
        Ok(Self {})
    }
}
