use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use std::error::Error;
use std::sync::{Arc, Mutex};


struct ChatServer{
    listener: TcpListener,
    clients: Arc<Mutex<Vec<TcpStream>>>
}

struct Client{
    socket: TcpStream,
    clients: Arc<Mutex<Vec<TcpStream>>>
}

impl Client{

    async fn handle_client(&mut self)
    {
        let mut buf = vec![0; 1024];

        
        // In a loop, read data from the socket and write the data back.
        loop {
            let n = self.socket
                .read(&mut buf)
                .await
                .expect("failed to read data from socket");

            if n == 0 {
                return;
            }

            self.socket
                .write_all(&buf[0..n])
                .await
                .expect("failed to write data to socket");
        }
    }

}

impl ChatServer{

    async fn init() -> Result<ChatServer, Box<dyn Error>>
    {
        let listener = TcpListener::bind("127.0.0.1:8080".to_string()).await?; 
        let clients = Arc::new(Mutex::new(Vec::new()));
        let chat = ChatServer{
            listener,
            clients};
        Ok(chat)
    }

    async fn run(&self) -> Result<(), Box<dyn Error>>
    {
        loop {
            // Asynchronously wait for an inbound socket.

            let (socket, _) = self.listener.accept().await?;

            let clients = Arc::clone(&self.clients);
            let mut client = Client{
                socket,
                clients
            };

            tokio::spawn(async move{
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