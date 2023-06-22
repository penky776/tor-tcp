use std::net::SocketAddr;

use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:3000").await?;

    match listener.accept().await {
        Ok((mut socket, addr)) => {
            println!("new client: {:?}", addr);
            process_socket(&mut socket, addr).await;
        }
        Err(e) => println!("couldn't get client: {:?}", e),
    }

    Ok(())
}

async fn process_socket(stream: &mut TcpStream, addr: SocketAddr) {
    let mut buf = [0u8; 14];

    let read_me = stream
        .read(&mut buf)
        .await
        .expect("failed to read received data");

    let response = String::from_utf8_lossy(&buf[..read_me]);

    println!("{}: {}", addr, response);
}
