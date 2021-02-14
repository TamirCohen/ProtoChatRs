use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use std::error::Error;

struct ChatServer{
    listener: TcpListener,
    clients: Vec<TcpStream>
}

impl ChatServer{
    
    async fn handle_client(mut socket: TcpStream)
    {
        let mut buf = vec![0; 1024];
        
        // In a loop, read data from the socket and write the data back.
        loop {
            let n = socket
                .read(&mut buf)
                .await
                .expect("failed to read data from socket");

            if n == 0 {
                return;
            }

            socket
                .write_all(&buf[0..n])
                .await
                .expect("failed to write data to socket");
        }
    }
    async fn init() -> Result<ChatServer, Box<dyn Error>>
    {
        let listener = TcpListener::bind("127.0.0.1:8080".to_string()).await?; 
        
        let chat = ChatServer{
            listener,
            clients: Vec::new()};  
        Ok(chat)
    }

    async fn run(&self) -> Result<(), Box<dyn Error>>
    {
        loop {
            // Asynchronously wait for an inbound socket.

            let (socket, _) = self.listener.accept().await?;
            tokio::spawn(async move{
                    ChatServer::handle_client(socket).await;
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