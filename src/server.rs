use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};

use std::error::Error;
use std::sync::{Arc};
use std::collections::HashMap;
use tokio::sync::{mpsc, Mutex};
use futures::executor; 

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
    async fn new(socket: TcpStream,
            rx: mpsc::Receiver<Vec<u8>>,
            tx: mpsc::Sender<Vec<u8>>,
            clients_tx: Arc<Mutex<HashMap<SocketAddr, mpsc::Sender<Vec<u8>>>>>,
            address: SocketAddr
        ) -> Client
    {
        clients_tx.lock().await.insert(address, tx);
        
        Client{
            socket,
            rx,
            clients_tx: clients_tx.clone(),
            address
        }

    }

    async fn handle_client(mut self) -> Result<(), Box<dyn Error>>
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
                for (addr, tx) in self.clients_tx.lock().await.iter_mut()
                   {
                       if *addr != self.address{
                           tx.send(buf.clone()).await?;
                       } 
                   }

            }   

            //TODO THATSS BAD :) ITs not error if all the clients exist.
            //TODO what happens if client exist? who cleans its state?
            
            let received_string = self.rx.recv().await.unwrap();
            
            self.socket
                .write_all(&received_string)
                .await?;
        }
    }

}

impl Drop for Client{
    
    fn drop(&mut self)
    {
        // executor::block_on(self.clients_tx.lock().remove(&self.address));
        let mut guard = executor::block_on(self.clients_tx.lock());
        guard.remove(&self.address);

        println!("DROPPED CLIENT: {:?}", self.address)
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
            
            

            let client = Client::new(
                socket,
                rx,
                tx,
                clients_tx.clone(),
                address
            ).await;

            tokio::spawn(async move{
                    Client::handle_client(client).await;
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