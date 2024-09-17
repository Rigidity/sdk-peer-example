use std::{net::SocketAddr, str::FromStr};

use chia::{
    protocol::{NewPeakWallet, ProtocolMessageTypes},
    traits::Streamable,
};
use chia_wallet_sdk::{connect_peer, create_tls_connector, load_ssl_cert, NetworkId};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cert = load_ssl_cert("wallet.crt", "wallet.key")?;
    let tls = create_tls_connector(&cert)?;

    // `dig dns-introducer.chia.net` for example to find peers
    let (peer, mut receiver) = connect_peer(
        NetworkId::Mainnet,
        tls,
        SocketAddr::from_str("31.192.64.239:8444")?,
    )
    .await?;

    println!("Connected to {}", peer.socket_addr());

    while let Some(message) = receiver.recv().await {
        if message.msg_type != ProtocolMessageTypes::NewPeakWallet {
            continue;
        }

        let message = NewPeakWallet::from_bytes(&message.data)?;

        println!("{:?}", message);
    }

    Ok(())
}
