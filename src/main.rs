use std::{
    env,
    error::Error,
    fs::File,
    io::{BufReader, Read},
    net::Ipv4Addr,
};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let proxy_addr = env::args()
        .nth(2)
        .unwrap_or_else(|| "127.0.0.1:9050".to_string());

    let mut stream = TcpStream::connect(proxy_addr).await?;

    // testing with a pre-configured listening server 35.224.248.232:3000

    let port = 3000;

    let target_host_ip = Ipv4Addr::new(35, 224, 248, 232);
    let target_host_ip_bytes = target_host_ip.octets();

    // carry out handshake & establish connection with host

    if let Err(handshake_result) = socks_handshake(&mut stream).await {
        println!("handshake failed: {}", handshake_result);
    } else {
        if let Err(connection_established) =
            establish_connection(&mut stream, port, target_host_ip_bytes).await
        {
            println!("Connection Error: {}", connection_established);
        };
    };

    // reading file content and writing it into buffer

    let file = File::open("./message.txt")?;
    let mut buffer: Vec<u8> = Vec::with_capacity(14);
    let mut reader = BufReader::new(file);
    reader.read_to_end(&mut buffer)?;

    // write buffer to stream

    stream.write_all(&buffer).await?;

    Ok(())
}

type HandshakeResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
struct HandshakeStatus<T> {
    details: T,
}

impl<T> HandshakeStatus<T> {
    fn details(&self) -> &T {
        &self.details
    }
}

async fn socks_handshake(stream: &mut TcpStream) -> HandshakeResult<bool> {
    stream
        .write_all(&[
            0x05, // SOCKS version 5
            0x01, //  1 authentication method supported by the client
            0x00, //  No authentication
        ])
        .await?;

    let mut buf = [0u8; 2];
    stream.read_exact(&mut buf).await?;

    let hand_success = HandshakeStatus { details: true };
    let hand_failed = HandshakeStatus { details: false };

    if buf == [0x05, 0x00] {
        eprintln!("Socks Handshake successful!");
        return Ok(*hand_success.details());
    } else {
        eprintln!("Socks Handshake failed :(");
        return Ok(*hand_failed.details());
    }
}

type ConnectionEstablished = Result<(), Box<dyn Error>>;

async fn establish_connection(
    stream: &mut TcpStream,
    port: u16,
    target_host_ip_bytes: [u8; 4],
) -> ConnectionEstablished {
    let request = [
        0x05, // SOCKS version
        0x01, // establish a TCP/IP stream connection
        0x00, // reserved byte
        0x01, //  IPv4 address, followed by 4 bytes IP
        target_host_ip_bytes[0],
        target_host_ip_bytes[1],
        target_host_ip_bytes[2],
        target_host_ip_bytes[3],
        ((port >> 8) & 0xFF) as u8, // High byte of the port
        (port & 0xFF) as u8,        // Low byte of the port
    ];

    eprintln!("Connecting to host...");
    stream.write_all(&request).await?;

    let mut buf = [0u8; 10];
    stream.read_exact(&mut buf).await?;

    if buf[1] == 0x00 {
        eprintln!("Connection established!")
    } else {
        eprintln!("Something went wrong: {:?}", buf)
    }

    Ok(())
}
