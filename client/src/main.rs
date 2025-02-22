use std::{
    fs::File,
    io::{self, BufReader, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use quinn::{
    crypto::rustls::QuicClientConfig, Connection, ReadExactError, RecvStream, SendStream,
    WriteError,
};
use rustls_pemfile::certs;

#[tokio::main]
async fn main() {
    let local = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 9460);
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9459);
    let mut roots = tokio_rustls::rustls::RootCertStore::empty();
    roots
        .add(
            certs(&mut BufReader::new(File::open("ca.crt").unwrap()))
                .next()
                .unwrap()
                .unwrap(),
        )
        .unwrap();
    tokio_rustls::rustls::crypto::ring::default_provider()
        .install_default()
        .unwrap();
    let client_crypto = tokio_rustls::rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    let client_config =
        quinn::ClientConfig::new(Arc::new(QuicClientConfig::try_from(client_crypto).unwrap()));
    let mut endpoint = quinn::Endpoint::client(local).unwrap();
    endpoint.set_default_client_config(client_config);
    let mut conn = endpoint.connect(addr, "localhost").unwrap().await.unwrap();
    loop {
        let line: String = text_io::read!("{}\n");
        cmd(&mut conn, line).await;
    }
    //conn.close(0u32.into(), b"DONE");
    //endpoint.wait_idle().await;
}

async fn cmd(conn: &mut Connection, data: String) -> Option<()> {
    let mut data = data.split_whitespace();
    let cmd = data.next()?;
    let parsed = match cmd {
        "ls" => 0x0,
        "exists" => 0x1,
        "meta" => 0x2,
        "size" => 0x3,
        "mkdir" => 0x4,
        "touch" => 0x5,
        "mv" => 0x6,
        "cp" => 0x7,
        "rm" => 0x8,
        "dl" => 0x9,
        "chmod" => 0xA,
        _ => return None,
    };
    let args: String = data.collect();
    let mut total = args.into_bytes();
    total.push(parsed);
    let (mut send, mut revc) = conn.open_bi().await.ok()?;
    println!("Sending {:?}", total);
    write_bytes(&mut send, &mut revc, total).await.ok()?;
    loop {
        let res = read_bytes(&mut send, &mut revc).await.ok()?;
        println!("Received");
        println!("{}", String::from_utf8(res).ok()?);
    }
}

async fn read_bytes(
    send: &mut SendStream,
    recv: &mut RecvStream,
) -> Result<Vec<u8>, ReadExactError> {
    let mut length = [0u8; 4];
    recv.read_exact(&mut length).await?;
    let length: usize = u32::from_be_bytes(length) as usize;
    let mut data = Vec::with_capacity(length);
    recv.read_exact(&mut data).await?;
    Ok(data)
}

async fn write_bytes(
    send: &mut SendStream,
    recv: &mut RecvStream,
    bytes: Vec<u8>,
) -> Result<(), WriteError> {
    let length: u32 = bytes.len().try_into().unwrap_or(0);
    send.write_all(&length.to_be_bytes()).await?;
    let mut buf = [0u8; 1];
    recv.read_exact(&mut buf).await.unwrap();
    send.write(&bytes).await?;
    send.stopped().await.unwrap();
    Ok(())
}
