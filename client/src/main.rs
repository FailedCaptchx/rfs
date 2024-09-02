use std::{
    fs::File,
    io::{BufReader, Read},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use quinn::{
    crypto::rustls::QuicClientConfig, rustls::client::danger::ServerCertVerifier, Connection,
};
use rustls_pemfile::certs;
use tokio::sync::Mutex;

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
    let conn = endpoint.connect(addr, "localhost").unwrap().await.unwrap();
    conn.send_datagram_wait(bytes::Bytes::from(Box::from([0, 0])))
        .await
        .unwrap();
    let conn = Mutex::from(Arc::from(conn));
    tokio::task::spawn(async move { listen(conn) });
    //conn.close(0u32.into(), b"DONE");
    //endpoint.wait_idle().await;
}

async fn listen(conn: Mutex<Arc<Connection>>) {
    loop {
        let res = conn.lock().await.read_datagram().await.unwrap();
        println!("{:?}", res.bytes());
    }
}
