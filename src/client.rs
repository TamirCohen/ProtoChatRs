pub mod protos{
    pub mod messages;
}

use protos::messages::ChatMessage;
use protobuf::Message;
use tokio::net::{TcpListener, TcpStream};
use futures::{future::FutureExt, select, executor};
use tokio::io::{AsyncReadExt, AsyncWriteExt, Lines};
use std::{thread, time};
use std::io::{stdin,stdout,Write};
use tokio::sync::mpsc;


fn get_input(tx: &mut mpsc::Sender<String>)
{
    let mut s=String::new();
    let _= stdout().flush();
    stdin().read_line(&mut s).expect("Did not enter a correct string");
    executor::block_on(tx.send(s));
}

async fn read_stream(stream: &mut TcpStream) -> Result<(), ()>
{
    let mut recv_buf = vec![0; 1024];

    let received_length = stream.read(&mut recv_buf).await.unwrap();
    if received_length == 0
    {
        return Err(());
    }
    let recv_message: ChatMessage = protobuf::Message::parse_from_bytes(&recv_buf[..received_length]).unwrap();
    println!("{}", recv_message.get_content());
    Ok(())
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    let (mut tx, mut rx) = mpsc::channel::<String>(10);
    
    let join_handle = std::thread::spawn(move || {
        loop {get_input(&mut tx)}
    });


    loop{
        select!{
            user_input = rx.recv().fuse() => {
                let mut out_msg = ChatMessage::new();
                out_msg.set_content(user_input.unwrap());
                out_msg.set_broadcast(true);
                let out_bytes: Vec<u8> = out_msg.write_to_bytes().unwrap();
                stream.write(&out_bytes).await.unwrap();

            }
            res = read_stream(&mut stream).fuse() => {
                match res{
                    Err(_) => return Ok(()),
                    Ok(_) => continue
                }
            }
        }
    }

    Ok(())
    
}