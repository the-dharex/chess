use serde::{Serialize, Deserialize};
use std::net::TcpStream;
use std::io::{Read, Write};
use std::thread;
use std::sync::mpsc;
use crate::pieces::PieceColor;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NetworkMessage {
    Handshake { color: PieceColor },
    Move { from: (usize, usize), to: (usize, usize) },
}

pub struct NetworkClient {
    stream: TcpStream,
    rx: mpsc::Receiver<NetworkMessage>,
}

impl NetworkClient {
    pub fn new(stream: TcpStream) -> Self {
        stream.set_nonblocking(false).ok();  
        let mut stream_clone = stream.try_clone().unwrap();
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let mut buffer = [0u8; 1024];
            loop {
                match stream_clone.read(&mut buffer) {
                    Ok(0) => break, // Fin del stream (EOF)
                    Ok(n) => {
                        if let Ok(msg) = bincode::deserialize::<NetworkMessage>(&buffer[..n]) {
                            let _ = tx.send(msg);
                        }
                    }
                    Err(_) => {
                        // Error o desconexión
                        break;
                    }
                }
            }
        });

        Self {
            stream,
            rx,
        }
    }

    pub fn send(&mut self, msg: NetworkMessage) {
        if let Ok(bytes) = bincode::serialize(&msg) {
            let _ = self.stream.write_all(&bytes);
        }
    }

    pub fn try_recv(&self) -> Option<NetworkMessage> {
        self.rx.try_recv().ok()
    }
}
