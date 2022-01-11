use std::net::SocketAddr;

use tokio::{
    io::{ AsyncBufReadExt, AsyncWriteExt, BufReader },
    net::{ TcpListener, TcpStream },
    sync::broadcast::*,
};

const LOCAL: &str = "127.0.0.1:8080";

struct Client {
    tx: Sender<(String, SocketAddr)>,
    rx: Receiver<(String, SocketAddr)>,
    socket: TcpStream,
    addr: SocketAddr,
    name: String,
}

async fn handle(client: &mut Client) {
    let (reader, mut writer) = client.socket.split();

    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    writer.write_all("enter your username: ".as_bytes()).await.unwrap();
    reader.read_line(&mut client.name).await.unwrap();
    let header = format!("[{}]: ", client.name.trim_end_matches('\n'));

    loop {
        tokio::select! {
            result = reader.read_line(&mut line) => {
                if result.unwrap() == 0 {
                    return;
                }
                client.tx.send((format!("{} {}", header, line.clone()) , client.addr)).unwrap();
                line.clear();
            }
            result = client.rx.recv() => {
                let (msg, other_addr) = result.unwrap();

                if client.addr != other_addr {
                    writer.write_all(msg.as_bytes()).await.unwrap();
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind(LOCAL).await.unwrap();

    let (tx, _rx) = channel(100);

    loop {
        let (socket, addr) = listener.accept().await.unwrap();

        println!("new client connected: {:?}", addr);

        let tx = tx.clone();
        let rx = tx.subscribe();

        let mut client = Client {
            tx,
            rx,
            socket,
            addr,
            name: "".to_string(),
        };

        tokio::spawn(async move {
            handle(&mut client).await;
        });
    }
}
