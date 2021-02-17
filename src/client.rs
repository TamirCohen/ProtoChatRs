pub mod protos{
    pub mod messages;
}

use protos::messages::ChatMessage;
use protobuf::Message;

use std::io::prelude::*;
use std::net::TcpStream;

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:8080")?;

    let mut out_msg = ChatMessage::new();
    out_msg.set_content(String::from("Hi everyone, this is my test message"));
    out_msg.set_broadcast(true);
    let out_bytes: Vec<u8> = out_msg.write_to_bytes().unwrap();

    stream.write(&out_bytes)?;
    // stream.read(&mut [0; 128])?;
    println!("SENT {:?}", out_bytes);
    Ok(())
    

}