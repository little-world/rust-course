

## std::net Overview

Commonly used types:

* `TcpListener` – for listening to incoming TCP connections.
* `TcpStream` – for reading/writing on an open TCP connection.
* `UdpSocket` – for sending/receiving UDP packets.
* `IpAddr`, `Ipv4Addr`, `Ipv6Addr` – IP address types.
* `SocketAddr` – combines IP and port.

---

## Basic TCP Server

```rust
use std::net::TcpListener;
use std::io::{Read, Write};

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    println!("Listening on 127.0.0.1:7878");

    for stream in listener.incoming() {
        let mut stream = stream?;
        println!("Connection established!");

        let mut buffer = [0; 512];
        let bytes_read = stream.read(&mut buffer)?;
        println!("Received: {}", String::from_utf8_lossy(&buffer[..bytes_read]));

        stream.write_all(b"Hello from server!")?;
    }

    Ok(())
}
```

---

## Basic TCP Client

```rust
use std::net::TcpStream;
use std::io::{self, Write, Read};

fn main() -> io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:7878")?;
    stream.write_all(b"Hello from client!")?;

    let mut buffer = [0; 512];
    let bytes_read = stream.read(&mut buffer)?;
    println!("Server replied: {}", String::from_utf8_lossy(&buffer[..bytes_read]));

    Ok(())
}
```

---

## UDP Example

```rust
use std::net::UdpSocket;

fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:34254")?;
    socket.send_to(b"hello world", "127.0.0.1:8080")?;

    let mut buf = [0; 512];
    let (amt, src) = socket.recv_from(&mut buf)?;
    println!("Received {} bytes from {}: {}", amt, src, String::from_utf8_lossy(&buf[..amt]));

    Ok(())
}
```

---

## Parse and Inspect IP Addresses

```rust
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

fn main() {
    let ip: IpAddr = "127.0.0.1".parse().unwrap();
    println!("IP: {:?}", ip);

    let socket: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    println!("Socket: {:?}", socket);
}
```

---