use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};
use std::net::SocketAddr;
use std::error::Error;
use std::sync::{Arc};
use std::collections::HashMap;
use futures::{future::FutureExt, select, executor};
use protos::messages::ChatMessage;
use protobuf::Message;

mod protos{
    pub mod messages;
}

struct ChatServer{
    listener: TcpListener,
}

struct Client{
    socket: TcpStream,
    rx : mpsc::Receiver<Vec<u8>>,
    clients_tx: Arc<Mutex<HashMap<SocketAddr, mpsc::Sender<Vec<u8>>>>>,
    address: SocketAddr
}

enum ClientError{
    ReceiveError,
    Disconnected,
    ParseError
}

//TODO module it up

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

    async fn read_from_socket(socket: &mut TcpStream) -> Result<ChatMessage, ClientError>
    {
        let mut recv_buf = vec![0; 1024];

        let recevied_length = socket
            .read(&mut recv_buf)
            .await
            .map_err(|_| ClientError::ReceiveError)?;
        
        if recevied_length == 0 {
            return Err(ClientError::Disconnected);
        }

        let recv_message: ChatMessage = protobuf::Message::parse_from_bytes(&recv_buf[..recevied_length]).
            map_err(|_| ClientError::ParseError)?;
        //TODO add tokio framing and stuff
        Ok(recv_message)
    }
    
    async fn broadcast_message(&self, message: ChatMessage) -> Result<(), Box<dyn Error>>
    {
        let buf: Vec<u8> = message.write_to_bytes().unwrap();

        for (addr, tx) in self.clients_tx.lock().await.iter_mut()
        {
            if *addr != self.address{
                tx.send(buf.clone()).await?;
            } 
        }
        Ok(())
    }

    async fn read_from_pipe(rx: &mut mpsc::Receiver<Vec<u8>>) -> Result< Vec::<u8>, Box<dyn Error>>
    {
        Ok(rx.recv().await.unwrap())
    }

    async fn handle_message(&self, message: ChatMessage) -> Result< (), Box<dyn Error>>
    {
        println!("RECEIVED MESSAGE {:?}", message);
        if message.get_broadcast()
        {
            self.broadcast_message(message).await?;
        }
        Ok(())
    }

    async fn handle_client(mut self) -> Result<(), Box<dyn Error>>
    {
        //TODO remove this idiotic block_on. It is just sync code in that way (NOOB MOVE TAMIR)

        //TODO handle all those errors, remove those hacky unwrap()
        loop
        {
            select! {
                received_from_client = Client::read_from_socket(&mut self.socket).fuse() =>{
                {
                    match received_from_client
                    {
                        Err(ClientError::Disconnected) => return Ok(()),
                        Err(ClientError::ParseError) => continue,
                        Err(ClientError::ReceiveError) => panic!("Socket receive Error"),
                        Ok(message) => executor::block_on(self.handle_message(message))?
                    }
                }}

                received_from_peer = Client::read_from_pipe(&mut self.rx).fuse() => {
                    executor::block_on(
                    self.socket
                    .write_all(&received_from_peer.unwrap())
                    )?;
                }
            };
        }

    }

}

impl Drop for Client{
    
    fn drop(&mut self)
    {
        let mut guard = executor::block_on(self.clients_tx.lock());
        guard.remove(&self.address);

        println!("DROPPED CLIENT: {:?}", self.address)
    }
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
                    Client::handle_client(client).await.unwrap();
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