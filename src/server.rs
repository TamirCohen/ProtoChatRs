use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};

use std::error::Error;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::sync::{mpsc};
use tokio::select;
use std::ops::Deref;

struct ChatServer{
    listener: TcpListener,
}


struct Client{
    socket: TcpStream,
    rx : mpsc::Receiver<Vec<u8>>,
    clients_tx: Arc<Mutex<HashMap<SocketAddr, mpsc::Sender<Vec<u8>>>>>,
    address: SocketAddr
}

impl Client{

    async fn handle_client(&'static mut self) -> Result<(), Box<dyn Error>>
    {
        
        // In a loop, read data from the socket and write the data back.
        loop {
            let mut buf = vec![0; 1024];

            let n = self.socket
                .read(&mut buf)
                .await?;
            
            if n == 0 {
                return Ok(());
            }
            
            {
                for (addr, tx) in self.clients_tx.lock()?.deref()
                {
                    if *addr != self.address{
                        tx.send(buf.clone());
                    } 
                }
                
            }

            //TODO THATSS BAD :) ITs not error if all the clients exist.
            //TODO what happens if client exist? who cleans its state?
            let received_string = self.rx.recv().await.unwrap_or(
            {
                return Err("Channel closed".into());
            });
            
            self.socket
                .write_all(&received_string)
                .await?;
        }
    }

}

struct ChatMessage{
    message: String,
    source: String
}
impl ChatServer{

    async fn init() -> Result<ChatServer, Box<dyn Error>>
    {
        let listener = TcpListener::bind("127.0.0.1:8080".to_string()).await?; 
        let chat = ChatServer{
            listener};
        Ok(chat)
    }

    async fn run(&self) -> Result<(), Box<dyn Error>>
    {
        
        let clients_tx = Arc::new(Mutex::new(HashMap::new()));
        
        loop {
            // Asynchronously wait for an inbound socket.
            
            let (socket, address) = self.listener.accept().await?;
            
            let (tx, rx) = mpsc::channel::<Vec<u8>>(10);
            
            clients_tx.lock().unwrap().insert(address, tx);
            

            let mut client = Client{
                socket,
                rx,
                clients_tx: clients_tx.clone(),
                address
            };

            tokio::spawn(async move{
                    let mut client = client;
                    client.handle_client().await;
                }
            );
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    ChatServer::init().await?.run().await?;
    Ok(())
}