mod actions;

use std::{
    error::Error,
    fs::File,
    io::{BufReader, Read},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
    sync::Arc,
};

use actions::{file_exists, list_files};
use quinn::{
    crypto::rustls::QuicServerConfig,
    rustls::{
        crypto::CryptoProvider,
        pki_types::{CertificateDer, PrivateKeyDer},
    },
};
use rustls_pemfile::{certs, private_key};

#[tokio::main]
async fn main() {
    let path: String = std::env::args().last().unwrap();
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

async fn handle(con: quinn::Incoming, base: String) -> Result<(), Box<dyn Error>> {
    let mut conn = con.accept()?.await?;
    loop {
        let gram = conn.read_datagram().await?;
        let mut gram = gram.bytes();
        let action: u8 = match gram.next() {
            Some(Ok(x)) => x,
            Some(Err(e)) => {
                println!("{}", e);
                continue;
            }
            None => continue,
        };
        match action {
            0x0 => list_files(&mut conn, gram, &base).await,
            0x1 => file_exists(&mut conn, gram, &base).await,
            _ => continue,
        }
    }
}

fn load_certs(path: &Path) -> std::io::Result<Vec<CertificateDer<'static>>> {
    certs(&mut BufReader::new(File::open(path)?)).collect()
}

fn load_key(path: &Path) -> std::io::Result<PrivateKeyDer<'static>> {
    Ok(private_key(&mut BufReader::new(File::open(path)?))
        .unwrap_or_else(|e| panic!("{}", e))
        .unwrap())
}
