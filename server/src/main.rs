mod actions;
mod error;

use std::{
    error::Error,
    fs::File,
    io::{BufReader, Read},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
    sync::Arc,
};

use actions::*;
use error::*;
use quinn::{
    crypto::rustls::QuicServerConfig,
    rustls::pki_types::{CertificateDer, PrivateKeyDer},
    ReadExactError, RecvStream, SendStream, WriteError,
};
use rustls_pemfile::{certs, private_key};

#[tokio::main]
async fn main() {
    let path: PathBuf = PathBuf::from(std::env::args().last().unwrap());
    let certs = load_certs(&PathBuf::from("cert.pem")).unwrap();
    let key = load_key(&PathBuf::from("key.pem")).unwrap();
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9459);
    tokio_rustls::rustls::crypto::ring::default_provider()
        .install_default()
        .unwrap();
    let crypto_config = tokio_rustls::rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .unwrap();
    let mut server_conf = quinn::ServerConfig::with_crypto(Arc::new(
        QuicServerConfig::try_from(crypto_config).unwrap(),
    ));
    let transport_config = Arc::get_mut(&mut server_conf.transport).unwrap();
    transport_config.max_concurrent_uni_streams(0_u8.into());
    let endpoint = quinn::Endpoint::server(server_conf, addr).unwrap();
    while let Some(con) = endpoint.accept().await {
        println!("Connected: {}", con.remote_address());
        let base = path.clone();
        let handle = handle(con, base);
        if let Err(e) = tokio::spawn(async move {
            if let Err(e) = handle.await {
                println!("Stream terminated with failure: {}", e)
            }
        })
        .await
        {
            println!("Could not spawn new task: {}", e)
        };
    }
}

async fn handle(con: quinn::Incoming, base: PathBuf) -> Result<(), Box<dyn Error>> {
    let conn = con.accept()?.await?;
    loop {
        let (send, recv) = conn.accept_bi().await?;
        println!("Opened stream");
        let base = base.clone();
        tokio::spawn(async move { println!("{:?}", parse(send, recv, base).await) });
    }
}

async fn parse(
    mut send: SendStream,
    mut recv: RecvStream,
    base: PathBuf,
) -> Result<(), Box<dyn Error>> {
    let mut command = read_bytes(&mut send, &mut recv).await.unwrap();
    let action: u8 = command.pop().ok_or(ParseError).unwrap();
    let result: Option<Vec<Vec<u8>>> = match action {
        0x0 => list_files(command, &base).await,
        0x1 => file_exists(command, &base).await,
        0x2 => file_meta(command, &base).await,
        0x3 => file_size(command, &base).await,
        0x4 => create_dir(command, &base).await,
        0x5 => create_file(command, &base).await,
        0x6 => move_file(command, &base).await,
        0x7 => copy_file(command, &base).await,
        0x8 => remove_file(command, &base).await,
        0x9 => download(command, &base).await,
        0xA => chmod(command, &base).await,
        _ => return Err(Box::new(InvalidActionError)),
    };
    match result {
        Some(o) => {
            for y in o {
                println!("Sending {:?}", y);
                if let Err(e) = write_bytes(&mut send, &mut recv, y).await {
                    println!("Error sending bytes: {}", e)
                }
            }
        }
        None => write_bytes(&mut send, &mut recv, vec![0x1F]).await?,
    }
    Ok(())
}

async fn read_bytes(
    send: &mut SendStream,
    recv: &mut RecvStream,
) -> Result<Vec<u8>, ReadExactError> {
    let mut length = [0u8; 4];
    recv.read_exact(&mut length).await?;
    let length: usize = u32::from_be_bytes(length) as usize;
    let mut data = Vec::with_capacity(length);
    send.write_all(&[0]).await.unwrap();
    recv.read(&mut data).await?;
    Ok(data)
}

async fn write_bytes(
    send: &mut SendStream,
    recv: &mut RecvStream,
    bytes: Vec<u8>,
) -> Result<(), WriteError> {
    let length: u32 = bytes.len().try_into().unwrap_or(0);
    send.write_all(&length.to_be_bytes()).await?;
    send.write_all(&bytes).await?;
    Ok(())
}

fn load_certs(path: &Path) -> std::io::Result<Vec<CertificateDer<'static>>> {
    certs(&mut BufReader::new(File::open(path)?)).collect()
}

fn load_key(path: &Path) -> std::io::Result<PrivateKeyDer<'static>> {
    Ok(private_key(&mut BufReader::new(File::open(path)?))
        .unwrap_or_else(|e| panic!("{}", e))
        .unwrap())
}
