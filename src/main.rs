use std::{env, ffi::OsString, net::SocketAddr};

use once_cell::sync::Lazy;
use tokio::{
    net::{TcpListener, TcpStream},
    signal, try_join,
};

fn env_var(env_var: &str) -> OsString {
    return env::var_os(env_var)
        .unwrap_or_else(|| panic!("Missing environment variable: {env_var}"));
}

static LISTEN: Lazy<OsString> = Lazy::new(|| env_var("LISTEN"));
static REMOTE: Lazy<OsString> = Lazy::new(|| env_var("REMOTE"));

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Running...");

    let proxy_server = TcpListener::bind(LISTEN.to_str().unwrap()).await?;

    tokio::select! {
        _ = signal::ctrl_c() => {
            println!("Exiting...")
        },
        _ = async {
            loop {
                let (socket, _) = proxy_server.accept().await.unwrap();
                let peer = socket.peer_addr().unwrap();
                tokio::spawn(async move {
                    record_inbound_peer(peer);
                });
                tokio::spawn(async move {
                    handle_client_conn(socket).await.ok();
                });
            }
        } => {}
    }

    Ok(())
}

async fn handle_client_conn(mut client_conn: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let mut main_server_conn = TcpStream::connect(REMOTE.to_str().unwrap()).await?;

    let (mut client_recv, mut client_send) = client_conn.split();
    let (mut server_recv, mut server_send) = main_server_conn.split();

    let handle_one = async { tokio::io::copy(&mut server_recv, &mut client_send).await };
    let handle_two = async { tokio::io::copy(&mut client_recv, &mut server_send).await };

    try_join!(handle_one, handle_two)?;

    Ok(())
}

fn record_inbound_peer(peer: SocketAddr) {
    println!("Inbound IP: {}", peer.ip().to_string());
}
